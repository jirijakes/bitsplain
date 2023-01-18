use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, RangeFrom, RangeTo};
use std::rc::Rc;

use nom::error::ParseError;
use nom::{AsBytes, IResult, InputIter, InputLength, InputTake, Needed, Offset, Parser, Slice};

use crate::ann::Ann;
use crate::tree::*;
use crate::value::*;
use crate::Void;

/// Pointer to a location in parsed data.
///
/// A bookmark can be crated from annotated span by calling [`bookmark`](Annotated::bookmar).
/// One wants to use bookmark when inserting annotation to a previous location but later
/// in the parsing. For example, when some data become available later than their annotation
/// should appear.
///
/// When having a bookmark, new annotation can be inserted at its location by calling
/// [`insert_at`](Annotated<Fragment>).
#[derive(Copy, Clone, Debug)]
pub struct Bookmark(Option<(usize, usize)>);

/// Relative placement of an [`Appendix`] to byte range.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Place {
    /// Appendix is inserted after byte range.
    After,
    /// Appendix is inserted before byte range.
    Before,
}

/// Annotation that is inserted externally to the parser. Typically it will be used
/// insert an annotation that is computed from parsed data. Appendices are rendered
/// as virtual leaves (see [`Tree`]).
#[derive(Clone, Debug)]
struct Appendix {
    /// Initial byte to which the annotation is attached.
    from: usize,
    /// Final byte to which the annotation is attached.
    to: usize,
    /// Where relative to the byte range, appendix is inserted.
    place: Place,
    /// Annotation information.
    information: Information,
}

/// Nom parser that stores user-defined annotations during parsing.
#[derive(Clone, Debug)]
pub struct Annotated<Fragment> {
    /// Index of next parsed value. Indices start at 0 and each parsed value
    /// will increase them by 1.
    next_index: usize,
    /// Offset of byte to be parsed next.
    next_offset: usize,
    /// Fragment (raw data) to be parsed next.
    next_fragment: Fragment,
    /// Tree of annotations.
    tree: Vec<Tree>,
    /// Most recently inserted range. None if no range inserted yet.
    last_range: Option<(usize, usize)>,
    /// Additional data that parsers can provide.
    data: HashMap<&'static str, String>,
    /// Tags.
    tags: Vec<Tag>,
    /// Additional annotations that parsers can insert.
    appendices: Rc<RefCell<Vec<Appendix>>>,
}

impl<Fragment> Annotated<Fragment> {
    /// Generate a bookmark for current position.
    pub fn bookmark(&self) -> Bookmark {
        Bookmark(self.last_range)
    }

    /// Insert an annotation at the bookmark's position.
    pub fn insert_at(&self, bookmark: &Bookmark, ann: Ann<Void>) {
        if let Some((from, to)) = bookmark.0 {
            self.appendices.borrow_mut().push(Appendix {
                from,
                to,
                place: Place::After,
                information: Information {
                    annotation: ann.label,
                    value: ann.value.resolve_static().unwrap_or(Value::Nil),
                    doc: ann.doc.clone(),
                    splain: ann.splain.resolve_static(),
                    data: HashMap::new(),
                    tags: vec![],
                },
            });
        }
    }

    // pub fn insert_before(
    //     &self,
    //     bookmark: &Bookmark,
    //     annotation: &'static str,
    //     value: Value,
    //     doc: Option<String>,
    // ) {
    //     if let Some((from, to)) = bookmark.0 {
    //         self.appendices.borrow_mut().push((
    //             from,
    //             to,
    //             Place::Before,
    //             Information {
    //                 annotation: annotation.to_string(),
    //                 value,
    //                 doc,
    //                 data: HashMap::new(),
    //             },
    //         ));
    //     }
    // }

    /// Insert an annotation to current position.
    ///
    /// To insert annotations to a previous position, see [`insert_at`].
    pub fn insert(&self, ann: Ann<Void>) {
        if let Some((from, to)) = self.last_range {
            self.appendices.borrow_mut().push(Appendix {
                from,
                to,
                place: Place::After,
                information: Information {
                    annotation: ann.label,
                    value: ann.value.resolve_static().unwrap_or(Value::Nil),
                    doc: ann.doc.clone(),
                    splain: ann.splain.resolve_static(),
                    data: HashMap::new(),
                    tags: vec![],
                },
            });
        }
    }

