use std::collections::HashMap;

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
