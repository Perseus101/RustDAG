use std::collections::HashMap;

use dag::storage::map::{Map, MapResult, MapError};

use super::{MerklePatriciaTree, NodeUpdates, node::Node};
use super::mpt::{MPTStorageMap, MPTData};

/// Temporary map to store updates to a MerklePatriciaTree
pub struct MPTTempMap<'a, T: MPTData, M: MPTStorageMap<T>> {
    mpt: &'a MerklePatriciaTree<T, M>,
    new_nodes: HashMap<u64, Node<T>>
}

impl<'a, T: MPTData, M: MPTStorageMap<T>> MPTTempMap<'a, T, M> {
    pub fn new(mpt: &'a MerklePatriciaTree<T, M>) -> Self {
        MPTTempMap {
            mpt,
            new_nodes: HashMap::new()
        }
    }

    pub fn write_out(mut self, root: u64) -> MapResult<NodeUpdates<T>> {
        /// Move root and all its children from nodes_in to nodes out
        fn move_nodes<T: MPTData>(root: Node<T>, nodes_in: &mut HashMap<u64, Node<T>>,
                nodes_out: &mut Vec<Node<T>>) {
            move_nodes_recurse(&root, nodes_in, nodes_out);
            nodes_out.push(root);
        }

        /// Move the children of root from nodes_in to nodes out
        fn move_nodes_recurse<T: MPTData>(root: &Node<T>, nodes_in: &mut HashMap<u64, Node<T>>,
                nodes_out: &mut Vec<Node<T>>) {
            if let Node::BranchNode(root_ptr) = root {
                for opt_node_hash in root_ptr.iter() {
                    if let Some(node_hash) = opt_node_hash {
                        if let Some(node) = nodes_in.remove(&node_hash) {
                            move_nodes(node, nodes_in, nodes_out);
                        }
                    }
                }
            }
        }

        let mut branches = Vec::new();
        let root = self.new_nodes.remove(&root)
            .map_or(Err(MapError::NotFound), |node| { Ok(node) })?;

        move_nodes_recurse(&root, &mut self.new_nodes, &mut branches);

        Ok(NodeUpdates::new(root, branches))
    }
}

impl<'a, T: MPTData, M: MPTStorageMap<T>> Map<u64, Node<T>> for MPTTempMap<'a, T, M> {
    fn get(&self, k: &u64) -> MapResult<&Node<T>> {
        self.new_nodes.get(&k)
            .map_or(self.mpt.nodes.get(&k), |node| { Ok(node) })
    }

    fn set(&mut self, k: u64, v: Node<T>) -> MapResult<()> {
        self.new_nodes.insert(k, v);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpt_temp_map() {
        let mut mpt: MerklePatriciaTree<u64, _> = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();

        for i in 0..64 {
            root = mpt.set(root, i, i);
        }
        for i in 0..64 {
            assert_eq!(mpt.get(root, i), Ok(&i));
        }

        let temp_map = MPTTempMap::new(&mpt);
        let mut temp_mpt: MerklePatriciaTree<u64, _> = MerklePatriciaTree::new(temp_map);
        let mut temp_root = root.clone();
        for i in 0..64 {
            assert_eq!(temp_mpt.get(temp_root, i), Ok(&i));
        }

        for i in 64..128 {
            temp_root = temp_mpt.set(temp_root, i, i);
        }

        for i in 0..128 {
            assert_eq!(temp_mpt.get(temp_root, i), Ok(&i));
        }
    }
}