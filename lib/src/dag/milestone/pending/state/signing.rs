use std::collections::HashMap;

use dag::{
    milestone::{
        Milestone,
        pending::{PendingMilestone, MilestoneSignature, _MilestoneErrorTag}
    },
    transaction::Transaction
};

use super::state::{PendingMilestoneState, StateUpdate};

/// Signing state
#[derive(Clone)]
pub struct SigningState {
    /// Milestone transaction
    transaction: Transaction,
    /// Hash of the previous milestone
    previous_milestone: u64,
    /// Chain of transactions to the previous milestone
    /// (transaction hash, contract id)
    transaction_chain: Vec<u64>,
    /// Signatures
    signatures: HashMap<u64, bool>
}

impl SigningState {
    pub fn new(transaction: Transaction, previous_milestone: u64,
            chain: Vec<(u64, u64)>) -> Self {
        let mut signatures: HashMap<u64, bool> = HashMap::with_capacity(chain.len());
        let transaction_chain: Vec<u64> = chain.into_iter().map(|(hash, contract)| {
            signatures.insert(contract, false);
            hash
        }).collect();
        SigningState {
            transaction,
            previous_milestone,
            transaction_chain,
            signatures
        }
    }

    fn sign(&mut self, signature: &MilestoneSignature) {
        self.signatures.insert(signature.get_contract(), true);
    }
}

impl PendingMilestoneState for SigningState {
    fn next(mut self, event: &StateUpdate)
            -> Result<PendingMilestone, _MilestoneErrorTag> {
        match event {
            StateUpdate::Chain(_) => Err(_MilestoneErrorTag::StaleChain(PendingMilestone::Signing(Box::new(self)))),
            StateUpdate::Sign(signature) => {
                self.sign(signature);
                if self.signatures.values().all(|value| *value) {
                    Ok(PendingMilestone::Approved(Milestone::new(self.previous_milestone, self.transaction)))
                }
                else {
                    Ok(PendingMilestone::Signing(Box::new(self)))
                }
            }
        }
    }
}

#[cfg(test)]
impl SigningState {
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
    fn test_signing_state() {
        // Create initial milestone
        let previous_milestone_transaction = create_transaction(0, 0, 0);
        let hash = previous_milestone_transaction.get_hash();

        // Intermediate transaction
        let trunk_transaction = create_transaction(0, hash, 1);
        // New milestone
        let transaction = create_transaction(0, trunk_transaction.get_hash(), 2);
        // Unrelated transaction for testing
        let unrelated_transaction = create_transaction(1, hash, 2);

        let pending = SigningState::new(transaction.clone(), hash,
            vec![
                (trunk_transaction.get_hash(), trunk_transaction.get_contract()),
                (transaction.get_hash(), transaction.get_contract())
            ]);

        // Transaction chain incoming
        match pending.clone().next(&StateUpdate::Chain(unrelated_transaction.clone())) {
            Err(_MilestoneErrorTag::StaleChain(PendingMilestone::Signing(state))) => {
                assert_eq!(state.transaction, pending.transaction);
            },
            Err(err) => panic!("Unexpected error while signing: {:?}", err),
            Ok(_) => panic!("New chain transaction in signing state did not raise error"),
        }

        // Incoming signature
        // Add two signatures, one for each of the contracts involved in this milestone
        match pending.clone().next(&StateUpdate::Sign(MilestoneSignature::new(hash, 1, 0))) {
            Ok(PendingMilestone::Signing(pending)) => { // First signature added successfully
                // Add the second signature
                match pending.next(&StateUpdate::Sign(MilestoneSignature::new(hash, 2, 0))) {
                    Ok(PendingMilestone::Approved(milestone)) => {
                        assert_eq!(milestone.get_hash(), transaction.get_hash());
                    },
                    Ok(_) => panic!("Pending milestone did not transition to approved state"),
                    Err(err) => panic!("Unexpected error while signing: {:?}", err)
                }
            },
            Ok(_) => panic!("Pending milestone unexpectedly transitioned away from signing state"),
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        }
    }
}
