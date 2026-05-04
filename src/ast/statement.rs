use tree_sitter::Range;

use crate::ast::expression::Expression;

pub enum StatementType {
    VariableDecl {
        is_const: bool,
        name: String,
        type_ann: Option<String>,
        value: Expression,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    Put(Expression),
    Break,
    Expression(Expression),
}

pub struct Statement {
    pub kind: StatementType,
    pub range: Range,
}

impl Statement {
    pub fn new(kind: StatementType, range: Range) -> Self {
        Statement { kind, range }
    }
}
