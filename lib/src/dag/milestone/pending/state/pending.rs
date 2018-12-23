use dag::{
    milestone::{
        Milestone,
        pending::{
            PendingMilestone,
            MilestoneError,
            _MilestoneErrorTag
        }
    },
    transaction::Transaction
};

use super::{
    SigningState,
    state::{PendingMilestoneState, StateUpdate},
};

/// Structure for representing a transaction in the DAG search tree
///
/// This struct holds all the data from a single Transaction needed in the
/// search process, and ignores everything else for the sake of efficiency
#[derive(Clone)]
struct MilestoneChainData {
    hash: u64,
    contract: u64,
    timestamp: u64,
    branch: MilestoneTreeNode,
    trunk: MilestoneTreeNode,
}

/// Structure for representing tree nodes in the DAG search tree
#[derive(Clone)]
enum MilestoneTreeNode {
    /// Transaction with unknown state
    Header(u64),
    /// Transaction with known children
    Node(Box<MilestoneChainData>),
    /// Transaction that occurs after the previous milestone
    Leaf(u64)
}

impl MilestoneTreeNode {
    fn new(hash: u64) -> Self {
        MilestoneTreeNode::Header(hash)
    }

    fn create(data: &MilestoneChainData) -> Self {
        MilestoneTreeNode::Node(Box::new(data.clone()))
    }

    fn get_hash(&self) -> u64 {
        match self {
            MilestoneTreeNode::Header(hash) => *hash,
            MilestoneTreeNode::Node(node) => node.hash,
            MilestoneTreeNode::Leaf(hash) => *hash,
        }
    }

    fn update(&mut self, hash: u64, new_data: &MilestoneChainData, milestone: &Milestone)
            -> Result<Option<Vec<(u64, u64)>>, MilestoneError> {
        let mut replace_node: Option<MilestoneTreeNode> = None;
        match self {
            MilestoneTreeNode::Header(_hash) => {
                if *_hash == hash {
                    // replace_node will replace the header node after the match
                    // statement when the borrow is released
                    if new_data.timestamp <= milestone.get_timestamp() {
                        // Milestone isn't on this chain
                        replace_node = Some(MilestoneTreeNode::Leaf(hash));
                    }
                    else {
                        replace_node = Some(MilestoneTreeNode::create(new_data));
                    }
                }
            },
            MilestoneTreeNode::Node(child_node) => {
                match child_node._insert(hash, new_data, milestone) {
                    Err(MilestoneError::DuplicateChain) => {
                        return Err(MilestoneError::DuplicateChain);
                    },
                    Ok(val) => { return Ok(val) },
                    _ => {}
                }
            },
            MilestoneTreeNode::Leaf(_) => {},
        }

        if let Some(new_node) = replace_node {
            *self = new_node;
            let prev_hash = milestone.get_hash();
            return if new_data.branch.get_hash() == prev_hash
                    || new_data.trunk.get_hash() == prev_hash {
                // Pending state complete, return data for signing state
                Ok(Some(vec![
                    (hash, new_data.contract),
                ]))
            }
            else {
                // Node placed, return to pending state
                Ok(None)
            }
        }
        else {
            Err(MilestoneError::StaleChain)
        }
    }
}

impl MilestoneChainData {
    fn new(transaction: &Transaction) -> Self {
        MilestoneChainData {
            hash: transaction.get_hash(),
            contract: transaction.get_contract(),
            timestamp: transaction.get_timestamp(),
            branch: MilestoneTreeNode::new(transaction.get_branch_hash()),
            trunk: MilestoneTreeNode::new(transaction.get_trunk_hash()),
        }
    }

    /// Insert a transaction into the tree
    ///
    /// Returns Ok if the transaction is inserted
    /// If the transaction is the previous milestone, the pending state is
    /// complete, so this function returns data for the signing state
    ///
    /// If there is an error, returns MilestoneError describing what went wrong
    fn insert(&mut self, transaction: &Transaction, milestone: &Milestone)
            -> Result<Option<Vec<(u64, u64)>>, MilestoneError> {
        let hash = transaction.get_hash();
        let new_data = MilestoneChainData::new(transaction);
        self._insert(hash, &new_data, milestone)
    }

