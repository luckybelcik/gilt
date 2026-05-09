use tree_sitter::Range;

use crate::ast::expression::Expression;

#[derive(Debug)]
pub enum StatementType<M> {
    VariableDecl {
        is_const: bool,
        name: String,
        type_ann: Option<String>,
        value: Expression<M>,
    },
    Assignment {
        name: String,
        value: Expression<M>,
    },
    Put(Expression<M>),
    Break,
    Expression(Expression<M>),
}

#[derive(Debug)]
pub struct Statement<M = ()> {
    pub kind: StatementType<M>,
    pub range: Range,
    pub metadata: M,
}

impl<M> Statement<M> {
    pub fn new(kind: StatementType<M>, range: Range, metadata: M) -> Self {
        Statement {
            kind,
            range,
            metadata,
        }
    }
}