    /// Place appendices to the proper place inside tree. Returns a copy of the original tree.
    fn inject_appendices(tree: Vec<Tree>, app: &[Appendix]) -> Vec<Tree> {
        let mut new_tree = vec![];

        tree.into_iter().for_each(|t| match t {
            Tree::Group {
                path,
                location,
                information,
                children,
            } => new_tree.push(Tree::Group {
                path,
                location,
                information,
                children: Self::inject_appendices(children, app),
            }),
            Tree::Leaf(Leaf::Real(r)) => {
                let from = r.location.from;
                let to = r.location.to;
                new_tree.push(Tree::Leaf(Leaf::Real(r)));
                app.iter()
                    .filter(|app| app.place == Place::After && app.from == from && app.to == to)
                    .for_each(|app| {
                        new_tree.push(Tree::Leaf(Leaf::Virtual(VirtualLeaf {
                            information: app.information.clone(),
                            path: vec![],
                        })))
                    });
            }
            leaf => new_tree.push(leaf),
        });

        new_tree
    }

    /// Traverse the tree and set path of each node.
    fn inject_paths(tree: &mut [Tree], prefix: Vec<String>) {
        tree.iter_mut().enumerate().for_each(|(i, t)| match t {
            Tree::Leaf(Leaf::Real(RealLeaf { path, .. })) => {
                path.append(&mut prefix.clone());
                path.push(i.to_string());
            }
            Tree::Leaf(Leaf::Virtual(VirtualLeaf { path, .. })) => {
                path.append(&mut prefix.clone());
                path.push(i.to_string());
            }
            Tree::Group { path, children, .. } => {
                path.append(&mut prefix.clone());
                path.push(i.to_string());
                Self::inject_paths(children, path.clone())
            }
        });
    }

    /// Replace annotations by data field 'annotation', if it exsts, and bake
    /// enumerations. This allows the specify annotation ex post.
    fn bake_annotations(tree: &mut Tree, enumeration: usize) {
        match tree {
            Tree::Leaf(Leaf::Real(RealLeaf { information, .. })) => {
                if let Some(x) = information.data.remove("annotation") {
                    information.annotation = x;
                };
            }
            Tree::Leaf(Leaf::Virtual(VirtualLeaf { information, .. })) => {
                if let Some(x) = information.data.remove("annotation") {
                    information.annotation = x;
                };
            }
            Tree::Group {
                information,
                children,
                ..
            } => {
                if let Some(x) = information.data.remove("annotation") {
                    information.annotation = x;
                } else if information.has_data("list", "enumerate") {
                    information.annotation = (enumeration - 1).to_string();
                };
                children
                    .iter_mut()
                    .enumerate()
                    .for_each(|(en, c)| Self::bake_annotations(c, en));
            }
        }
    }

    /// Render annotations.
    pub fn annotations(self) -> Annotations {
        let mut tree = Self::inject_appendices(self.tree, &self.appendices.as_ref().borrow());
        Self::inject_paths(&mut tree, vec![]);
        tree.iter_mut().for_each(|t| Self::bake_annotations(t, 0));
        Annotations::from_trees(tree)
    }
}

impl<Fragment> Annotated<Fragment> {
    pub fn new(fragment: Fragment) -> Annotated<Fragment> {
        Annotated {
            next_index: 0,
            next_offset: 0,
            next_fragment: fragment,
            tree: vec![],
            data: HashMap::new(),
            tags: vec![],
            appendices: Rc::new(RefCell::new(vec![])),
            last_range: None,
        }
    }

    /// Add a tag to the current span.
    #[must_use]
    #[inline]
    pub fn add_tag(self, tag: Tag) -> Self {
        let mut tags = self.tags;
        tags.push(tag);
        Annotated {
            next_index: self.next_index,
            next_offset: self.next_offset,
            next_fragment: self.next_fragment,
            data: self.data,
            tags,
            tree: self.tree,
            appendices: self.appendices,
            last_range: self.last_range,
        }
    }

