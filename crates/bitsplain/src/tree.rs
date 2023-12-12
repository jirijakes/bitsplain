//! Hierarchical structure of [`Values`](crate::value) that is built
//! during parsing of the binary input.

use std::collections::HashMap;
use std::ops::{Deref, Range};

use crate::dsl::Reference;
use crate::value::Value;

/// Node in the [`Tree`] of [`Values`](crate::value).
#[derive(Debug, Clone)]
pub enum Node {
    Group {
        /// Path to this group.
        path: Vec<String>,
        /// Location of the group.
        location: GroupLocation,
        /// Group's information.
        information: Information,
        /// Children.
        children: Vec<Node>,
    },
    Leaf(Leaf),
}

impl Node {
    pub fn index_range(&self) -> (usize, usize) {
        match self {
            // Tree::Virtual { .. } => todo!(), //(500, 500),
            Node::Group { children, .. } => children
                .iter()
                // .filter(|t| !matches!(t, Tree::Virtual { .. }))
                .fold((usize::MAX, usize::MIN), |(min_from, max_to), ch| {
                    let (from, to) = ch.index_range();
                    (min_from.min(from), max_to.max(to))
                }),
            Node::Leaf(Leaf::Real(RealLeaf { location, .. })) => (location.index, location.index),
            _ => (usize::MAX, usize::MIN),
        }
    }

    pub fn information(&self) -> &Information {
        match self {
            Node::Group { information, .. } => information,
            Node::Leaf(Leaf::Real(l)) => &l.information,
            Node::Leaf(Leaf::Virtual(l)) => &l.information,
        }
    }
}

/// Range of bytes in the binary input that is further
/// indivisible (i. e. leaf of tree).
///
/// The location is exclusive in the upper bound (`to`), i. e.
/// the number of bytes in the range is `from - to`.
#[derive(Debug, Clone)]
pub struct LeafLocation {
    /// Offset of the first byte of the leaf.
    pub from: usize,

    /// Exclusive offset of the last byte of the leaf.
    pub to: usize,

    /// Ordinal index of this leaf.
    pub index: usize,
}

impl LeafLocation {
    pub fn range(&self) -> Range<usize> {
        self.from..self.to
    }
}

/// Range of bytes in the binary input that is further divided,
/// i. e. may contains leaves or other groups.
///
/// The upper bounds are exclusive.
#[derive(Debug, Clone)]
pub struct GroupLocation {
    /// Offset of the first byte of the group.
    pub byte_from: usize,

    /// Exclusive offset of the last byte of the group.
    pub byte_to: usize,

    /// Ordinal index of the first byte of the group.
    pub index_from: usize,

    /// Ordinal index of the last byte of the group.
    pub index_to: usize,
}

/// Details about leaf or group.
#[derive(Debug, Clone)]
pub struct Information {
    /// Label of the leaf or group.
    pub label: String,

    /// Auxiliary data attached to the leaf or group.
    pub data: HashMap<&'static str, String>,

    /// Tags attached to the leaf or group.
    pub tags: Vec<Tag>,

    pub refs: Vec<Reference>,

    /// Value of the leaf or group.
    pub value: Value,

    /// Documentation string.
    pub doc: Option<String>,

    /// Splain string.
    pub splain: Option<String>,
}

impl Information {
    pub fn has_data(&self, key: &'static str, value: &str) -> bool {
        match self.data.get(key) {
            Some(v) => v == value,
            _ => false,
        }
    }
}

/// Tag attached to leaf or group.
#[derive(Debug, Clone)]
pub struct Tag {
    pub label: String,
    pub color: Option<String>,
    pub doc: Option<String>,
}

/// Leaf that is not directly represented in binary input. Its value is
/// calculated from other available data.
#[derive(Debug, Clone)]
pub struct VirtualLeaf {
    /// Path to this leaf.
    pub path: Vec<String>,

    /// The leaf's information.
    pub information: Information,
}

/// Leaf that is represented in binary input. Its value is interpretation
/// of the input.
#[derive(Debug, Clone)]
pub struct RealLeaf {
    /// Path to this leaf.
    pub path: Vec<String>,

    /// Location of this leaf.
    pub location: LeafLocation,

    /// The leaf's information.
    pub information: Information,
}

/// A leaf in the tree.
#[derive(Debug, Clone)]
pub enum Leaf {
    /// Real leaf, represented in binary input.
    Real(RealLeaf),

    /// Virtual leaf, not represented in binary input.
    Virtual(VirtualLeaf),
}

impl Leaf {
    /// Returns information of this leaf.
    pub fn information(&self) -> &Information {
        match self {
            Leaf::Real(l) => &l.information,
            Leaf::Virtual(l) => &l.information,
        }
    }

    /// Returns path of this leaf.
    pub fn path(&self) -> &[String] {
        match self {
            Leaf::Real(l) => &l.path,
            Leaf::Virtual(l) => &l.path,
        }
    }

    /// Returns length of this leaf, if present.
    pub fn length(&self) -> Option<usize> {
        match self {
            Leaf::Real(l) => Some(l.location.to - l.location.from),
            _ => None,
        }
    }

    /// Returns index of this leaf, if present.
    pub fn index(&self) -> Option<usize> {
        match self {
            Leaf::Real(l) => Some(l.location.index),
            _ => None,
        }
    }
}

/// Tree of annotations.
#[derive(Debug)]
pub struct Tree(Vec<Node>);

impl Tree {
    #[inline]
    pub fn from_nodes(trees: Vec<Node>) -> Tree {
        Tree(trees)
    }

    pub fn leaves(&self) -> Vec<&Leaf> {
        Self::tree_leaves(&self.0)
    }

    pub fn real_leaves(&self) -> Vec<&RealLeaf> {
        Self::tree_leaves(&self.0)
            .iter()
            .filter_map(|l| match l {
                Leaf::Real(r) => Some(r),
                _ => None,
            })
            .collect()
    }

    fn tree_leaves(trees: &[Node]) -> Vec<&Leaf> {
        trees
            .iter()
            .flat_map(|tree| match tree {
                Node::Group { children, .. } => Self::tree_leaves(children),
                Node::Leaf(leaf) => vec![leaf],
            })
            .collect()
    }

    pub fn select<'a>(&'a self, path: &'a [String]) -> Option<&'a Node> {
        Self::select_path(&self.0, path)
    }

    fn select_path<'a>(tree: &'a [Node], path: &'a [String]) -> Option<&'a Node> {
        let (head, tail) = path.split_first()?;
        let i = head.parse::<usize>().ok()?;
        // TODO: Find more efficient solution
        let subtree = tree
            .iter()
            // .filter(|t| !matches!(t, Tree::Virtual { .. }))
            .collect::<Vec<_>>();
        let subtree = subtree.get(i)?;
        if tail.is_empty() {
            Some(subtree)
        } else {
            match subtree {
                Node::Group { children, .. } => Self::select_path(children, tail),
                _ => None,
                // Tree::Virtual { .. } => None,
            }
        }
    }
}

impl Deref for Tree {
    type Target = [Node];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
