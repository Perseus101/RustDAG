use dag::{
    milestone::pending::{MilestoneSignature, PendingMilestone, _MilestoneErrorTag},
    transaction::Transaction,
};

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum StateUpdate {
    Chain(Transaction),
    Sign(MilestoneSignature),
}

pub trait PendingMilestoneState {
    fn next(self, event: &StateUpdate) -> Result<PendingMilestone, _MilestoneErrorTag>;
}
