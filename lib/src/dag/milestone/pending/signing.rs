/// Temporary structure pending proper cryptography
#[derive(Clone)]
pub struct MilestonePubKey {}

#[derive(Clone)]
pub struct MilestoneSignature {
    milestone: u64,
    pub_key: MilestonePubKey,
    next_key: u64,
}

impl MilestoneSignature {
    pub fn new(milestone: u64, next_key: u64) -> Self {
        MilestoneSignature {
            milestone: milestone,
            pub_key: MilestonePubKey {},
            next_key: next_key
        }
    }

    pub fn get_milestone(&self) -> u64 {
        self.milestone
    }
}

pub struct MilestoneSelection {
    pub signature: MilestoneSignature,
}

impl From<MilestoneSelection> for MilestoneSignature {
    fn from(selection: MilestoneSelection) -> Self {
        selection.signature
    }
}

impl MilestoneSelection {
    pub fn new(signature: MilestoneSignature) -> Self {
        MilestoneSelection {
            signature: signature
        }
    }
}