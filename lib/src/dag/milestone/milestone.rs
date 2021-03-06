use dag::transaction::Transaction;

#[derive(Clone)]
pub struct Milestone {
    previous_milestone: u64,
    transaction: Transaction,
}

impl Milestone {
    pub fn new(previous_milestone: u64, transaction: Transaction) -> Milestone {
        Milestone {
            previous_milestone,
            transaction,
        }
    }

    pub fn get_hash(&self) -> u64 {
        self.transaction.get_hash()
    }

    pub fn get_timestamp(&self) -> u64 {
        self.transaction.get_timestamp()
    }

    pub fn get_transaction(&self) -> &Transaction {
        &self.transaction
    }
}
