use std::ops::{Index, IndexMut};

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub struct ContractState {
    raw: Vec<u8>,
}

impl ContractState {
    pub fn new(capacity: usize) -> Self {
        ContractState {
            raw: vec![0; capacity],
        }
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.raw
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.raw
    }
}

impl Index<usize> for ContractState {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.raw[index]
    }
}

impl IndexMut<usize> for ContractState {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.raw[index]
    }
}
