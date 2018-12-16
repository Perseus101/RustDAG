use std::collections::HashMap;

use dag::milestone::Milestone;
use dag::transaction::Transaction;

use super::signing::MilestoneSignature;

/// Bundle of Milestone data
///
/// This data is used to build a full milestone while it is being confirmed
#[derive(Clone)]
pub struct MilestoneBundle {
    previous_milestone: u64,
    transaction_chain: Vec<Transaction>,
    signatures: HashMap<u64, MilestoneSignature>,
}

impl MilestoneBundle {
    pub fn new(previous_milestone: u64, transaction: Transaction) -> MilestoneBundle {
        MilestoneBundle {
            previous_milestone: previous_milestone,
            transaction_chain: vec![transaction],
            signatures: HashMap::new(),
        }
    }

    pub fn get_milestone_transaction(&self) -> &Transaction {
        &self.transaction_chain[0]
    }

    pub fn get_hash(&self) -> u64 {
        self.transaction_chain[0].get_hash()
    }

    pub fn add_signature(&mut self, signature: MilestoneSignature) {
        self.signatures.insert(signature.get_contract(), signature);
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> bool {
        let mut valid = false;
        if let Some(tx) = self.transaction_chain.last() {
            let hash = transaction.get_hash();
            if tx.get_branch_hash() == hash || tx.get_trunk_hash() == hash {
                valid = true;
            }
        }
        if valid {
            self.transaction_chain.push(transaction);
            true
        }
        else { false }
    }

    pub fn complete_chain(&self) -> bool {
        let mut ref_iter = self.transaction_chain.iter().skip(1);
        for transaction in self.transaction_chain.iter() {
            // Check integrity of the transaction chain to the previous milestone
            if let Some(tx) = ref_iter.next() {
                // Check transactions in the chain
                let hash = tx.get_hash();
                if transaction.get_branch_hash() != hash
                        && transaction.get_trunk_hash() != hash {
                    return false;
                }
            }
            else {
                // Check that the last transaction references the previous milestone
                if transaction.get_branch_hash() != self.previous_milestone
                        && transaction.get_trunk_hash() != self.previous_milestone {
                    return false;
                }
            }
        }
        true
    }

    pub fn complete_signatures(&self) -> bool {
        for transaction in self.transaction_chain.iter() {
            // Look for a signature for this transaction
            if let Some(signature) =
                    self.signatures.get(&transaction.get_contract()) {
                // TODO check signature validity
            }
            else {
                println!("Missing signature: {:?}", transaction.get_contract());
                return false;
            }
        }
        true
    }
}

impl From<MilestoneBundle> for Milestone {
    fn from(from: MilestoneBundle) -> Self {
        Milestone::new(from.previous_milestone, from.transaction_chain[0].clone())
    }
}