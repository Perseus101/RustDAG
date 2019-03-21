use std::{error::Error, fmt};

use dag::milestone::pending::PendingMilestone;

/// Describe an error in transitioning between milestone states
#[derive(Debug)]
pub enum MilestoneError {
    StaleChain,
    DuplicateChain,
    StaleSignature,
    DuplicateMilestone,
    NotPending,
}

impl Error for MilestoneError {}

impl fmt::Display for MilestoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MilestoneError::StaleChain => write!(f, "Stale Chain"),
            MilestoneError::DuplicateChain => write!(f, "Duplicate Chain"),
            MilestoneError::StaleSignature => write!(f, "Stale Signature"),
            MilestoneError::DuplicateMilestone => write!(f, "Duplicate Milestone"),
            MilestoneError::NotPending => write!(f, "Pending Milestone not found"),
        }
    }
}

impl MilestoneError {
    pub fn convert(self, pending: PendingMilestone) -> _MilestoneErrorTag {
        match self {
            MilestoneError::StaleChain => _MilestoneErrorTag::StaleChain(pending),
            MilestoneError::DuplicateChain => _MilestoneErrorTag::DuplicateChain(pending),
            MilestoneError::StaleSignature => _MilestoneErrorTag::StaleSignature(pending),
            MilestoneError::DuplicateMilestone => _MilestoneErrorTag::DuplicateMilestone(pending),
            MilestoneError::NotPending => _MilestoneErrorTag::NotPending(pending),
        }
    }
}
/// Describe an error in transitioning between milestone states
///
/// Because the state machine consumes the current pending milestone when
/// transitioning, this error preserves the state so that it can be recovered.
/// This error type should only be returned by the PendingMilestone next method.
/// For returning an error to an outside function, convert this error to a
/// MilestoneError.
pub enum _MilestoneErrorTag {
    StaleChain(PendingMilestone),
    DuplicateChain(PendingMilestone),
    StaleSignature(PendingMilestone),
    DuplicateMilestone(PendingMilestone),
    NotPending(PendingMilestone),
}

impl Error for _MilestoneErrorTag {}

impl fmt::Debug for _MilestoneErrorTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _MilestoneErrorTag::StaleChain(_) => write!(f, "Stale Chain"),
            _MilestoneErrorTag::DuplicateChain(_) => write!(f, "Duplicate Chain"),
            _MilestoneErrorTag::StaleSignature(_) => write!(f, "Stale Signature"),
            _MilestoneErrorTag::DuplicateMilestone(_) => write!(f, "Duplicate Milestone"),
            _MilestoneErrorTag::NotPending(_) => write!(f, "Pending Milestone not found"),
        }
    }
}

impl fmt::Display for _MilestoneErrorTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _MilestoneErrorTag::StaleChain(_) => write!(f, "Stale Chain"),
            _MilestoneErrorTag::DuplicateChain(_) => write!(f, "Duplicate Chain"),
            _MilestoneErrorTag::StaleSignature(_) => write!(f, "Stale Signature"),
            _MilestoneErrorTag::DuplicateMilestone(_) => write!(f, "Duplicate Milestone"),
            _MilestoneErrorTag::NotPending(_) => write!(f, "Pending Milestone not found"),
        }
    }
}

impl _MilestoneErrorTag {
    pub fn convert(self) -> (PendingMilestone, MilestoneError) {
        match self {
            _MilestoneErrorTag::StaleChain(pending) => (pending, MilestoneError::StaleChain),
            _MilestoneErrorTag::DuplicateChain(pending) => {
                (pending, MilestoneError::DuplicateChain)
            }
            _MilestoneErrorTag::StaleSignature(pending) => {
                (pending, MilestoneError::StaleSignature)
            }
            _MilestoneErrorTag::DuplicateMilestone(pending) => {
                (pending, MilestoneError::DuplicateMilestone)
            }
            _MilestoneErrorTag::NotPending(pending) => (pending, MilestoneError::NotPending),
        }
    }
}