    #[must_use]
    #[inline]
    pub fn with(self, key: &'static str, value: &'static str) -> Self {
        let mut data = self.data;
        data.insert(key, value.to_string());
        Annotated {
            next_index: self.next_index,
            next_offset: self.next_offset,
            next_fragment: self.next_fragment,
            data,
            tags: self.tags,
            tree: self.tree,
            appendices: self.appendices,
            last_range: self.last_range,
        }
    }
}

impl<Fragment> Deref for Annotated<Fragment> {
    type Target = Fragment;

    fn deref(&self) -> &Self::Target {
        &self.next_fragment
    }
}

impl<Fragment> InputIter for Annotated<Fragment>
where
    Fragment: InputIter + Eq + Clone + Sync + Send,
{
    type Item = Fragment::Item;

    type Iter = Fragment::Iter;

    type IterElem = Fragment::IterElem;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.next_fragment.iter_indices()
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.next_fragment.iter_elements()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.next_fragment.position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        self.next_fragment.slice_index(count)
    }
}

impl<Fragment> InputLength for Annotated<Fragment>
where
    Fragment: InputLength,
{
    #[inline]
    fn input_len(&self) -> usize {
        self.next_fragment.input_len()
    }
}

impl<Fragment> InputTake for Annotated<Fragment>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
{
    #[inline]
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<Fragment, Range> Slice<Range> for Annotated<Fragment>
where
    Fragment: Slice<Range> + Offset + AsBytes + Slice<RangeTo<usize>>,
    Value: Clone,
{
    fn slice(&self, range: Range) -> Self {
        let next_fragment = self.next_fragment.slice(range);
        let consumed_len = self.next_fragment.offset(&next_fragment);
        let next_offset = self.next_offset + consumed_len;

        Annotated {
            next_index: self.next_index,
            next_offset,
            next_fragment,
            data: HashMap::new(),
            tags: vec![],
            tree: self.tree.clone(),
            appendices: self.appendices.clone(),
            last_range: self.last_range,
        }
    }
}

pub fn with<Parse, Error, Output, Fragment>(
    key: &'static str,
    value: &'static str,
    mut parse: Parse,
) -> impl FnMut(Annotated<Fragment>) -> IResult<Annotated<Fragment>, Output, Error>
where
    Parse: Parser<Annotated<Fragment>, Output, Error>,
    Error: ParseError<Annotated<Fragment>>,
{
    move |input: Annotated<Fragment>| {
        let (span, out) = parse.parse(input)?;
        Ok((span.with(key, value), out))
    }
}

pub enum P<Output> {
    Ann(Ann<Output>),
    Xxx,
}

pub fn xxx<Parse, Error, Output, Fragment>(
    mut parse: Parse,
    p: P<Output>,
) -> impl FnMut(Annotated<Fragment>) -> IResult<Annotated<Fragment>, Output, Error>
where
    Parse: Parser<Annotated<Fragment>, Output, Error>,
    Error: ParseError<Annotated<Fragment>>,
{
    move |input: Annotated<Fragment>| parse.parse(input)
}

pub fn p<Parse, Error, Output, Fragment>(
    mut parse: Parse,
    ann: Ann<Output>,
) -> impl FnMut(Annotated<Fragment>) -> IResult<Annotated<Fragment>, Output, Error>
where
    Parse: Parser<Annotated<Fragment>, Output, Error>,
    Error: ParseError<Annotated<Fragment>>,
{
    move |mut input: Annotated<Fragment>| {
        let from = input.next_offset;
        let index = input.next_index;
        let mut next_tree = input.tree;
        input.tree = vec![];
        let (span, out) = parse.parse(input)?;
        let to = span.next_offset;

        // If the tree returned by parser does not have any new items,
        // we are in the leaf situation (parser did not produce any new branches).
        let node = if span.tree.is_empty() {
            Tree::Leaf(Leaf::Real(RealLeaf {
                path: vec![],
                location: LeafLocation { from, to, index },
                information: Information {
                    annotation: ann.label.clone(),
                    data: span.data,
                    tags: ann.tags.iter().filter_map(|t| t.resolve(&out)).collect(),
                    value: ann.value.resolve(&out),
                    doc: ann.doc.clone(),
                    splain: ann.splain.resolve(&out),
                },
            }))
        } else {
            Tree::Group {
                path: vec![],
                location: GroupLocation {
                    byte_from: from,
                    byte_to: to,
                    index_from: index,
                    index_to: span.next_index - 1, // inclusive
                },
                information: Information {
                    annotation: ann.label.clone(),
                    data: span.data,
                    tags: span.tags,
                    value: ann.value.resolve(&out),
                    doc: ann.doc.clone(),
                    splain: ann.splain.resolve(&out),
                },
                children: span.tree,
            }
        };

        // Only leaves advance index, groups are only logical collections of
        // leaves, they do not represent anything in the raw data.
        let next_index = if matches!(node, Tree::Leaf(_)) {
            span.next_index + 1
        } else {
            span.next_index
        };

        next_tree.push(node);

        let next_span = Annotated {
            next_index,
            next_offset: span.next_offset,
            next_fragment: span.next_fragment,
            data: HashMap::new(),
            tags: vec![],
            tree: next_tree,
            appendices: span.appendices,
            last_range: Some((from, to)),
        };
        Ok((next_span, out))
    }
}

pub fn alt<Parse, AltParse, Error, Output, AltOutput, Fragment: Clone>(
    mut parse: Parse,
    mut alt_parse: AltParse,
) -> impl FnMut(Annotated<Fragment>) -> IResult<Annotated<Fragment>, (Output, AltOutput), Error>
where
    Parse: Parser<Annotated<Fragment>, Output, Error>,
    AltParse: Parser<Annotated<Fragment>, AltOutput, Error>,
    Error: ParseError<Annotated<Fragment>>,
{
    move |input: Annotated<Fragment>| {
        let i = input.clone();
        let (_, alt_out) = alt_parse.parse(i)?;
        let (s, out) = parse.parse(input)?;
        Ok((s, (out, alt_out)))
    }
}

#[cfg(test)]
mod tests {
    use nom::combinator::success;
    use nom::number::complete::{be_u16, be_u8};

