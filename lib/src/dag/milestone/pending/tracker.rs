use std::collections::HashMap;

use dag::{
    transaction::Transaction,
    milestone::{
        Milestone,
        pending::{
            MilestoneSignature,
            MilestoneError,
            state::StateUpdate
        }
    }
};

use super::PendingMilestone;

///
pub struct MilestoneTracker {
    milestones: Vec<Milestone>,
    pending_milestones: HashMap<u64, PendingMilestone>
}

impl MilestoneTracker {
    /// Create tracker from initial milestone
    pub fn new(milestone: Milestone) -> Self {
        MilestoneTracker {
            milestones: vec![milestone],
            pending_milestones: HashMap::new()
        }
    }

    /// Insert a new pending milestone
    pub fn new_milestone(&mut self, transaction: Transaction) -> bool {
        let hash = transaction.get_hash();
        if let Some(_) = self.pending_milestones.get(&hash) {
            false
        }
        else {
            let milestone = self.get_head_milestone().clone();
            self.pending_milestones.insert(hash, PendingMilestone::new(transaction, milestone));
            true
        }
    }

    /// Add a new chain element to the pending milestone specified by hash
    pub fn new_chain(&mut self, hash: u64, transaction: Transaction)
            -> Result<(), MilestoneError> {
        if let Some(milestone) = self.pending_milestones.get_mut(&hash) {
            milestone.next(StateUpdate::Chain(transaction))
        }
        else {
            Err(MilestoneError::StaleChain)
        }
    }

    /// Add a new signature to a pending milestone
    pub fn sign(&mut self, signature: MilestoneSignature)
            -> Result<Option<Milestone>, MilestoneError> {
        let hash = signature.get_milestone();
        if let Some(pending_milestone) = self.pending_milestones.get_mut(&hash) {
            if let Err(err) = pending_milestone.next(StateUpdate::Sign(signature)) {
                Err(err)
            }
            else {
                if let PendingMilestone::Approved(milestone) = pending_milestone {
                    Ok(Some(milestone.clone()))
                }
                else {
                    Ok(None)
                }
            }

        }
        else {
            Err(MilestoneError::StaleSignature)
        }
    }

    /// Get the most recent milestone
    pub fn get_head_milestone(&self) -> &Milestone {
        &self.milestones[self.milestones.len() - 1]
    }
}
