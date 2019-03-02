use std::collections::HashMap;
use std::hash::{Hash,Hasher};
use std::iter::IntoIterator;
use std::fmt::Debug;

use security::hash::hasher::Sha3Hasher;

#[inline]
fn get_top_nibble(val: u64) -> u8 {
    ((val & 0xF000_0000_0000_0000) >> 60) as u8
}

#[inline]
fn get_bottom_nibble(val: u64) -> u8 {
    (val & 0x0000_0000_0000_000F) as u8
}

#[derive(Clone, Hash, PartialEq, Debug)]
pub struct PointerNode {
    x_0: Option<u64>,
    x_1: Option<u64>,
    x_2: Option<u64>,
    x_3: Option<u64>,
    x_4: Option<u64>,
    x_5: Option<u64>,
    x_6: Option<u64>,
    x_7: Option<u64>,
    x_8: Option<u64>,
    x_9: Option<u64>,
    x_a: Option<u64>,
    x_b: Option<u64>,
    x_c: Option<u64>,
    x_d: Option<u64>,
    x_e: Option<u64>,
    x_f: Option<u64>
}

impl Default for PointerNode {
    fn default() -> Self {
        PointerNode {
            x_0: None,
            x_1: None,
            x_2: None,
            x_3: None,
            x_4: None,
            x_5: None,
            x_6: None,
            x_7: None,
            x_8: None,
            x_9: None,
            x_a: None,
            x_b: None,
            x_c: None,
            x_d: None,
            x_e: None,
            x_f: None,
        }
    }
}

impl PointerNode {
    fn get_next_hash(&self, k: u64) -> Option<u64> {
        self.get(get_top_nibble(k))
    }

    fn get(&self, index: u8) -> Option<u64> {
        match index {
            0x0 => self.x_0,
            0x1 => self.x_1,
            0x2 => self.x_2,
            0x3 => self.x_3,
            0x4 => self.x_4,
            0x5 => self.x_5,
            0x6 => self.x_6,
            0x7 => self.x_7,
            0x8 => self.x_8,
            0x9 => self.x_9,
            0xA => self.x_a,
            0xB => self.x_b,
            0xC => self.x_c,
            0xD => self.x_d,
            0xE => self.x_e,
            0xF => self.x_f,
            _ => panic!("Invalid Hex Bit?"),
        }
    }

    fn set_hash(&mut self, k: u8, v: u64) {
        match k {
            0x0 => self.x_0 = Some(v),
            0x1 => self.x_1 = Some(v),
            0x2 => self.x_2 = Some(v),
            0x3 => self.x_3 = Some(v),
            0x4 => self.x_4 = Some(v),
            0x5 => self.x_5 = Some(v),
            0x6 => self.x_6 = Some(v),
            0x7 => self.x_7 = Some(v),
            0x8 => self.x_8 = Some(v),
            0x9 => self.x_9 = Some(v),
            0xA => self.x_a = Some(v),
            0xB => self.x_b = Some(v),
            0xC => self.x_c = Some(v),
            0xD => self.x_d = Some(v),
            0xE => self.x_e = Some(v),
            0xF => self.x_f = Some(v),
            _ => panic!("Invalid Hex Bit?"),
        }
    }
}

struct PointerNodeIterator<'a> {
    index: u8,
    node: &'a PointerNode
}

impl<'a> PointerNodeIterator<'a> {
    fn new(node: &'a PointerNode) -> Self {
        PointerNodeIterator {
            index: 0,
            node: node
        }
    }
}

impl<'a> Iterator for PointerNodeIterator<'a> {
    type Item = Option<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 16 {
            return None;
        }

        let res = self.node.get(self.index);
        self.index += 1;

        Some(res)
    }
}

#[derive(Clone, Hash, PartialEq, Debug)]
pub enum Node<T: Hash + PartialEq + Clone + Debug> {
    BranchNode(PointerNode),
    LeafNode(T)
}

impl<T: Hash + PartialEq + Clone + Debug> Node<T> {
    pub fn get_hash(&self) -> u64 {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

#[derive(PartialEq, Debug)]
pub struct NodeUpdates<T: Hash + PartialEq + Clone + Debug> {
    root: Node<T>,
    branches: Vec<Node<T>>,
}

impl<T: Hash + PartialEq + Clone + Debug> IntoIterator for NodeUpdates<T> {
    type Item = Node<T>;
    type IntoIter = ::std::vec::IntoIter<Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        let root = self.root;
        let mut vec = self.branches;
        vec.push(root);
        vec.into_iter()
    }
}

impl<T: Hash + PartialEq + Clone + Debug> NodeUpdates<T> {
    fn new(root: Node<T>, branches: Vec<Node<T>>) -> Self {
        NodeUpdates {
            root,
            branches,
        }
    }
}

pub struct MerklePatriciaTree<T: Hash + PartialEq + Clone + Debug> {
    nodes: HashMap<u64, Node<T>>,
}

impl<T: Hash + PartialEq + Clone + Debug> MerklePatriciaTree<T> {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        let root = Node::BranchNode(PointerNode::default());
        let hash = root.get_hash();
        nodes.insert(hash, root);
        MerklePatriciaTree {
            nodes: nodes,
        }
    }

