use std::{error::Error, fmt};

use dag::milestone::pending::PendingMilestone;

/// Describe an error in transitioning between milestone states
#[derive(Debug)]
pub enum MilestoneError {
    StaleSignature,
    StaleSelection,
}

impl Error for MilestoneError {}

impl fmt::Display for MilestoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MilestoneError::StaleSignature => { write!(f, "Stale signature") },
            MilestoneError::StaleSelection => { write!(f, "Stale selection") },
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
    StaleSignature(PendingMilestone),
    StaleSelection(PendingMilestone),
}

impl Error for _MilestoneErrorTag {}

impl fmt::Debug for _MilestoneErrorTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _MilestoneErrorTag::StaleSelection(_) => write!(f, "Stale Selection"),
            _MilestoneErrorTag::StaleSignature(_) => write!(f, "Stale Signature")
        }
    }
}

impl fmt::Display for _MilestoneErrorTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _MilestoneErrorTag::StaleSignature(_) => { write!(f, "Stale signature") },
            _MilestoneErrorTag::StaleSelection(_) => { write!(f, "Stale selection") },
        }
    }
}

impl From<_MilestoneErrorTag> for MilestoneError {
    fn from(err: _MilestoneErrorTag) -> Self {
        match err {
            _MilestoneErrorTag::StaleSelection(_) => MilestoneError::StaleSelection,
            _MilestoneErrorTag::StaleSignature(_) => MilestoneError::StaleSignature,
        }
    }
}

impl _MilestoneErrorTag {
    pub fn convert(self) -> (PendingMilestone, MilestoneError) {
        match self {
            _MilestoneErrorTag::StaleSelection(pending) => (pending, MilestoneError::StaleSelection),
            _MilestoneErrorTag::StaleSignature(pending) => (pending, MilestoneError::StaleSignature),
        }
    }
}