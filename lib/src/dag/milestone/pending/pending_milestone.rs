use dag::transaction::Transaction;
use dag::milestone::{
    Milestone,
    pending::{
        MilestoneError,
        _MilestoneErrorTag
    }
};

use replace_with::replace_with_or_abort;

use super::state::{
    PendingState,
    SigningState,
    PendingMilestoneState,
    StateUpdate,
};

/// Manages the state of a milestone in the process of being confirmed
///
/// A new milestone is created in the Pending state, and transitions between
/// states based on events from the MilestoneEvent enum.
///
/// Once a milestone enters the Approved state, it is considered confirmed.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum PendingMilestone {
    Pending(PendingState),
    Signing(SigningState),
    Approved(Milestone),
}

impl PendingMilestone {
    /// Create a new pending milestone
    pub fn new(transaction: Transaction, previous_milestone: Milestone) -> Self {
        let milestone_hash = previous_milestone.get_hash();
        if transaction.get_trunk_hash() == milestone_hash
                || transaction.get_branch_hash() == milestone_hash {
            let transaction_chain = vec![
                (transaction.get_hash(), transaction.get_contract())
            ];
            PendingMilestone::Signing(SigningState::new(transaction, milestone_hash, transaction_chain))
        }
        else {
            PendingMilestone::Pending(PendingState::new(transaction, previous_milestone))
        }
    }

    pub fn next(&mut self, event: StateUpdate) -> Result<(), MilestoneError> {
        let mut res = Ok(());
        replace_with_or_abort(self, |_self| {
            let out = match _self {
                PendingMilestone::Pending(pending) => {
                    pending.next(&event)
                }
                PendingMilestone::Signing(signing) => {
                    signing.next(&event)
                }
                PendingMilestone::Approved(_) => {
                    match event {
                        StateUpdate::Chain(_) => Err(_MilestoneErrorTag::StaleChain(_self)),
                        StateUpdate::Sign(_) => Err(_MilestoneErrorTag::StaleSignature(_self))
                    }
                }
            };
            match out {
                Ok(state) => state,
                Err(err) => {
                    let (state, _err) = err.convert();
                    res = Err(_err);
                    state
                }
            }
        });
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dag::transaction::{Transaction, data::TransactionData};
    use dag::milestone::Milestone;
    use dag::milestone::pending::MilestoneSignature;

    fn update(pending: &PendingMilestone, update: StateUpdate) -> PendingMilestone {
        let mut _pending = pending.clone();
        assert!(_pending.next(update).is_ok());
        _pending
    }

    fn raw_update(pending: &PendingMilestone, update: StateUpdate)
            -> Result<(), MilestoneError> {
        let mut _pending = pending.clone();
        let res = _pending.next(update);
        if let Err(_) = &res {
            match pending {
                PendingMilestone::Pending(_) => {
                    match _pending {
                        PendingMilestone::Pending(_) => {},
                        _ => panic!("Unexpected state transition")
                    }
                },
                PendingMilestone::Signing(_) => {
                    match _pending {
                        PendingMilestone::Signing(_) => {},
                        _ => panic!("Unexpected state transition")
                    }
                },
                PendingMilestone::Approved(_) => {
                    match _pending {
                        PendingMilestone::Approved(_) => {},
                        _ => panic!("Unexpected state transition")
                    }
                }
            }
        }
        res
    }

    #[test]
    fn test_new_pending_milestone() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, 0, TransactionData::Genesis);
        let second_transaction = Transaction::new(transaction.get_hash(), 0,
            Vec::new(), 2, 0, 1, 0, TransactionData::Genesis);

        {
            // Test milestone that should start in pending
            let pending = PendingMilestone::new(second_transaction.clone(), milestone.clone());
            match pending {
                PendingMilestone::Pending(bundle) => {
                    assert_eq!(bundle.get_hash(), second_transaction.get_hash());
                },
                _ => panic!("Pending milestone not created in pending state"),
            }
        }

