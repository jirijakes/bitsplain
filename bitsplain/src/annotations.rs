use crate::lines::Lines;
use crate::tree::*;

#[derive(Debug)]
pub struct Annotations(pub Vec<Tree>);

impl Annotations {
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