    use super::{p, Annotated};

    type TestResult = Result<(), nom::Err<nom::error::Error<Annotated<&'static [u8], String>>>>;

    #[test]
    fn test_ann_flat() -> TestResult {
        let span: Annotated<&[u8], String> = Annotated::new(&[100, 50, 0]);

        assert_eq!(span.next_fragment, &[100, 50, 0]);
        assert_eq!(span.next_offset, 0);
        assert_eq!(span.next_index, 0);
        assert_eq!(span.last_range, None);
        assert!(span.appendices.borrow().is_empty());

        let (span, value) = p("One byte", |s| s.to_string(), be_u8)(span)?;

        let first = {
            let sp = span.intervals.borrow();
            let sp = sp.overlapping(&(..)).collect::<Vec<_>>();
            assert_eq!(sp.len(), 1);

            let first = sp.get(0).unwrap();
            assert_eq!(first.sublist().count(), 0);

            let data = first.value;

            assert_eq!(data.annotation, "One byte");
            assert_eq!(data.value, "100".to_string());
            assert_eq!(data.from, 0);
            assert_eq!(data.to, 1);
            assert_eq!(data.index, 0);

            data.clone()
        };

        assert_eq!(value, 100);
        assert_eq!(span.next_fragment, &[50, 0]);
        assert_eq!(span.next_offset, 1);
        assert_eq!(span.next_index, 1);
        assert_eq!(span.last_range, Some((0, 1)));
        assert!(span.appendices.borrow().is_empty());

        let (span, value) = p("Two bytes", |s| s.to_string(), be_u16)(span)?;

        {
            let sp = span.intervals.borrow();
            let sp = sp.overlapping(&(..)).collect::<Vec<_>>();
            assert_eq!(sp.len(), 2);

            let first2 = sp.get(0).unwrap().value;
            assert_eq!(first2, &first);

            let second = sp.get(1).unwrap();
            assert_eq!(second.sublist().count(), 0);

            let data = second.value;

            assert_eq!(data.annotation, "Two bytes");
            assert_eq!(data.value, "12800".to_string());
            assert_eq!(data.from, 1);
            assert_eq!(data.to, 3);
            assert_eq!(data.index, 1);
        };

        assert_eq!(value, 12800);
        assert_eq!(span.next_fragment, &[]);
        assert_eq!(span.next_offset, 3);
        assert_eq!(span.next_index, 2);
        assert_eq!(span.last_range, Some((1, 3)));
        assert!(span.appendices.borrow().is_empty());

        Ok(())
    }

