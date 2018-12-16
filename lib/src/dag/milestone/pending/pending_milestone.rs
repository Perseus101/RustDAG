use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use dag::transaction::Transaction;
use dag::milestone::Milestone;

use super::error::_MilestoneErrorTag;
use super::signing;
use super::bundle::MilestoneBundle;

/// Manages the state of a milestone in the process of being confirmed
///
/// A new milestone is created in the Pending state, and transitions between
/// states based on events from the MilestoneEvent enum.
///
/// Once a milestone enters the Approved state, it is considered confirmed.
#[derive(Clone)]
pub enum PendingMilestone {
    Pending(MilestoneBundle),
    Signing(MilestoneBundle),
    Approved(Milestone)
}

/// Milestone confirmation step events
///
/// These enum members represent events that move a milestone from creation
/// to confirmation.
pub enum MilestoneEvent {
    New((u64, Transaction)),
    Chain(Transaction),
    Sign(signing::MilestoneSignature),
}

impl PendingMilestone {
    /// Create a new pending milestone
    pub fn new(transaction: Transaction, previous_milestone: Milestone) -> Self {
        let milestone_hash = previous_milestone.get_hash();
        if transaction.get_trunk_hash() == milestone_hash
                || transaction.get_branch_hash() == milestone_hash {
            PendingMilestone::Signing(MilestoneBundle::new(previous_milestone.get_hash(), transaction))
        }
        else {
            PendingMilestone::Pending(MilestoneBundle::new(previous_milestone.get_hash(), transaction))
        }
    }

    pub fn next(self, event: MilestoneEvent) -> Result<Self, _MilestoneErrorTag> {
        match (self, event) {
            // Pending state
            (PendingMilestone::Pending(bundle), MilestoneEvent::New((prev_hash, transaction))) => {
                let second_bundle = MilestoneBundle::new(prev_hash, transaction);
                match select_bundle(bundle, second_bundle) {
                    Ok(bundle) => Ok(PendingMilestone::Pending(bundle)),
                    Err(err) => Err(_MilestoneErrorTag::HashCollision(PendingMilestone::Pending(err.get_bundle())))
                }
            },
            (PendingMilestone::Pending(mut bundle), MilestoneEvent::Chain(transaction)) => {
                bundle.add_transaction(transaction);
                if bundle.complete_chain() { Ok(PendingMilestone::Signing(bundle)) }
                else { Ok(PendingMilestone::Pending(bundle)) }
            }
            (PendingMilestone::Pending(mut bundle), MilestoneEvent::Sign(signature)) => {
                bundle.add_signature(signature);
                Ok(PendingMilestone::Pending(bundle))
            },


            // Signing state
            (PendingMilestone::Signing(bundle), MilestoneEvent::New((prev_hash, transaction))) => {
                let second_bundle = MilestoneBundle::new(prev_hash, transaction);
                match select_bundle(bundle, second_bundle) {
                    Ok(bundle) => Ok(PendingMilestone::Signing(bundle)),
                    Err(err) => Err(_MilestoneErrorTag::HashCollision(PendingMilestone::Signing(err.get_bundle())))
                }
            },
            (PendingMilestone::Signing(bundle), MilestoneEvent::Chain(_)) => {
                Err(_MilestoneErrorTag::StaleChain(PendingMilestone::Signing(bundle)))
            }
            (PendingMilestone::Signing(mut bundle), MilestoneEvent::Sign(signature)) => {
                bundle.add_signature(signature);
                if bundle.complete_signatures() { Ok(PendingMilestone::Approved(bundle.into()))}
                else { Ok(PendingMilestone::Signing(bundle)) }
            },


            // Approved state
            (PendingMilestone::Approved(milestone), MilestoneEvent::New((prev_hash, transaction))) => {
                if prev_hash != milestone.get_hash() {
                    Err(_MilestoneErrorTag::ConflictingMilestone(PendingMilestone::Approved(milestone)))
                }
                else {
                    Ok(PendingMilestone::new(transaction, milestone))
                }
            },

            (PendingMilestone::Approved(milestone), _) => { Ok(PendingMilestone::Approved(milestone)) },
        }
    }
}

struct HashCollisionError {
    bundle: MilestoneBundle
}

impl fmt::Debug for HashCollisionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash Collision")
    }
}

impl fmt::Display for HashCollisionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash Collision")
    }
}

impl Error for HashCollisionError {}

impl HashCollisionError {
    pub fn new(bundle: MilestoneBundle) -> Self {
        HashCollisionError { bundle }
    }

