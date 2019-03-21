use super::{node::Node, MPTData};

use std::iter::IntoIterator;

#[derive(Clone, PartialEq, Debug)]
pub struct NodeUpdates<T: MPTData> {
    root: Node<T>,
    branches: Vec<Node<T>>,
}

impl<T: MPTData> IntoIterator for NodeUpdates<T> {
    type Item = Node<T>;
    type IntoIter = ::std::vec::IntoIter<Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        let root = self.root;
        let mut vec = self.branches;
        vec.push(root);
        vec.into_iter()
    }
}

impl<T: MPTData> NodeUpdates<T> {
    pub fn new(root: Node<T>, branches: Vec<Node<T>>) -> Self {
        NodeUpdates {
            root,
            branches,
        }
    }

    pub fn get_root_hash(&self) -> u64 {
        self.root.get_hash()
    }
}

#[cfg(test)]
use super::node;

#[cfg(test)]
mod tests {
    use super::*;
    use self::node::PointerNode;

    #[test]
    fn test_mpt_node_updates() {
        let root: Node<u64> = Node::BranchNode(PointerNode::default());

        let mut branch0 = root.clone();
        if let Node::BranchNode(ref mut ptr) = branch0 {
            ptr.set_hash(0, 0);
        }
        let mut first_root = root.clone();
        if let Node::BranchNode(ref mut ptr) = first_root {
            ptr.set_hash(0, branch0.get_hash());
        }

        let updates = NodeUpdates::new(first_root, vec![branch0]);

        let mut branch1 = root.clone();
        if let Node::BranchNode(ref mut ptr) = branch1 {
            ptr.set_hash(0, 1);
        }

        let mut branch2 = root.clone();
        if let Node::BranchNode(ref mut ptr) = branch2 {
            ptr.set_hash(0, 2);
        }

        let mut new_root = root.clone();
        if let Node::BranchNode(ref mut ptr) = new_root {
            ptr.set_hash(0, branch1.get_hash());
            ptr.set_hash(1, branch2.get_hash());
        }

        let new_updates = NodeUpdates::new(new_root.clone(), vec![branch1, branch2]);

        assert_eq!(new_updates.clone().into_iter().collect::<Vec<Node<u64>>>().len(), 3);
        assert_eq!(updates.clone().into_iter().collect::<Vec<Node<u64>>>().len(), 2);
    }
}