    pub fn default_root() -> u64 {
        Node::BranchNode::<T>(PointerNode::default()).get_hash()
    }

    pub fn get(&self, root: u64, mut k: u64) -> Option<&T> {
        let mut node = self.nodes.get(&root);
        // 16 branch nodes + 1 leaf node
        for _ in 0..17 {
            match node {
                Some(Node::BranchNode(pointers)) => {
                    if let Some(hash) = pointers.get_next_hash(k) {
                        node = self.nodes.get(&hash);
                    }
                    else {
                        return None;
                    }
                },
                Some(Node::LeafNode(value)) => {
                    return Some(value);
                }
                _ => panic!("get: Malformed MerklePatriciaTree node(s)"),
            }
            k <<= 4;
        }
        None
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
                    pointers.set_hash(get_bottom_nibble(key), hash);
                }
                hash = node.get_hash();
                key >>= 4;
            }
            new_nodes.push(leaf_node);
            let mut new_root = root_node.clone();
            if let Node::BranchNode(ref mut pointers) = new_root {
                pointers.set_hash(get_bottom_nibble(key), hash);
            }
            NodeUpdates::new(new_root, new_nodes)
        }
    }

    pub fn commit_set(&mut self, updates: NodeUpdates<T>) {
        for node in updates.into_iter() {
            self.nodes.insert(node.get_hash(), node);
        }
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

            let a_iter = PointerNodeIterator::new(a_pointers);
            let b_iter = PointerNodeIterator::new(b_pointers);
            let ref_iter = PointerNodeIterator::new(ref_pointers);

            for ((i, ref_ptr), (a_ptr, b_ptr)) in ref_iter.enumerate().zip(a_iter.zip(b_iter)) {
                if a_ptr != b_ptr {
                    match (a_ptr, b_ptr, ref_ptr) {
                        (Some(a), Some(b), Some(r)) => {
                            // Recurse, checking valid merge for child
                            let mut res = self.try_merge(a, b, r);
                            if let Some(child_updates) = res {
                                // Insert child data into new_ptr and new_nodes
                                new_ptr.set_hash(i as u8, child_updates.root.get_hash());
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

    #[test]
    fn test_get_top_nibble() {
        assert_eq!(0x0, get_top_nibble(0x0000_0000_0000_0000));
        assert_eq!(0x1, get_top_nibble(0x1000_0000_0000_0000));
        assert_eq!(0x2, get_top_nibble(0x2000_0000_0000_0000));
        assert_eq!(0x3, get_top_nibble(0x3000_0000_0000_0000));
        assert_eq!(0x4, get_top_nibble(0x4000_0000_0000_0000));
        assert_eq!(0x5, get_top_nibble(0x5000_0000_0000_0000));
        assert_eq!(0x6, get_top_nibble(0x6000_0000_0000_0000));
        assert_eq!(0x7, get_top_nibble(0x7000_0000_0000_0000));
        assert_eq!(0x8, get_top_nibble(0x8000_0000_0000_0000));
        assert_eq!(0x9, get_top_nibble(0x9000_0000_0000_0000));
        assert_eq!(0xA, get_top_nibble(0xA000_0000_0000_0000));
        assert_eq!(0xB, get_top_nibble(0xB000_0000_0000_0000));
        assert_eq!(0xC, get_top_nibble(0xC000_0000_0000_0000));
        assert_eq!(0xD, get_top_nibble(0xD000_0000_0000_0000));
        assert_eq!(0xE, get_top_nibble(0xE000_0000_0000_0000));
        assert_eq!(0xF, get_top_nibble(0xF000_0000_0000_0000));

        assert_eq!(0xF, get_top_nibble(0xFE00_0000_0000_0000));
        assert_eq!(0xF, get_top_nibble(0xF000_E000_0000_0000));
        assert_eq!(0xF, get_top_nibble(0xF000_0000_0000_0007));
        assert_eq!(0xF, get_top_nibble(0xFFFF_FFFF_FFFF_FFFF));
    }

    #[test]
    fn test_get_bottom_nibble() {
        assert_eq!(0x0, get_bottom_nibble(0x0000_0000_0000_0000));
        assert_eq!(0x1, get_bottom_nibble(0x0000_0000_0000_0001));
        assert_eq!(0x2, get_bottom_nibble(0x0000_0000_0000_0002));
        assert_eq!(0x3, get_bottom_nibble(0x0000_0000_0000_0003));
        assert_eq!(0x4, get_bottom_nibble(0x0000_0000_0000_0004));
        assert_eq!(0x5, get_bottom_nibble(0x0000_0000_0000_0005));
        assert_eq!(0x6, get_bottom_nibble(0x0000_0000_0000_0006));
        assert_eq!(0x7, get_bottom_nibble(0x0000_0000_0000_0007));
        assert_eq!(0x8, get_bottom_nibble(0x0000_0000_0000_0008));
        assert_eq!(0x9, get_bottom_nibble(0x0000_0000_0000_0009));
        assert_eq!(0xA, get_bottom_nibble(0x0000_0000_0000_000A));
        assert_eq!(0xB, get_bottom_nibble(0x0000_0000_0000_000B));
        assert_eq!(0xC, get_bottom_nibble(0x0000_0000_0000_000C));
        assert_eq!(0xD, get_bottom_nibble(0x0000_0000_0000_000D));
        assert_eq!(0xE, get_bottom_nibble(0x0000_0000_0000_000E));
        assert_eq!(0xF, get_bottom_nibble(0x0000_0000_0000_000F));

        assert_eq!(0xF, get_bottom_nibble(0x0000_0000_0000_00EF));
        assert_eq!(0xF, get_bottom_nibble(0x0000_0000_000E_000F));
        assert_eq!(0xF, get_bottom_nibble(0xE000_0000_0000_000F));
        assert_eq!(0xF, get_bottom_nibble(0xFFFF_FFFF_FFFF_FFFF));
    }

    #[test]
    fn test_default_root() {
        let mpt: MerklePatriciaTree<u64> = MerklePatriciaTree::new();
        assert_eq!(*mpt.nodes.iter().next().unwrap().0,
            MerklePatriciaTree::<u64>::default_root());
    }

    #[test]
    fn test_mpt_get_empty() {
        let mpt: MerklePatriciaTree<u64> = MerklePatriciaTree::new();
        assert_eq!(mpt.get(MerklePatriciaTree::<u64>::default_root(), 0), None);
    }

    fn mpt_set<T: Hash + PartialEq + Clone + Debug>(mpt: &mut MerklePatriciaTree<T>, root: u64, k: u64, v: T) -> u64 {
        let updates = {
            mpt.try_set(root, k, v)
        };
        let new_root = updates.root.get_hash();
        mpt.commit_set(updates);
        new_root
    }

    #[test]
    fn test_mpt_get_set() {
        let mut mpt: MerklePatriciaTree<u64> = MerklePatriciaTree::new();
        let mut root = MerklePatriciaTree::<u64>::default_root();
        root = mpt_set(&mut mpt, root, 0x1234_5678_9ABC_DEF0, 0);
        assert_eq!(mpt.get(root, 0x1234_5678_9ABC_DEF0), Some(&0));

        root = mpt_set(&mut mpt, root, 0, 0);
        assert_eq!(mpt.get(root, 0), Some(&0));
        root = mpt_set(&mut mpt, root, 1, 0);
        assert_eq!(mpt.get(root, 1), Some(&0));
        root = mpt_set(&mut mpt, root, 2, 100);
        assert_eq!(mpt.get(root, 2), Some(&100));
        root = mpt_set(&mut mpt, root, u64::max_value(), u64::max_value());
        assert_eq!(mpt.get(root, u64::max_value()), Some(&u64::max_value()));
        for i in 8..64 {
            root = mpt_set(&mut mpt, root, i, i);
        }
        for i in 8..64 {
            assert_eq!(mpt.get(root, i), Some(&i));
        }
    }

    #[test]
    fn test_mpt_merge() {
        let mut mpt: MerklePatriciaTree<u64> = MerklePatriciaTree::new();
        let mut root = MerklePatriciaTree::<u64>::default_root();
        root = mpt_set(&mut mpt, root, 0, 0);
        assert_eq!(mpt.get(root, 0), Some(&0));

        let mut root_a;
        let mut root_b;

        // Invalid merge, key 1 modified in both
        root_a = mpt_set(&mut mpt, root, 1, 1);
        root_b = mpt_set(&mut mpt, root, 1, 2);

        assert_eq!(mpt.try_merge(root_a, root_b, root), None);

        // Valid merges, different keys
        root_a = mpt_set(&mut mpt, root, 1, 1);
        for i in 2..128 {
            root_b = mpt_set(&mut mpt, root, i, i);

            let updates = mpt.try_merge(root_a, root_b, root).unwrap();
            let new_root = updates.root.get_hash();
            mpt.commit_set(updates);
            assert_eq!(mpt.get(new_root, i), Some(&i));
        }
    }
}