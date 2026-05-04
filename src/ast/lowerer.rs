use tree_sitter::Node;

use crate::{
    ast::{
        binary_op::BinaryOp,
        expression::{Expression, ExpressionType},
        statement::{Statement, StatementType},
    },
    error_handling::diagnostic::Diagnostic,
};

pub struct Lowerer<'a> {
    source: &'a str,
}

impl<'a> Lowerer<'a> {
    pub fn lower_statement(&self, node: Node) -> Result<Statement, Diagnostic> {
        match node.kind() {
            "variable_declaration" => {
                let is_const = self.child_numerical(node, 0)?.kind() == "const";
                let name = self.text_of_field(node, "name")?.to_string();
                let value_node = self.child_field(node, "value")?;

                let type_ann = node
                    .child_by_field_name("type")
                    .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                    .map(|s| s.to_string());

                Ok(Statement::new(
                    StatementType::VariableDecl {
                        is_const,
                        name,
                        type_ann,
                        value: self.lower_expression(value_node)?,
                    },
                    node.range(),
                ))
            }
            "assignment" => {
                let name = self.text_of_field(node, "name")?.to_string();
                let value_node = self.child_field(node, "value")?;
                Ok(Statement::new(
                    StatementType::Assignment {
                        name,
                        value: self.lower_expression(value_node)?,
                    },
                    node.range(),
                ))
            }
            "put_statement" => {
                let value_node = self.child_field(node, "value")?;
                Ok(Statement::new(
                    StatementType::Put(self.lower_expression(value_node)?),
                    node.range(),
                ))
            }
            "break_statement" => Ok(Statement::new(StatementType::Break, node.range())),
            "block" => Ok(Statement::new(
                StatementType::Expression(self.lower_expression(node)?),
                node.range(),
            )),
            _ if node.kind() == "expression_statement" || node.is_extra() => {
                let expr_node = self.child_numerical(node, 0)?;
                Ok(Statement::new(
                    StatementType::Expression(self.lower_expression(expr_node)?),
                    node.range(),
                ))
            }
            _ => panic!("Unknown statement type: {}", node.kind()),
        }
    }

    pub fn lower_expression(&self, node: Node) -> Result<Expression, Diagnostic> {
        match node.kind() {
            "identifier" => {
                let text = self.text_of_node(node)?;
                Ok(Expression::new(
                    ExpressionType::Identifier(text.to_string()),
                    node.range(),
                ))
            }
            "integer" => {
                let text = self.text_of_node(node)?;
                let is_signed = text.starts_with('-');
                if is_signed {
                    let value = text.parse::<i128>().unwrap();
                    Ok(Expression::new(
                        ExpressionType::SignedInteger(value),
                        node.range(),
                    ))
                } else {
                    let value = text.parse::<u128>().unwrap();
                    Ok(Expression::new(
                        ExpressionType::UnsignedInteger(value),
                        node.range(),
                    ))
                }
            }
            "float" => {
                let text = self.text_of_node(node)?;
                Ok(Expression::new(
                    ExpressionType::Float(text.parse().unwrap()),
                    node.range(),
                ))
            }
            "boolean" => {
                let text = self.text_of_node(node)?;
                Ok(Expression::new(
                    ExpressionType::Boolean(text == "true"),
                    node.range(),
                ))
            }
            "binary_expression" => {
                let left = Box::new(self.lower_expression(self.child_field(node, "left")?)?);
                let op = self.text_of_field(node, "operator")?;
                let operator = match op.as_str() {
                    "+" => BinaryOp::Add,
                    "-" => BinaryOp::Sub,
                    "==" => BinaryOp::Equal,
                    "!=" => BinaryOp::NotEqual,
                    ">" => BinaryOp::Greater,
                    "<" => BinaryOp::Less,
                    _ => panic!("Unknown operator: {}", op),
                };
                let right = Box::new(self.lower_expression(self.child_field(node, "right")?)?);
                Ok(Expression::new(
                    ExpressionType::Binary {
                        left,
                        operator,
                        right,
                    },
                    node.range(),
                ))
            }
            "block" => {
                let statements_result: Vec<Result<Statement, Diagnostic>> = node
                    .children(&mut node.walk())
                    .filter(|child| child.is_named() && child.kind() != "comment")
                    .map(|child| self.lower_statement(child))
                    .collect();

                let mut statements = vec![];

                for statement in statements_result {
                    if let Err(diagnostic) = statement {
                        return Err(diagnostic);
                    }
                    statements.push(statement?);
                }

                Ok(Expression::new(
                    ExpressionType::Block(statements),
                    node.range(),
                ))
            }
            _ => panic!("Unknown expression type: {}", node.kind()),
        }
    }

    fn text_of_node(&self, node: Node) -> Result<String, Diagnostic> {
        node.utf8_text(self.source.as_bytes())
            .map(|s| s.to_string())
            .map_err(|_| Diagnostic::new_internal_error("UTF8 conversion failed", node.range()))
    }

    fn text_of_field(&self, node: Node, field_name: &str) -> Result<String, Diagnostic> {
        let child = node.child_by_field_name(field_name).ok_or_else(|| {
            Diagnostic::new_internal_error(&format!("Missing field '{}'", field_name), node.range())
        })?;

        child
            .utf8_text(self.source.as_bytes())
            .map(|s| s.to_string())
            .map_err(|_| Diagnostic::new_internal_error("UTF8 conversion failed", child.range()))
    }

    fn child_field(&self, node: Node<'a>, field_name: &str) -> Result<Node<'a>, Diagnostic> {
        node.child_by_field_name(field_name).ok_or_else(|| {
            Diagnostic::new_internal_error(&format!("Missing field '{}'", field_name), node.range())
        })
    }

    fn child_numerical(&self, node: Node<'a>, num: u32) -> Result<Node<'a>, Diagnostic> {
        node.child(num).ok_or_else(|| {
            Diagnostic::new_internal_error(&format!("Missing field '({})'", num), node.range())
        })
    }
}