    pub fn get_bundle(self) -> MilestoneBundle {
        self.bundle
    }
}

fn select_bundle(first_bundle: MilestoneBundle, second_bundle: MilestoneBundle)
        -> Result<MilestoneBundle, HashCollisionError> {
    let first_transaction = first_bundle.get_milestone_transaction().clone();
    let second_transaction = second_bundle.get_milestone_transaction().clone();
    match first_transaction.get_nonce().cmp(&second_transaction.get_nonce()) {
        Ordering::Less => Ok(first_bundle),
        Ordering::Greater => Ok(second_bundle),
        Ordering::Equal => {
            // In the VERY UNLIKELY event that the nonces are the same,
            // compare hashes to determine priority
            match first_transaction.get_hash()
                    .cmp(&second_transaction.get_hash()) {
                Ordering::Less => Ok(first_bundle),
                Ordering::Greater => Ok(second_bundle),
                Ordering::Equal => {
                    // In the EVEN MORE unlikely event that there is a
                    // hash collision, give up
                    Err(HashCollisionError::new(first_bundle))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dag::transaction::{Transaction, data::TransactionData};
    use dag::milestone::Milestone;

    #[test]
    fn test_new_pending_milestone() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, TransactionData::Genesis);
        let second_transaction = Transaction::new(transaction.get_hash(), 0, Vec::new(), 2, 0, 1, TransactionData::Genesis);

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
    fn test_pending_state() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        let trunk_transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, TransactionData::Genesis);
        let transaction = Transaction::new(0, trunk_transaction.get_hash(),
                Vec::new(), 1, 0, 0, TransactionData::Genesis);

        let pending = PendingMilestone::new(transaction.clone(), milestone);

        // New milestone incoming
        let second_transaction = Transaction::new(0, hash, Vec::new(), 2, 0, 0, TransactionData::Genesis);
        match pending.clone().next(MilestoneEvent::New((hash, second_transaction))) {
            Ok(PendingMilestone::Pending(_)) => {},
            Ok(_) => panic!("New milestone did not remain in pending state"),
            Err(err) => panic!("Unexpected error while creating new milestone: {:?}", err)
        }

        // Transaction chain incoming
        match pending.clone().next(MilestoneEvent::Chain(trunk_transaction)) {
            Ok(PendingMilestone::Signing(_)) => {},
            Ok(_) => panic!("Pending milestone did not transition to signing state"),
            Err(err) => panic!("Unexpected error while adding chain: {:?}", err)
        }

        // Signature incoming
        match pending.clone().next(MilestoneEvent::Sign(
                signing::MilestoneSignature::new(hash, 0, 0))) {
            Ok(PendingMilestone::Pending(_)) => {},
            Ok(_) => panic!("Pending milestone did not stay pending state"),
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        }
    }

    #[test]
    fn test_signing_state() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        // New milestone transaction
        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, TransactionData::Genesis);
        let new_milestone = Transaction::new(transaction.get_hash(), 0,
                Vec::new(), 2, 0, 0, TransactionData::Genesis);

        // Second transaction for testing chain and new events
        let second_transaction = Transaction::new(0, hash, Vec::new(), 2, 0, 1, TransactionData::Genesis);

        // Create a milestone in the signing state
        let pending = match PendingMilestone::new(new_milestone.clone(), milestone)
                .next(MilestoneEvent::Chain(transaction)) {
            Ok(PendingMilestone::Signing(bundle)) => {
                assert_eq!(bundle.get_hash(), new_milestone.get_hash());
                PendingMilestone::Signing(bundle)
            },
            Ok(_) => panic!("Pending milestone did not move into signature state"),
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        };

        // New milestone incoming
        match pending.clone().next(MilestoneEvent::New((hash, second_transaction.clone()))) {
            Ok(PendingMilestone::Signing(_)) => {},
            Ok(_) => panic!("New milestone did not stay in signing state"),
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        }

        // Transaction chain incoming
        match pending.clone().next(MilestoneEvent::Chain(second_transaction.clone())) {
            Err(_MilestoneErrorTag::StaleChain(PendingMilestone::Signing(bundle))) => {
                assert_eq!(bundle.get_hash(), new_milestone.get_hash());
            },
            Err(err) => panic!("Unexpected error while signing: {:?}", err),
            Ok(_) => panic!("New chain transaction in signing state did not raise error"),
        }

        // Incoming signature
        // Add two signatures, one for each of the contracts involved in this milestone
        match pending.clone().next(MilestoneEvent::Sign(signing::MilestoneSignature::new(hash, 1, 0))) {
            Ok(pending) => { // First signature added successfully
                // Assert the milestone is still in the signing state
                match pending.clone() {
                    PendingMilestone::Signing(_) => {},
                    _ => panic!("Pending milestone did not stay in the signing state"),
                }

                // Add the second signature
                match pending.next(MilestoneEvent::Sign(signing::MilestoneSignature::new(hash, 2, 0))) {
                    Ok(PendingMilestone::Approved(milestone)) => {
                        assert_eq!(milestone.get_hash(), new_milestone.get_hash());
                    },
                    Ok(_) => panic!("Pending milestone did not transition to approved state"),
                    Err(err) => panic!("Unexpected error while signing: {:?}", err)
                }
            },
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        }
    }

    #[test]
    fn test_approved_state() {
        let milestone_transaction = Transaction::new(0, 0, Vec::new(), 0, 0, 0, TransactionData::Genesis);
        let hash = milestone_transaction.get_hash();
        let milestone = Milestone::new(0, milestone_transaction);

        // New milestone transaction
        let transaction = Transaction::new(0, hash, Vec::new(), 1, 0, 0, TransactionData::Genesis);
        let new_milestone = Transaction::new(transaction.get_hash(), 0,
                Vec::new(), 2, 0, 0, TransactionData::Genesis);

        // Create a milestone in the signing state
        let pending = match PendingMilestone::new(new_milestone.clone(), milestone)
                .next(MilestoneEvent::Chain(transaction)) {
            Ok(PendingMilestone::Signing(bundle)) => {
                assert_eq!(bundle.get_hash(), new_milestone.get_hash());
                PendingMilestone::Signing(bundle)
            },
            Ok(_) => panic!("Pending milestone did not move into signature state"),
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        };
        // Move the signing state milestone into the approved state
        // Add two signatures, one for each of the contracts involved in this milestone
        let approved = match pending.clone().next(MilestoneEvent::Sign(
                signing::MilestoneSignature::new(hash, 1, 0))) {
            Ok(pending) => { // First signature added successfully
                // Assert the milestone is still in the signing state
                match pending.clone() {
                    PendingMilestone::Signing(_) => {},
                    _ => panic!("Pending milestone did not stay in the signing state"),
                }

                // Add the second signature
                match pending.next(MilestoneEvent::Sign(signing::MilestoneSignature::new(hash, 2, 0))) {
                    Ok(PendingMilestone::Approved(milestone)) => {
                        assert_eq!(milestone.get_hash(), new_milestone.get_hash());
                        PendingMilestone::Approved(milestone)
                    },
                    Ok(_) => panic!("Pending milestone did not transition to approved state"),
                    Err(err) => panic!("Unexpected error while signing: {:?}", err)
                }
            },
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        };

        // Chain and signature events should do nothing
        let second_transaction = Transaction::new(0, hash, Vec::new(), 2, 0, 1, TransactionData::Genesis);
        match approved.clone().next(MilestoneEvent::Chain(second_transaction)) {
            Ok(PendingMilestone::Approved(milestone)) => {
                assert_eq!(milestone.get_hash(), new_milestone.get_hash());
            },
            Ok(_) => panic!("Pending milestone did not remain in approved state"),
            Err(err) => panic!("Unexpected error while adding chain: {:?}", err)
        }

        match approved.clone().next(MilestoneEvent::Sign(
                signing::MilestoneSignature::new(hash, 1, 0))) {
            Ok(PendingMilestone::Approved(milestone)) => {
                assert_eq!(milestone.get_hash(), new_milestone.get_hash());
            },
            Ok(_) => panic!("Pending milestone did not remain in approved state"),
            Err(err) => panic!("Unexpected error while signing: {:?}", err)
        }

        // New Milestone incoming
        let second_milestone_transaction = Transaction::new(hash,
            new_milestone.get_hash(), Vec::new(), 2, 0, 1, TransactionData::Genesis
        );
        match approved.clone().next(MilestoneEvent::New(
                (new_milestone.get_hash(), second_milestone_transaction.clone()))) {
            // Because this milestone directly references the previous, it should
            // jump directly to the signing state
            Ok(PendingMilestone::Signing(bundle)) => {
                assert_eq!(second_milestone_transaction.get_hash(), bundle.get_hash());
            },
            Ok(_) => panic!("Pending milestone did not transition to signing state"),
            Err(err) => panic!("Unexpected error while creating new milestone: {:?}", err)
        }
    }
}