        {
            // Test milestone that should jump directly to signing
            let pending = PendingMilestone::new(transaction.clone(), milestone);
            match pending {
                PendingMilestone::Signing(bundle) => {
                    assert_eq!(bundle.get_hash(), transaction.get_hash());
                },
                _ => panic!("Pending milestone not created in signing state"),
            }
        }
    }

    #[test]
    fn test_pending_milestone_state() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        let trunk_transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, 0, TransactionData::Genesis);
        let transaction = Transaction::new(0, trunk_transaction.get_hash(),
                Vec::new(), 1, 0, 0, 0, TransactionData::Genesis);

        let pending = PendingMilestone::new(transaction.clone(), milestone);

        // Transaction chain incoming
        match update(&pending, StateUpdate::Chain(trunk_transaction)) {
            PendingMilestone::Signing(_) => {},
            _ => panic!("Pending milestone did not transition to signing state"),
        }

        // Signature incoming
        match raw_update(&pending, StateUpdate::Sign(
                MilestoneSignature::new(hash, 0, 0))) {
            Err(MilestoneError::StaleSignature) => {},
            Err(err) => panic!("Unexpected error while signing: {:?}", err),
            Ok(_) => panic!("New signature in pending state did not raise error"),
        }

    }

    #[test]
    fn test_signing_milestone_state() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        // New milestone transaction
        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, 0, TransactionData::Genesis);
        let new_milestone = Transaction::new(transaction.get_hash(), 0,
                Vec::new(), 2, 0, 0, 0, TransactionData::Genesis);

        // Second transaction for testing chain and new events
        let second_transaction = Transaction::new(0, hash, Vec::new(), 2, 0, 1, 0, TransactionData::Genesis);

        // Create a milestone in the signing state
        let mut pending = PendingMilestone::new(new_milestone.clone(), milestone);
        assert!(pending.next(StateUpdate::Chain(transaction)).is_ok());
        match &pending {
            PendingMilestone::Signing(_) => {},
            _ => panic!("Failed to create pending milestone in signing state")
        }
        // Transaction chain incoming
        match raw_update(&pending, StateUpdate::Chain(second_transaction.clone())) {
            Err(MilestoneError::StaleChain) => {},
            Err(err) => panic!("Unexpected error while signing: {:?}", err),
            Ok(_) => panic!("New chain transaction in signing state did not raise error"),
        }

        // Incoming signature
        // Add two signatures, one for each of the contracts involved in this milestone
        match update(&pending, StateUpdate::Sign(MilestoneSignature::new(hash, 1, 0))) {
            PendingMilestone::Signing(state) => {
                // First signature added successfully
                // Add the second signature
                match update(&PendingMilestone::Signing(state),
                        StateUpdate::Sign(MilestoneSignature::new(hash, 2, 0))) {
                    PendingMilestone::Approved(milestone) => {
                        assert_eq!(milestone.get_hash(), new_milestone.get_hash());
                    },
                    _ => panic!("Pending milestone did not transition to approved state")
                }
            },
            _ => panic!("Unexpected state transition")
        }
    }

    #[test]
    fn test_approved_state() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        // New milestone transaction
        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, 0, TransactionData::Genesis);
        let new_milestone = Transaction::new(transaction.get_hash(), 0,
                Vec::new(), 2, 0, 0, 0, TransactionData::Genesis);

        // Create a milestone in the signing state
        let mut pending = PendingMilestone::new(new_milestone.clone(), milestone);
        assert!(pending.next(StateUpdate::Chain(transaction)).is_ok());
        // Move the signing state milestone into the approved state
        // Add two signatures, one for each of the contracts involved in this milestone
        assert!(pending.next(StateUpdate::Sign(
                MilestoneSignature::new(hash, 1, 0))).is_ok());
        assert!(pending.next(StateUpdate::Sign(
                MilestoneSignature::new(hash, 2, 0))).is_ok());

        let approved = pending;
        // Chain and signature events should raise errors
        let second_transaction = Transaction::new(0, hash, Vec::new(), 2, 0, 1, 0, TransactionData::Genesis);
        match raw_update(&approved, StateUpdate::Chain(second_transaction)) {
            Err(MilestoneError::StaleChain) => {},
            Err(err) => panic!("Unexpected error while adding chain: {:?}", err),
            Ok(()) => panic!("Expected error did not occur")
        }

        match raw_update(&approved, StateUpdate::Sign(
                MilestoneSignature::new(hash, 1, 0))) {
            Err(MilestoneError::StaleSignature) => {},
            Err(err) => panic!("Unexpected error while adding chain: {:?}", err),
            Ok(()) => panic!("Expected error did not occur")
        }
    }
}