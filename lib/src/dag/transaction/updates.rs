use dag::contract::{Contract, ContractValue};
use dag::storage::mpt::NodeUpdates;

#[derive(Debug, PartialEq)]
pub struct TransactionUpdates {
    pub contract: Option<Contract>,
    pub node_updates: Option<NodeUpdates<ContractValue>>,
    pub referenced: Vec<u64>,
}

impl TransactionUpdates {
    pub fn new(referenced: Vec<u64>) -> Self {
        TransactionUpdates {
            contract: None,
            node_updates: None,
            referenced,
        }
    }

    pub fn add_contract(&mut self, contract: Contract) {
        self.contract = Some(contract);
    }

    pub fn add_node_updates(&mut self, node_updates: NodeUpdates<ContractValue>) {
        self.node_updates = Some(node_updates);
    }

    pub fn get_storage_root(&self) -> Option<u64> {
        if let Some(ref updates) = self.node_updates {
            Some(updates.get_root_hash())
        } else {
            None
        }
    }
}
