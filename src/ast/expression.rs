use tree_sitter::Range;

use crate::ast::{binary_op::BinaryOp, statement::Statement};

#[derive(Debug)]
pub enum ExpressionType<M> {
    Binary {
        left: Box<Expression<M>>,
        operator: BinaryOp,
        right: Box<Expression<M>>,
    },
    Block(Vec<Statement<M>>),
    Boolean(bool),
    Identifier(String),
    NegativeInteger(i128),
    PositiveInteger(u128),
    Float(f64),
    If {
        condition: Box<Expression<M>>,
        consequence: Box<Expression<M>>,
        alternative: Option<Box<Expression<M>>>,
    },
    FuncCall {
        name: String,
        arguments: Vec<Expression<M>>,
    },
}

impl<M> ExpressionType<M> {
    pub fn is_literal(&self) -> bool {
        match self {
            ExpressionType::Boolean(_)
            | ExpressionType::NegativeInteger(_)
            | ExpressionType::PositiveInteger(_)
            | ExpressionType::Float(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Expression<M = ()> {
    pub expression_type: ExpressionType<M>,
    pub range: Range,
    pub metadata: M,
}

impl<M> Expression<M> {
    pub fn new(expression_type: ExpressionType<M>, range: Range, metadata: M) -> Self {
        Expression {
            expression_type,
            range,
            metadata,
        }
    }

    pub fn range(&self) -> Range {
        self.range
    }

    pub fn expression_type(&self) -> &ExpressionType<M> {
        &self.expression_type
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
