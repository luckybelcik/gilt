use tree_sitter::Range;

use crate::ast::{binary_op::BinaryOp, statement::Statement};

pub enum ExpressionType {
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Block(Vec<Statement>),
    Boolean(bool),
    Identifier(String),
    SignedInteger(i128),
    UnsignedInteger(u128),
    Float(f64),
}

impl ExpressionType {
    pub fn is_literal(&self) -> bool {
        match self {
            ExpressionType::Boolean(_)
            | ExpressionType::SignedInteger(_)
            | ExpressionType::UnsignedInteger(_)
            | ExpressionType::Float(_) => true,
            _ => false,
        }
    }
}

pub struct Expression {
    expression_type: ExpressionType,
    range: Range,
}

impl Expression {
    pub fn new(expression_type: ExpressionType, range: Range) -> Self {
        Expression {
            expression_type,
            range,
        }
    }

    pub fn range(&self) -> Range {
        self.range
    }

    pub fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }
}
