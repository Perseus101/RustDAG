use dag::transaction::Transaction;
use dag::milestone::Milestone;

use super::error::_MilestoneErrorTag;
use super::signing;
use super::bundle;

/// Manages the state of a milestone in the process of being confirmed
///
/// A new milestone is created in the Pending state, and transitions between
/// states based on events from the MilestoneEvent enum.
///
/// Once a milestone enters the Approved state, it is considered confirmed.
#[derive(Clone)]
pub enum PendingMilestone {
    Pending(bundle::MilestoneBundle),
    Signing(bundle::MilestoneBundle),
    Negotiating(bundle::ConflictingBundles),
    Approved(Milestone)
}

/// Milestone confirmation step events
///
/// These enum members represent events that move a milestone from creation
/// to confirmation.
pub enum MilestoneEvent {
    New((u64, Transaction)),
    Sign(signing::MilestoneSignature),
    Select(signing::MilestoneSelection),
}

impl PendingMilestone {
    /// Create a new pending milestone
    pub fn new(transaction: Transaction, previous_milestone: Milestone) -> Self {
        PendingMilestone::Pending(bundle::MilestoneBundle::new(
            previous_milestone.get_hash(), transaction))
    }

    /// Get the hash of the pending milestone
    ///
    /// This returns the hash associated with the pending milestone
    ///
    /// If this pending milestone is in the negotiating state, it returns zero,
    /// since there are multiple milestone hashes associated with that state.
    pub fn get_hash(&self) -> u64 {
        match self {
            PendingMilestone::Pending(bundle) => bundle.get_hash(),
            PendingMilestone::Signing(bundle) => bundle.get_hash(),
            PendingMilestone::Negotiating(_) => 0,
            PendingMilestone::Approved(milestone) => milestone.get_hash(),
        }
    }

    pub fn next(self, event: MilestoneEvent) -> Result<Self, _MilestoneErrorTag> {
        match (self, event) {
            // Pending state
            (PendingMilestone::Pending(bundle), MilestoneEvent::New((prev_hash, transaction))) => {
                let mut conflict: bundle::ConflictingBundles = From::from(PendingMilestone::Pending(bundle));
                conflict.add(PendingMilestone::Pending(bundle::MilestoneBundle::new(prev_hash, transaction)));
                Ok(PendingMilestone::Negotiating(conflict))
            },
            (PendingMilestone::Pending(mut bundle), MilestoneEvent::Sign(signature)) => {
                bundle.add_signature(signature);
                Ok(PendingMilestone::Pending(bundle))
            },
            (PendingMilestone::Pending(bundle), MilestoneEvent::Select(_)) => {
                Err(_MilestoneErrorTag::StaleSelection(PendingMilestone::Pending(bundle)))
            },


            // Signing state
            (PendingMilestone::Signing(bundle), MilestoneEvent::New((prev_hash, transaction))) => {
                let mut conflict: bundle::ConflictingBundles = From::from(PendingMilestone::Signing(bundle));
                conflict.add(PendingMilestone::Pending(bundle::MilestoneBundle::new(prev_hash, transaction)));
                Ok(PendingMilestone::Negotiating(conflict))
            },
            (PendingMilestone::Signing(mut bundle), MilestoneEvent::Sign(signature)) => {
                bundle.add_signature(signature);
                Ok(PendingMilestone::Signing(bundle))
            },
            (PendingMilestone::Signing(bundle), MilestoneEvent::Select(_)) => {
                Err(_MilestoneErrorTag::StaleSelection(PendingMilestone::Signing(bundle)))
            },


            // Negotiating state
            (PendingMilestone::Negotiating(mut conflict), MilestoneEvent::New((prev_hash, transaction))) => {
                conflict.add(PendingMilestone::Pending(bundle::MilestoneBundle::new(prev_hash, transaction)));
                Ok(PendingMilestone::Negotiating(conflict))
            },
            (PendingMilestone::Negotiating(mut conflict), MilestoneEvent::Sign(signature)) => {
                if conflict.add_signature(signature) {
                    Ok(PendingMilestone::Negotiating(conflict))
                }
                else {
                    Err(_MilestoneErrorTag::StaleSignature(PendingMilestone::Negotiating(conflict)))
                }
            },
            (PendingMilestone::Negotiating(mut conflict), MilestoneEvent::Select(selection)) => {
                conflict.select(selection).map_or(Ok(PendingMilestone::Negotiating(conflict)), |pending| Ok(pending))
            },

            // Approved state
            (PendingMilestone::Approved(milestone), _) => { Ok(PendingMilestone::Approved(milestone)) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dag::transaction::Transaction;
    use dag::milestone::Milestone;

    #[test]
    fn test_new_pending_milestone() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0);

        let pending = PendingMilestone::new(transaction.clone(), milestone);
        match pending {
            PendingMilestone::Pending(bundle) => {
                assert_eq!(bundle.get_hash(), transaction.get_hash());
            },
            _ => panic!("Pending milestone not created in pending state"),
        }
    }

    #[test]
    fn test_pending_state() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0);

        let pending = PendingMilestone::new(transaction.clone(), milestone);

        // New milestone incoming
        let second_transaction = Transaction::new(0, hash, Vec::new(), 2, 0, 0);
        match pending.clone().next(MilestoneEvent::New((hash, second_transaction))) {
            Ok(PendingMilestone::Negotiating(_)) => {},
            _ => panic!("New milestone did not move into negotiating state"),
        }

        // New milestone incoming
        match pending.clone().next(MilestoneEvent::Sign(
                signing::MilestoneSignature::new(hash, 0))) {
            Ok(PendingMilestone::Pending(_)) => {},
            _ => panic!("Pending milestone not created in pending state"),
        }


        if let Err(err) = pending.clone().next(MilestoneEvent::Select(
            signing::MilestoneSelection::new(
                signing::MilestoneSignature::new(hash, 0)
            ))) {
            if let _MilestoneErrorTag::StaleSelection(pending) = err {
                match pending {
                    PendingMilestone::Pending(_) => {},
                    _ => panic!("Error wrapped state not returned in pending state"),
                }
            }
            else { panic!("Selection in pending state did not produce stale selection error"); }
        }
        else { panic!("Selection in pending state did not produce error"); }
    }

    #[test]
    fn test_signing_state() {
        // TODO
    }

    #[test]
    fn test_negotiating_state() {
        // TODO
    }

    #[test]
    fn test_approved_state() {
        // TODO
    }
}