use std::collections::HashMap;
use std::ops::Deref;

use crate::lines::Lines;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct LeafLocation {
    pub from: usize,
    pub to: usize,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct GroupLocation {
    pub byte_from: usize,
    pub byte_to: usize,
    pub index_from: usize,
    pub index_to: usize,
}

#[derive(Debug, Clone)]
pub struct Information {
    pub annotation: String,
    pub data: HashMap<&'static str, String>,
    pub tags: Vec<Tag>,
    pub value: Value,
    pub doc: Option<String>,
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

#[derive(Debug, Clone)]
pub struct Tag {
    pub label: String,
    pub color: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VirtualLeaf {
    pub path: Vec<String>,
    pub information: Information,
}

#[derive(Debug, Clone)]
pub struct RealLeaf {
    pub path: Vec<String>,
    pub location: LeafLocation,
    pub information: Information,
}

#[derive(Debug, Clone)]
pub enum Leaf {
    Real(RealLeaf),
    Virtual(VirtualLeaf),
}

#[derive(Debug, Clone)]
pub enum Tree {
    Group {
        path: Vec<String>,
        location: GroupLocation,
        information: Information,
        children: Vec<Tree>,
    },
    Leaf(Leaf),
}

impl Tree {
    pub fn index_range(&self) -> (usize, usize) {
        match self {
            // Tree::Virtual { .. } => todo!(), //(500, 500),
            Tree::Group { children, .. } => children
                .iter()
                // .filter(|t| !matches!(t, Tree::Virtual { .. }))
                .fold((usize::MAX, usize::MIN), |(min_from, max_to), ch| {
                    let (from, to) = ch.index_range();
                    (min_from.min(from), max_to.max(to))
                }),
            Tree::Leaf(Leaf::Real(RealLeaf { location, .. })) => (location.index, location.index),
            _ => (usize::MAX, usize::MIN),
        }
    }

    pub fn information(&self) -> &Information {
        match self {
            Tree::Group { information, .. } => information,
            Tree::Leaf(Leaf::Real(l)) => &l.information,
            Tree::Leaf(Leaf::Virtual(l)) => &l.information,
        }
    }
}

#[derive(Debug)]
pub struct Annotations(Vec<Tree>);

impl Annotations {
    #[inline]
    pub fn from_trees(trees: Vec<Tree>) -> Annotations {
        Annotations(trees)
    }

    pub fn leaves(&self) -> Vec<&RealLeaf> {
        Self::tree_leaves(&self.0)
    }

    pub fn lines<'a>(&'a self, bytes: &'a [u8]) -> Lines<'a> {
        Lines::new(self.leaves(), bytes)
    }

    fn tree_leaves(trees: &[Tree]) -> Vec<&RealLeaf> {
        trees
            .iter()
            .flat_map(|tree| match tree {
                Tree::Group { children, .. } => Self::tree_leaves(children),
                Tree::Leaf(Leaf::Real(leaf)) => vec![leaf],
                _ => vec![], // Tree::Virtual { .. } => vec![],
            })
            .collect()
    }

    pub fn select<'a>(&'a self, path: &'a [String]) -> Option<&'a Tree> {
        Self::select_path(&self.0, path)
    }

    fn select_path<'a>(tree: &'a [Tree], path: &'a [String]) -> Option<&'a Tree> {
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
                Tree::Group { children, .. } => Self::select_path(children, tail),
                _ => None,
                // Tree::Virtual { .. } => None,
            }
        }
    }
}

impl Deref for Annotations {
    type Target = [Tree];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
