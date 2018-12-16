use std::{error::Error, fmt};

use dag::milestone::pending::PendingMilestone;

/// Describe an error in transitioning between milestone states
#[derive(Debug)]
pub enum MilestoneError {
    StaleChain,
    StaleSignature,
    ConflictingMilestone,
    HashCollision
}

impl Error for MilestoneError {}

impl fmt::Display for MilestoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MilestoneError::StaleChain => write!(f, "Stale Chain"),
            MilestoneError::StaleSignature => write!(f, "Stale Signature"),
            MilestoneError::ConflictingMilestone => write!(f, "Conflicting Milestone"),
            MilestoneError::HashCollision => write!(f, "Milestone Hash Collision"),
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
    StaleSignature(PendingMilestone),
    ConflictingMilestone(PendingMilestone),
    HashCollision(PendingMilestone)
}

impl Error for _MilestoneErrorTag {}

impl fmt::Debug for _MilestoneErrorTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _MilestoneErrorTag::StaleChain(_) => write!(f, "Stale Chain"),
            _MilestoneErrorTag::StaleSignature(_) => write!(f, "Stale Signature"),
            _MilestoneErrorTag::ConflictingMilestone(_) => write!(f, "Conflicting Milestone"),
            _MilestoneErrorTag::HashCollision(_) => write!(f, "Milestone Hash Collision")
        }
    }
}

impl fmt::Display for _MilestoneErrorTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _MilestoneErrorTag::StaleChain(_) => write!(f, "Stale Chain"),
            _MilestoneErrorTag::StaleSignature(_) => write!(f, "Stale Signature"),
            _MilestoneErrorTag::ConflictingMilestone(_) => write!(f, "Conflicting Milestone"),
            _MilestoneErrorTag::HashCollision(_) => write!(f, "Milestone Hash Collision")
        }
    }
}

impl From<_MilestoneErrorTag> for MilestoneError {
    fn from(err: _MilestoneErrorTag) -> Self {
        match err {
            _MilestoneErrorTag::StaleChain(_) => MilestoneError::StaleChain,
            _MilestoneErrorTag::StaleSignature(_) => MilestoneError::StaleSignature,
            _MilestoneErrorTag::ConflictingMilestone(_) => MilestoneError::ConflictingMilestone,
            _MilestoneErrorTag::HashCollision(_) => MilestoneError::HashCollision
        }
    }
}

impl _MilestoneErrorTag {
    pub fn convert(self) -> (PendingMilestone, MilestoneError) {
        match self {
            _MilestoneErrorTag::StaleChain(pending) => (pending, MilestoneError::StaleChain),
            _MilestoneErrorTag::StaleSignature(pending) => (pending, MilestoneError::StaleSignature),
            _MilestoneErrorTag::ConflictingMilestone(pending) => (pending, MilestoneError::ConflictingMilestone),
            _MilestoneErrorTag::HashCollision(pending) => (pending, MilestoneError::HashCollision)
        }
    }
}