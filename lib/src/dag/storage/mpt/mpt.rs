use std::hash::Hash;
use std::fmt::Debug;
use std::marker::PhantomData;

use dag::storage::map::{Map, MapError};

use super::node::{Node, PointerNode};
use super::node_updates::NodeUpdates;

pub trait MPTStorageMap<T: MPTData> = Map<u64, Node<T>>;
pub trait MPTData = Hash + PartialEq + Clone + Debug;

pub struct MerklePatriciaTree<T: MPTData, M: MPTStorageMap<T>> {
    pub(super) nodes: M,
    phantom: PhantomData<T>,
}

impl<T: MPTData, M: MPTStorageMap<T> + Default> Default for MerklePatriciaTree<T, M> {
    fn default() -> Self {
        Self::new(M::default())
    }
}

impl<T: MPTData, M: MPTStorageMap<T>> MerklePatriciaTree<T, M> {

    #[allow(unused_must_use)]
    pub fn new(mut nodes: M) -> Self {
        let root = Node::BranchNode(PointerNode::default());
        let hash = root.get_hash();
        nodes.set(hash, root);
        MerklePatriciaTree {
            nodes,
            phantom: PhantomData
        }
    }

    pub fn default_root(&self) -> u64 {
        Node::BranchNode::<T>(PointerNode::default()).get_hash()
    }

    pub fn inner_map(self) -> M {
        self.nodes
    }

    pub fn get(&self, root: u64, mut k: u64) -> Result<&T, MapError> {
        let mut node = self.nodes.get(&root)?;
        // 16 branch nodes + 1 leaf node
        for _ in 0..17 {
            match node {
                Node::BranchNode(pointers) => {
                    let hash = pointers.get_next_hash(k)
                        .map_or(Err(MapError::NotFound), |hash| { Ok(hash) })?;
                    node = self.nodes.get(&hash)?;
                },
                Node::LeafNode(value) => {
                    return Ok(value);
                }
            }
            k <<= 4;
        }
        Err(MapError::LookupError)
    }

    pub fn try_set(&self, root: u64, k: u64, v: T) -> NodeUpdates<T> {
        let mut new_nodes = Vec::new();
        let root_node = self.nodes.get(&root)
            .expect("Root node does not exist");
        {
            let mut node = root_node;
            let mut key = k;
            let mut i = 1;
            // Insert clones of nodes on the path into new_nodes
            loop {
                if i == 16 {
                    break;
                }
                match node {
                    Node::BranchNode(pointers) => {
                        if let Some(hash) = pointers.get_next_hash(key) {
                            node = self.nodes.get(&hash).expect("Node does not exist");
                        }
                        else {
                            break;
                        }
                    },
                    _ => break,
                }
                new_nodes.push(node.clone());
                key <<= 4;
                i += 1;
            }
            // Create any missing nodes on the path
            loop {
                if i == 16 {
                    break;
                }
                new_nodes.push(Node::BranchNode(PointerNode::default()));
                key <<= 4;
                i += 1;
            }
        }

        // Update the hashes of all the nodes and return the resulting nodes
        {
            let mut key = k;
            let leaf_node = Node::LeafNode(v);
            let mut hash = leaf_node.get_hash();
            for node in new_nodes.iter_mut().rev() {
                if let Node::BranchNode(pointers) = node {
                    pointers.set_from(key, hash);
                }
                hash = node.get_hash();
                key >>= 4;
            }
            new_nodes.push(leaf_node);
            let mut new_root = root_node.clone();
            if let Node::BranchNode(ref mut pointers) = new_root {
                pointers.set_from(key, hash);
            }
            NodeUpdates::new(new_root, new_nodes)
        }
    }

    pub fn commit_set(&mut self, updates: NodeUpdates<T>) -> Result<(), MapError> {
        for node in updates.into_iter() {
            self.nodes.set(node.get_hash(), node)?;
        }
        Ok(())
    }

    pub fn set(&mut self, root: u64, k: u64, v: T) -> Result<u64, MapError> {
        let updates = {
            self.try_set(root, k, v)
        };
        let new_root = updates.get_root_hash();
        self.commit_set(updates)?;
        Ok(new_root)
    }

