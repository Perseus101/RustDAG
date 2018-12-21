use dag::{
    transaction::Transaction,
    milestone::pending::{
        PendingMilestone,
        MilestoneSignature,
        _MilestoneErrorTag
    }
};

#[derive(Clone)]
pub enum StateUpdate {
    Chain(Transaction),
    Sign(MilestoneSignature)
}

pub trait PendingMilestoneState {
    fn next(self, event: &StateUpdate)
            -> Result<PendingMilestone, _MilestoneErrorTag>;
}