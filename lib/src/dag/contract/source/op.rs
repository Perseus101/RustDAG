#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub enum ContractOp {
    Add((usize, usize, usize)),
    Mul((usize, usize, usize)),
    AddConst((u8, usize, usize)),
    MulConst((u8, usize, usize)),
}
