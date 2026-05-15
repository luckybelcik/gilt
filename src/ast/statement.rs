use tree_sitter::Range;

use crate::ast::expression::Expression;

#[derive(Debug)]
pub struct DefParameter {
    pub name: String,
    pub type_ann: String,
}

#[derive(Debug)]
pub enum StatementType<M> {
    VarDecl {
        is_const: bool,
        name: String,
        type_ann: Option<String>,
        value: Box<Expression<M>>,
    },
    Assignment {
        name: String,
        value: Box<Expression<M>>,
    },
    Put(Box<Expression<M>>),
    Break,
    Return(Option<Box<Expression<M>>>),
    Expression(Box<Expression<M>>),
    FuncDef {
        is_public: bool,
        name: String,
        parameters: Vec<DefParameter>,
        body: Box<Expression<M>>,
        return_type: Option<String>,
    },
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
