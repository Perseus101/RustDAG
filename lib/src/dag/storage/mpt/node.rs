use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use serde::de::{Deserialize, Deserializer, EnumAccess, VariantAccess, Visitor};
use serde::ser::{Serialize, Serializer};

use security::hash::hasher::Sha3Hasher;

use super::MPTData;

#[inline]
fn get_top_nibble(val: u64) -> u8 {
    ((val & 0xF000_0000_0000_0000) >> 60) as u8
}

#[inline]
fn get_bottom_nibble(val: u64) -> u8 {
    (val & 0x0000_0000_0000_000F) as u8
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Debug)]
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
    x_f: Option<u64>,
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
    pub fn get_next_hash(&self, k: u64) -> Option<u64> {
        self.get(get_top_nibble(k))
    }

    pub fn get(&self, index: u8) -> Option<u64> {
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

    pub fn set_hash(&mut self, k: u8, v: u64) {
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

    pub fn set_from(&mut self, key: u64, v: u64) {
        self.set_hash(get_bottom_nibble(key), v);
    }

    pub fn iter(&self) -> PointerNodeIterator {
        PointerNodeIterator::new(self)
    }
}

pub struct PointerNodeIterator<'a> {
    index: u8,
    node: &'a PointerNode,
}

impl<'a> PointerNodeIterator<'a> {
    fn new(node: &'a PointerNode) -> Self {
        PointerNodeIterator { index: 0, node }
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
#[allow(clippy::large_enum_variant)]
pub enum Node<T: MPTData> {
    BranchNode(PointerNode),
    LeafNode(T),
}

impl<T: MPTData> Node<T> {
    pub fn get_hash(&self) -> u64 {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl<T: MPTData + Serialize> Serialize for Node<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Node::BranchNode(ptr) => {
                serializer.serialize_newtype_variant("Node", 0, "BranchNode", ptr)
            }
            Node::LeafNode(value) => {
                serializer.serialize_newtype_variant("Node", 1, "LeafNode", value)
            }
        }
    }
}

impl<'de, T: 'de + MPTData + Deserialize<'de>> Deserialize<'de> for Node<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            BranchNode,
            LeafNode,
        }

        struct NodeVisitor<'de, T: 'de + MPTData + Deserialize<'de>> {
            _p: PhantomData<T>,
            _l: PhantomData<&'de ()>,
        };

        impl<'de, T: 'de + MPTData + Deserialize<'de>> Visitor<'de> for NodeVisitor<'de, T> {
            type Value = Node<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Node")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match data.variant()? {
                    (Field::BranchNode, variant) => {
                        Ok(Node::BranchNode(variant.newtype_variant()?))
                    }
                    (Field::LeafNode, variant) => Ok(Node::LeafNode(variant.newtype_variant()?)),
                }
            }
        }

        const VARIANTS: &[&str] = &["BranchNode", "LeafNode"];
        deserializer.deserialize_enum(
            "Node",
            VARIANTS,
            NodeVisitor {
                _p: PhantomData,
                _l: PhantomData,
            },
        )
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
    fn test_serialize() {
        let branch_node = Node::BranchNode::<u64>(PointerNode::default());
        let branch_json = json!({
            "BranchNode": {
                "x_0": null, "x_1": null, "x_2": null, "x_3": null,
                "x_4": null, "x_5": null, "x_6": null, "x_7": null,
                "x_8": null, "x_9": null, "x_a": null, "x_b": null,
                "x_c": null, "x_d": null, "x_e": null, "x_f": null
            }
        });
        assert_eq!(branch_json, serde_json::to_value(branch_node).unwrap());

        let leaf_node = Node::LeafNode::<u64>(5);
        let leaf_json = json!({ "LeafNode": 5 });
        assert_eq!(leaf_json, serde_json::to_value(leaf_node).unwrap());
    }

    #[test]
    fn test_deserialize() {
        let branch_node = Node::BranchNode::<u64>(PointerNode::default());
        let branch_json = json!({
            "BranchNode": {
                "x_0": null, "x_1": null, "x_2": null, "x_3": null,
                "x_4": null, "x_5": null, "x_6": null, "x_7": null,
                "x_8": null, "x_9": null, "x_a": null, "x_b": null,
                "x_c": null, "x_d": null, "x_e": null, "x_f": null
            }
        });
        assert_eq!(branch_node, serde_json::from_value(branch_json).unwrap());

        let leaf_node = Node::LeafNode::<u64>(5);
        let leaf_json = json!({ "LeafNode": 5 });
        assert_eq!(leaf_node, serde_json::from_value(leaf_json).unwrap());
    }

    #[test]
    fn test_serialize_deserialize() {
        // Check that the branch node is identical after serializing and deserializing
        let branch_node = Node::BranchNode::<u64>(PointerNode::default());
        let json_value = serde_json::to_value(branch_node.clone()).unwrap();
        assert_eq!(branch_node, serde_json::from_value(json_value).unwrap());

        // Check that a non-null branch node is identical as well
        let mut ptr = PointerNode::default();
        ptr.set_hash(0, 10);
        let branch_node = Node::BranchNode::<u64>(ptr);
        let json_value = serde_json::to_value(branch_node.clone()).unwrap();
        assert_eq!(branch_node, serde_json::from_value(json_value).unwrap());
    }
}
