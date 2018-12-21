/// Temporary structure pending proper cryptography
#[derive(Clone)]
pub struct MilestonePubKey {}

#[derive(Clone)]
pub struct MilestoneSignature {
    milestone: u64,
    contract: u64,
    pub_key: MilestonePubKey,
    next_key: u64,
}

impl MilestoneSignature {
    pub fn new(milestone: u64, contract: u64, next_key: u64) -> Self {
        MilestoneSignature {
            milestone: milestone,
            contract: contract,
            pub_key: MilestonePubKey {},
            next_key: next_key
        }
    }

    pub fn get_milestone(&self) -> u64 {
        self.milestone
    }
    pub fn get_contract(&self) -> u64 {
        self.contract
    }
}