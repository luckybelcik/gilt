use tree_sitter::Node;

use crate::{
    ast::{
        binary_op::BinaryOp,
        expression::{Expression, ExpressionType},
        statement::{DefParameter, Statement, StatementType},
    },
    error_handling::{diagnostic::Diagnostic, diagnostic_kind::DiagnosticKind},
};

pub struct Lowerer<'a> {
    source: &'a str,
}

impl<'a> Lowerer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn lower(&self, root_node: Node) -> Result<Vec<Statement<()>>, Vec<Diagnostic>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            println!("Child kind: {}", child.kind());
            if !child.is_named() {
                continue;
            }

            if child.kind() != "function_definition" {
                errors.push(Diagnostic::new_internal_error(
                    DiagnosticKind::StatementAtTopLevelWhenShouldntBe,
                    child.range(),
                    line!(),
                    file!(),
                ));

                continue;
            }

            match self.lower_statement(child) {
                Ok(stmt) => statements.push(stmt),
                Err(diag) => errors.push(diag),
            }
        }

        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    pub fn lower_statement(&self, node: Node) -> Result<Statement, Diagnostic> {
        match node.kind() {
            "variable_declaration" => {
                let is_const = self.child_numerical(node, 0, line!())?.kind() == "const";
                let name = self
                    .text_of_field(node, "name".to_string(), line!())?
                    .to_string();
                let value_node = self.child_field(node, "value".to_string(), line!())?;

                let type_ann = node
                    .child_by_field_name("type")
                    .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                    .map(|s| s.to_string());

                Ok(Statement::new(
                    StatementType::VarDecl {
                        is_const,
                        name,
                        type_ann,
                        value: Box::new(self.lower_expression(value_node)?),
                    },
                    node.range(),
                    (),
                ))
            }
            "assignment" => {
                let name = self
                    .text_of_field(node, "name".to_string(), line!())?
                    .to_string();
                let value_node = self.child_field(node, "value".to_string(), line!())?;
                Ok(Statement::new(
                    StatementType::Assignment {
                        name,
                        value: Box::new(self.lower_expression(value_node)?),
                    },
                    node.range(),
                    (),
                ))
            }
            "put_statement" => {
                let value_node = self.child_field(node, "value".to_string(), line!())?;
                Ok(Statement::new(
                    StatementType::Put(Box::new(self.lower_expression(value_node)?)),
                    node.range(),
                    (),
                ))
            }
            "break_statement" => Ok(Statement::new(StatementType::Break, node.range(), ())),
            "return_statement" => {
                let value_node = self.child_field(node, "value".to_string(), line!());
                let value = if let Ok(node) = value_node {
                    Some(Box::new(self.lower_expression(node)?))
                } else {
                    None
                };
                Ok(Statement::new(
                    StatementType::Return(value),
                    node.range(),
                    (),
                ))
            }
            _ if node.kind() == "expression_statement" || node.is_extra() => {
                let expr_node = self.child_numerical(node, 0, line!())?;
                Ok(Statement::new(
                    StatementType::Expression(Box::new(self.lower_expression(expr_node)?)),
                    node.range(),
                    (),
                ))
            }
            "function_definition" => {
                let is_public = self.child_numerical(node, 0, line!());
                let is_public = is_public.is_ok();

                let name = self.text_of_field(node, "name".to_string(), line!())?;

                let parameter_list = self.child_field(node, "parameters".to_string(), line!())?;
                let mut parameters = vec![];
                let mut cursor = parameter_list.walk();
                for child in parameter_list.children(&mut cursor).filter(|n| {
                    let text = n.utf8_text(self.source.as_bytes());
                    if let Ok(t) = text {
                        match t {
                            "(" | ")" | "," => false,
                            _ => true,
                        }
                    } else {
                        false
                    }
                }) {
                    let p = DefParameter {
                        name: self.text_of_field(child, "name".to_string(), line!())?,
                        type_ann: self.text_of_field(child, "type".to_string(), line!())?,
                    };

                    parameters.push(p);
                }

                let body_node = self.child_field(node, "body".to_string(), line!())?;
                let body = Box::new(self.lower_expression(body_node)?);

                let return_type = self
                    .text_of_field(node, "return_type".to_string(), line!())
                    .ok();

                Ok(Statement::new(
                    StatementType::FuncDef {
                        is_public,
                        name,
                        parameters,
                        body,
                        return_type,
                    },
                    node.range(),
                    (),
                ))
            }
            _ => panic!(
                "Unknown statement type: {} {}",
                node.kind(),
                node.utf8_text(self.source.as_bytes()).unwrap()
            ),
        }
    }

    pub fn lower_expression(&self, node: Node) -> Result<Expression, Diagnostic> {
        match node.kind() {
            "identifier" => {
                let text = self.text_of_node(node, line!())?;
                Ok(Expression::new(
                    ExpressionType::Identifier(text.to_string()),
                    node.range(),
                    (),
                ))
            }
            "integer" => {
                let text = self.text_of_node(node, line!())?;
                let is_signed = text.starts_with('-');
                if is_signed {
                    let value = text.parse::<i128>().unwrap();
                    Ok(Expression::new(
                        ExpressionType::NegativeInteger(value),
                        node.range(),
                        (),
                    ))
                } else {
                    let value = text.parse::<u128>().unwrap();
                    Ok(Expression::new(
                        ExpressionType::PositiveInteger(value),
                        node.range(),
                        (),
                    ))
                }
            }
            "float" => {
                let text = self.text_of_node(node, line!())?;
                Ok(Expression::new(
                    ExpressionType::Float(text.parse().unwrap()),
                    node.range(),
                    (),
                ))
            }
            "boolean" => {
                let text = self.text_of_node(node, line!())?;
                Ok(Expression::new(
                    ExpressionType::Boolean(text == "true"),
                    node.range(),
                    (),
                ))
            }
            "binary_expression" => {
                let left = Box::new(self.lower_expression(self.child_field(
                    node,
                    "left".to_string(),
                    line!(),
                )?)?);
                let op = self.text_of_field(node, "operator".to_string(), line!())?;
                let operator = match op.as_str() {
                    "+" => BinaryOp::Add,
                    "-" => BinaryOp::Sub,
                    "==" => BinaryOp::Equal,
                    "!=" => BinaryOp::NotEqual,
                    ">" => BinaryOp::Greater,
                    "<" => BinaryOp::Less,
                    _ => panic!("Unknown operator: {}", op),
                };
                let right = Box::new(self.lower_expression(self.child_field(
                    node,
                    "right".to_string(),
                    line!(),
                )?)?);
                Ok(Expression::new(
                    ExpressionType::Binary {
                        left,
                        operator,
                        right,
                    },
                    node.range(),
                    (),
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
                    (),
                ))
            }
            "if_statement" => {
                let condition = self.child_field(node, "condition".to_string(), line!())?;
                let condition = self.lower_expression(condition)?;

                let consequence = self.child_field(node, "consequence".to_string(), line!())?;
                let consequence = self.lower_expression(consequence)?;

                let alternative = self
                    .child_field(node, "alternative".to_string(), line!())
                    .ok();
                let alternative = if let Some(a) = alternative {
                    Some(Box::new(self.lower_expression(a)?))
                } else {
                    None
                };

                Ok(Expression::new(
                    ExpressionType::If {
                        condition: Box::new(condition),
                        consequence: Box::new(consequence),
                        alternative: alternative,
                    },
                    node.range(),
                    (),
                ))
            }
            "function_call" => {
                let name = self.text_of_field(node, "name".to_string(), line!())?;
                let parameter_list = self.child_field(node, "arguments".to_string(), line!())?;
                let mut arguments = vec![];
                let mut cursor = parameter_list.walk();
                for child in parameter_list.children(&mut cursor).filter(|n| {
                    let text = n.utf8_text(self.source.as_bytes());
                    if let Ok(t) = text {
                        match t {
                            "(" | ")" | "," => false,
                            _ => true,
                        }
                    } else {
                        false
                    }
                }) {
                    let expr = self.lower_expression(child)?;
                    arguments.push(expr);
                }

                Ok(Expression::new(
                    ExpressionType::FuncCall { name, arguments },
                    node.range(),
                    (),
                ))
            }
            _ => {
                panic!("Unknown expression type: {}", node.kind())
            }
        }
    }

    fn text_of_node(&self, node: Node, line: u32) -> Result<String, Diagnostic> {
        node.utf8_text(self.source.as_bytes())
            .map(|s| s.to_string())
            .map_err(|_| {
                Diagnostic::new_internal_error(
                    DiagnosticKind::UTF8ConversionFailed,
                    node.range(),
                    line,
                    file!(),
                )
            })
    }

    fn text_of_field(
        &self,
        node: Node,
        field_name: String,
        line: u32,
    ) -> Result<String, Diagnostic> {
        let child = node.child_by_field_name(&field_name).ok_or_else(|| {
            Diagnostic::new_internal_error(
                DiagnosticKind::MissingFieldNamed(field_name),
                node.range(),
                line,
                file!(),
            )
        })?;

        child
            .utf8_text(self.source.as_bytes())
            .map(|s| s.to_string())
            .map_err(|_| {
                Diagnostic::new_internal_error(
                    DiagnosticKind::UTF8ConversionFailed,
                    child.range(),
                    line,
                    file!(),
                )
            })
    }

    fn child_field(
        &self,
        node: Node<'a>,
        field_name: String,
        line: u32,
    ) -> Result<Node<'a>, Diagnostic> {
        node.child_by_field_name(&field_name).ok_or_else(|| {
            Diagnostic::new_internal_error(
                DiagnosticKind::MissingFieldNamed(field_name),
                node.range(),
                line,
                file!(),
            )
        })
    }

    fn child_numerical(&self, node: Node<'a>, num: u32, line: u32) -> Result<Node<'a>, Diagnostic> {
        node.child(num).ok_or_else(|| {
            Diagnostic::new_internal_error(
                DiagnosticKind::MissingFieldNumbered(num),
                node.range(),
                line,
                file!(),
            )
        })
    }
}