    #[test]
    fn test_ann_success() -> TestResult {
        let span: Annotated<&[u8], String> = Annotated::new(&[100, 50, 0]);

        // assert_eq!(span.next_fragment, &[100, 50, 0]);
        // assert_eq!(span.next_offset, 0);
        // assert_eq!(span.next_index, 0);
        // assert_eq!(span.last_range, None);
        // assert!(span.appendices.borrow().is_empty());
        // assert!(span.intervals.borrow().is_empty());

        let (span, value) = p("One byte", |s| s.to_string(), be_u8)(span)?;
        println!("1 #### {:#?}", span);

        let first = {
            let sp = span.intervals.borrow();
            let sp = sp.overlapping(&(..)).collect::<Vec<_>>();
            // assert_eq!(sp.len(), 1);

            let first = sp.get(0).unwrap();
            // assert_eq!(first.sublist().count(), 0);

            let data = first.value;

            // assert_eq!(data.annotation, "One byte");
            // assert_eq!(data.value, "100".to_string());
            // assert_eq!(data.from, 0);
            // assert_eq!(data.to, 1);
            // assert_eq!(data.index, 0);

            data.clone()
        };

        // assert_eq!(value, 100);
        // assert_eq!(span.next_fragment, &[50, 0]);
        // assert_eq!(span.next_offset, 1);
        // assert_eq!(span.next_index, 1);
        // assert_eq!(span.last_range, Some((0, 1)));
        // assert!(span.appendices.borrow().is_empty());

        let (span, value) = p("Zero byte", |s| s.to_string(), success(123u16))(span)?;
        println!("2 #### {:#?}", span);

        {
            let sp = span.intervals.borrow();
            let sp = sp.overlapping(&(..)).collect::<Vec<_>>();
            // assert_eq!(sp.len(), 2);

            let first2 = sp.get(0).unwrap().value;
            // assert_eq!(first2, &first);

            // let second = sp.get(1).unwrap();
            // assert_eq!(second.sublist().count(), 0);

            // let data = second.value;

            // assert_eq!(data.annotation, "Two bytes");
            // assert_eq!(data.value, "12800".to_string());
            // assert_eq!(data.from, 1);
            // assert_eq!(data.to, 3);
            // assert_eq!(data.index, 1);
        };

        // assert_eq!(value, 12800);
        // assert_eq!(span.next_fragment, &[]);
        // assert_eq!(span.next_offset, 3);
        // assert_eq!(span.next_index, 2);
        // assert_eq!(span.last_range, Some((1, 3)));
        // assert!(span.appendices.borrow().is_empty());

        let (span, value) = p("Two bytes", |s| s.to_string(), be_u16)(span)?;
        println!("3 #### {:#?}", span);

        {
            let sp = span.intervals.borrow();
            let sp = sp.overlapping(&(..)).collect::<Vec<_>>();
            // assert_eq!(sp.len(), 2);

            let first2 = sp.get(0).unwrap().value;
            // assert_eq!(first2, &first);

            let second = sp.get(1).unwrap();
            // assert_eq!(second.sublist().count(), 0);

            let data = second.value;

            // assert_eq!(data.annotation, "Two bytes");
            // assert_eq!(data.value, "12800".to_string());
            // assert_eq!(data.from, 1);
            // assert_eq!(data.to, 3);
            // assert_eq!(data.index, 1);
        };

        // assert_eq!(value, 12800);
        // assert_eq!(span.next_fragment, &[]);
        // assert_eq!(span.next_offset, 3);
        // assert_eq!(span.next_index, 2);
        // assert_eq!(span.last_range, Some((1, 3)));
        // assert!(span.appendices.borrow().is_empty());

        Ok(())
    }
}
