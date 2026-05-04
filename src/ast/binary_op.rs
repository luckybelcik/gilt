pub enum BinaryOp {
    Add,
    Sub,
    Equal,
    NotEqual,
    Greater,
    Less,
}

impl BinaryOp {
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Greater | BinaryOp::Less
        )
    }
}