    /// Helper function for insert
    fn _insert(&mut self, hash: u64, new_data: &MilestoneChainData, milestone: &Milestone)
            -> Result<Option<Vec<(u64, u64)>>, MilestoneError> {
        if self.hash == hash {
            return Err(MilestoneError::DuplicateChain);
        }
        match self.branch.update(hash, new_data, milestone) {
            Ok(Some(mut vec)) => {
                vec.push((self.hash, self.contract));
                return Ok(Some(vec));
            },
            Ok(None) => {
                return Ok(None)
            },
            Err(MilestoneError::DuplicateChain) => {
                return Err(MilestoneError::DuplicateChain)
            },
            _ => {}
        }
        match self.trunk.update(hash, new_data, milestone) {
            Ok(Some(mut vec)) => {
                vec.push((self.hash, self.contract));
                return Ok(Some(vec));
            },
            Ok(None) => {
                return Ok(None)
            },
            Err(MilestoneError::DuplicateChain) => {
                return Err(MilestoneError::DuplicateChain)
            },
            _ => {}
        }

        Err(MilestoneError::StaleChain)
    }
}

/// Pending state
#[derive(Clone)]
pub struct PendingState {
    /// Root of the DAG search tree
    head: MilestoneChainData,
    /// Milestone transaction
    transaction: Transaction,
    /// Previous milestone
    previous_milestone: Milestone,
}

impl PendingState {
    pub fn new(transaction: Transaction, previous_milestone: Milestone) -> Self {
        PendingState {
            head: MilestoneChainData::new(&transaction),
            transaction,
            previous_milestone
        }
    }
}

impl PendingMilestoneState for PendingState {
    fn next(mut self, event: &StateUpdate)
            -> Result<PendingMilestone, _MilestoneErrorTag> {
        match event {
            StateUpdate::Chain(transaction) => {
                match self.head.insert(transaction, &self.previous_milestone) {
                    Ok(Some(chain)) => {
                        Ok(PendingMilestone::Signing(Box::new(SigningState::new(self.transaction,
                            self.previous_milestone.get_hash(),chain))))
                    },
                    Ok(None) => {
                        Ok(PendingMilestone::Pending(Box::new(self)))
                    },
                    Err(err) => Err(err.convert(PendingMilestone::Pending(Box::new(self))))
                }
            },
            StateUpdate::Sign(_) => Err(_MilestoneErrorTag::StaleSignature(PendingMilestone::Pending(Box::new(self))))
        }
    }
}

#[cfg(test)]
impl PendingState {
    pub fn get_hash(&self) -> u64 {
        self.transaction.get_hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dag::transaction::data::TransactionData;
    use dag::milestone::pending::MilestoneSignature;

    fn create_transaction(branch: u64, trunk: u64, contract: u64) -> Transaction {
        Transaction::new(branch, trunk, Vec::new(), contract, 0, 0, TransactionData::Genesis)
    }

    #[test]
    fn test_pending_state() {
        // Create initial milestone
        let previous_milestone_transaction = create_transaction(0, 0, 0);
        let hash = previous_milestone_transaction.get_hash();
        let previous_milestone = Milestone::new(0, previous_milestone_transaction.clone());

        // Intermediate transaction
        let trunk_transaction = create_transaction(0, hash, 1);
        // New milestone
        let transaction = create_transaction(0, trunk_transaction.get_hash(), 1);
        // Unrelated transaction for testing
        let unrelated_transaction = create_transaction(1, hash, 2);
        assert_ne!(unrelated_transaction.get_hash(), trunk_transaction.get_hash());

        let pending = PendingState::new(transaction.clone(), previous_milestone);

        // Transaction chain incoming
        match pending.clone().next(&StateUpdate::Chain(unrelated_transaction)) {
            Err(_MilestoneErrorTag::StaleChain(PendingMilestone::Pending(state))) => {
                assert_eq!(state.transaction, pending.transaction);
            },
            Err(err) => panic!("Unexpected error while adding chain: {:?}", err),
            Ok(_) => panic!("Expected error"),
        }

        match pending.clone().next(&StateUpdate::Chain(trunk_transaction)) {
            Ok(PendingMilestone::Signing(_)) => {},
            Ok(_) => panic!("Pending milestone did not transition to signing state"),
            Err(err) => panic!("Unexpected error while adding chain: {:?}", err)
        }

        // Signature incoming
        match pending.clone().next(&StateUpdate::Sign(MilestoneSignature::new(hash, 0, 0))) {
            Err(_MilestoneErrorTag::StaleSignature(PendingMilestone::Pending(state))) => {
                assert_eq!(state.transaction, pending.transaction);
            },
            Err(err) => panic!("Unexpected error while adding chain: {:?}", err),
            Ok(_) => panic!("Expected error"),
        }
    }
}