    pub fn try_merge(&self, hash_a: u64, hash_b: u64, hash_ref: u64)
            -> Option<NodeUpdates<T>> {
        if hash_a == hash_b {
            return Some(NodeUpdates::new(self.nodes.get(&hash_a)
                .expect("Root node does not exist").clone(), Vec::new()))
        }
        let root_a = self.nodes.get(&hash_a)
            .expect("Root node does not exist");
        let root_b = self.nodes.get(&hash_b)
            .expect("Root node does not exist");
        let root_ref = self.nodes.get(&hash_ref)
            .expect("Root node does not exist");

        if let (Node::LeafNode(a_val), Node::LeafNode(b_val),
                    Node::LeafNode(ref_val)) = (root_a, root_b, root_ref) {
            if a_val != ref_val && b_val != ref_val {
                // Invalid merge
                None
            }
            else if a_val != ref_val {
                Some(NodeUpdates::new(Node::LeafNode(a_val.clone()), Vec::new()))
            }
            else {
                Some(NodeUpdates::new(Node::LeafNode(b_val.clone()), Vec::new()))
            }
        }
        else if let (Node::BranchNode(a_pointers), Node::BranchNode(b_pointers),
                Node::BranchNode(ref_pointers)) = (root_a, root_b, root_ref) {
            let mut new_nodes = Vec::new();
            let mut new_ptr = ref_pointers.clone();

            let a_iter = a_pointers.iter();
            let b_iter = b_pointers.iter();
            let ref_iter = ref_pointers.iter();

            for ((i, ref_ptr), (a_ptr, b_ptr)) in ref_iter.enumerate().zip(a_iter.zip(b_iter)) {
                if a_ptr != b_ptr {
                    match (a_ptr, b_ptr, ref_ptr) {
                        (Some(a), Some(b), Some(r)) => {
                            // Recurse, checking valid merge for child
                            let mut res = self.try_merge(a, b, r);
                            if let Some(child_updates) = res {
                                // Insert child data into new_ptr and new_nodes
                                new_ptr.set_hash(i as u8, child_updates.get_root_hash());
                                new_nodes.extend(child_updates.into_iter());
                            }
                            else {
                                // The merge is invalid
                                return None;
                            }
                        },
                        (Some(_), Some(_), None) => {
                            // There is no way to know if a and b can be merged,
                            // so return invalid merge
                            return None;
                        },
                        (Some(child_ptr), None, None) |
                        (None, Some(child_ptr), None) => {
                            // Insert updated node
                            new_ptr.set_hash(i as u8, child_ptr);
                        },
                        (None, _, Some(_)) |
                        (_, None, Some(_)) => {
                            // This is a special invalid merge, because the
                            // chosen reference tree was incorrect
                            return None;
                        },
                        (None, None, _) => {
                            // This should be unreachable, since a_ptr and b_ptr
                            // are not equal
                            panic!("try_merge: a_ptr and b_ptr are unexpectedly equal");
                        }
                    }
                }
            }
            Some(NodeUpdates::new(Node::BranchNode(new_ptr), new_nodes))
        }
        else {
            // If we get here, one or more of the trees is malformed
            panic!("try_merge: Malformed MerklePatriciaTree node(s)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default_root() {
        let mpt: MerklePatriciaTree<u64, _> = MerklePatriciaTree::new(HashMap::new());
        assert_eq!(*mpt.nodes.iter().next().unwrap().0,
            mpt.default_root());
    }

    #[test]
    fn test_mpt_get_empty() {
        let mpt: MerklePatriciaTree<u64, _> = MerklePatriciaTree::new(HashMap::new());
        assert_eq!(mpt.get(mpt.default_root(), 0), Err(MapError::NotFound));
    }

    #[test]
    fn test_mpt_get_set() {
        let mut mpt: MerklePatriciaTree<u64, _> = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();
        root = mpt.set(root, 0x1234_5678_9ABC_DEF0, 0).unwrap();
        assert_eq!(mpt.get(root, 0x1234_5678_9ABC_DEF0), Ok(&0));

        root = mpt.set(root, 0, 0).unwrap();
        assert_eq!(mpt.get(root, 0), Ok(&0));
        root = mpt.set(root, 1, 0).unwrap();
        assert_eq!(mpt.get(root, 1), Ok(&0));
        root = mpt.set(root, 2, 100).unwrap();
        assert_eq!(mpt.get(root, 2), Ok(&100));
        root = mpt.set(root, u64::max_value(), u64::max_value()).unwrap();
        assert_eq!(mpt.get(root, u64::max_value()), Ok(&u64::max_value()));
        for i in 8..64 {
            root = mpt.set(root, i, i).unwrap();
        }
        for i in 8..64 {
            assert_eq!(mpt.get(root, i), Ok(&i));
        }
    }

    #[test]
    fn test_mpt_merge() {
        let mut mpt: MerklePatriciaTree<u64, _> = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();
        root = mpt.set(root, 0, 0).unwrap();
        assert_eq!(mpt.get(root, 0), Ok(&0));

        let mut root_a;
        let mut root_b;

        // Invalid merge, key 1 modified in both
        root_a = mpt.set(root, 1, 1).unwrap();
        root_b = mpt.set(root, 1, 2).unwrap();

        assert_eq!(mpt.try_merge(root_a, root_b, root), None);

        // Valid merges, different keys
        root_a = mpt.set(root, 1, 1).unwrap();
        for i in 2..128 {
            root_b = mpt.set(root, i, i).unwrap();

            let updates = mpt.try_merge(root_a, root_b, root).unwrap();
            let new_root = updates.get_root_hash();
            assert!(mpt.commit_set(updates).is_ok());
            assert_eq!(mpt.get(new_root, i), Ok(&i));
        }
    }
}