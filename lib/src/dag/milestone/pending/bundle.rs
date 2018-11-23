use std::collections::HashMap;

use dag::milestone::Milestone;
use dag::transaction::Transaction;

use super::PendingMilestone;
use super::signing::{MilestoneSignature, MilestoneSelection};

/// Bundle of Milestone data
///
/// This data is used to build a full milestone while it is being confirmed
#[derive(Clone)]
pub struct MilestoneBundle {
    previous_milestone: u64,
    transaction: Transaction,
    transaction_chain: Vec<u64>,
    signatures: Vec<MilestoneSignature>,
}

impl MilestoneBundle {
    pub fn new(previous_milestone: u64, transaction: Transaction) -> MilestoneBundle {
        MilestoneBundle {
            previous_milestone: previous_milestone,
            transaction: transaction,
            transaction_chain: Vec::new(),
            signatures: Vec::new(),
        }
    }

    pub fn get_hash(&self) -> u64 {
        self.transaction.get_hash()
    }

    pub fn add_signature(&mut self, signature: MilestoneSignature) {
        self.signatures.push(signature);
    }
}

impl From<MilestoneBundle> for Milestone {
    fn from(from: MilestoneBundle) -> Self {
        Milestone::new(from.previous_milestone, from.transaction)
    }
}

#[derive(Clone)]
pub struct ConflictingBundles {
    bundles: HashMap<u64, PendingMilestone>,
}

impl ConflictingBundles {
    pub fn add(&mut self, pending: PendingMilestone) {
        self.bundles.insert(pending.get_hash(), pending);
    }

    pub fn add_signature(&mut self, signature: MilestoneSignature) -> bool {
        if let Some(pending) = self.bundles.get_mut(&signature.get_milestone()) {
            match pending {
                PendingMilestone::Pending(bundle) |
                PendingMilestone::Signing(bundle) => {
                    bundle.add_signature(signature);
                    true
                },
                PendingMilestone::Negotiating(_) |
                PendingMilestone::Approved(_) => {
                    false
                }
            }
        }
        else {
            false
        }
    }

    pub fn select(&mut self, selection: MilestoneSelection) -> Option<PendingMilestone> {
        self.bundles.remove(&selection.signature.get_milestone())
    }
}

impl From<PendingMilestone> for ConflictingBundles {
    fn from(pending: PendingMilestone) -> Self {
        let mut bundles = HashMap::new();
        bundles.insert(pending.get_hash(), pending);
        ConflictingBundles {
            bundles: bundles,
        }
    }
}
