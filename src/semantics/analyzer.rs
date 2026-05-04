use rustc_hash::FxHashMap;
use tree_sitter::Range;

use crate::{
    ast::{
        expression::{Expression, ExpressionType},
        statement::{Statement, StatementType},
    },
    error_handling::diagnostic::Diagnostic,
    semantics::{
        symbol_table::{Symbol, SymbolTable},
        types::GiltType,
    },
};

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    diagnostics: Vec<Diagnostic>,
    loop_depth: usize,
    block_depth: usize,
}

impl SemanticAnalyzer {
    pub fn check_statement(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementType::VariableDecl {
                is_const,
                name,
                type_ann,
                value,
            } => {
                let expected_type = if let Some(ex) = type_ann {
                    Some(&GiltType::from_string(ex))
                } else {
                    None
                };

                let value_type = self.check_expression(value, expected_type);

                self.symbols
                    .define(Symbol {
                        name: name.clone(),
                        is_const: *is_const,
                        symbol_type: value_type,
                    })
                    .unwrap_or_else(|err| self.report_error(&err, stmt.range));
            }
            StatementType::Assignment { name, value } => {
                let symbol = self.symbols.resolve(&name).cloned();

                if symbol.is_none() {
                    self.report_error("Undefined symbol", stmt.range);
                }

                // we can unwrap the symbol here because we already checked for None above
                let symbol = symbol.unwrap();

                if symbol.is_const {
                    self.report_error("Cannot assign to constant", stmt.range);
                }

                let value_type = self.check_expression(value, Some(&symbol.symbol_type));

                if !value_type.coercable_to(&symbol.symbol_type) {
                    self.report_error(
                        &format!(
                            "Type {} not coercable to {}",
                            value_type, symbol.symbol_type
                        ),
                        stmt.range,
                    );
                }
            }
            // ignore checking the expression here cause we already do that in the block code
            StatementType::Put(expression) => {
                if self.loop_depth == 0 && self.block_depth == 0 {
                    self.report_error("Put statement without scope", expression.range());
                }
            }
            StatementType::Break => {}
            StatementType::Expression(_) => {}
        }
    }

    pub fn check_expression(
        &mut self,
        expr: &Expression,
        expected_type: Option<&GiltType>,
    ) -> GiltType {
        let range = expr.range();
        let expr_type = expr.expression_type();
        match expr_type {
            ExpressionType::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = if left.expression_type().is_literal() {
                    self.check_expression(left, expected_type)
                } else {
                    self.check_expression(left, None)
                };
                let right_type = if right.expression_type().is_literal() {
                    self.check_expression(right, expected_type)
                } else {
                    self.check_expression(right, None)
                };

                let common_t = GiltType::get_common_type(&left_type, &right_type);

                match common_t {
                    Some(t) => {
                        // check if operator is logical or arithmetic
                        if operator.is_comparison() {
                            GiltType::Bool
                        } else {
                            t // widened type
                        }
                    }
                    None => {
                        self.report_error(
                            &format!(
                                "Binary operator cannot be applied to {} and {}",
                                left_type, right_type
                            ),
                            range,
                        );
                        GiltType::Unknown
                    }
                }
            }
            ExpressionType::Block(statements) => {
                self.block_depth += 1;
                self.symbols.enter_scope();

                let mut return_type = GiltType::Void;
                let mut seen_return_types: FxHashMap<GiltType, Range> = FxHashMap::default();
                for stmt in statements {
                    if seen_return_types.len() > 0 {
                        self.report_warning(
                            "Unreachable code after returning from block",
                            stmt.range,
                        );
                    }
                    self.check_statement(stmt);
                    if let StatementType::Put(expr) = &stmt.kind {
                        return_type = self.check_expression(expr, expected_type);
                        seen_return_types.insert(return_type.clone(), stmt.range);
                    }
                    if let StatementType::Break = &stmt.kind {
                        seen_return_types.insert(GiltType::Void, stmt.range);
                    }
                }

                self.symbols.exit_scope();
                self.block_depth -= 1;

                if seen_return_types.is_empty() {
                    GiltType::Void
                } else {
                    if seen_return_types.len() <= 1 {
                        return_type
                    } else if seen_return_types.contains_key(&GiltType::Void)
                        && seen_return_types.len() > 1
                    {
                        for (_, range) in &seen_return_types {
                            self.report_error("Can't mix break and put statements", *range);
                        }
                        GiltType::Unknown
                    } else {
                        for (_, range) in &seen_return_types {
                            self.report_error("Multiple return statement types in block", *range);
                        }
                        GiltType::Unknown
                    }
                }
            }
            ExpressionType::Boolean(_) => GiltType::Bool,
            ExpressionType::Identifier(identifier) => {
                if let Some(symbol) = self.symbols.resolve(identifier) {
                    let symbol_type = symbol.symbol_type.clone();
                    if let Some(expected) = expected_type {
                        if symbol_type.coercable_to(expected) {
                            expected.clone()
                        } else {
                            self.report_error("Type mismatch", range);
                            GiltType::Unknown
                        }
                    } else {
                        symbol_type
                    }
                } else {
                    self.report_error("Undefined identifier", range);
                    GiltType::Unknown
                }
            }
            ExpressionType::SignedInteger(int) => {
                if let Some(expected) = expected_type {
                    if expected.signed_int_fits(*int).unwrap_or_else(|err| {
                        self.report_error(&err, range);
                        false
                    }) {
                        // if int fits expected type, return expected type
                        expected.clone()
                    } else {
                        self.report_error(
                            &format!("Number out of range for type {}", expected),
                            range,
                        );
                        GiltType::Unknown
                    }
                } else {
                    GiltType::I32
                }
            }
            ExpressionType::UnsignedInteger(int) => {
                if let Some(expected) = expected_type {
                    if expected.unsigned_int_fits(*int).unwrap_or_else(|err| {
                        self.report_error(&err, range);
                        false
                    }) {
                        expected.clone()
                    } else {
                        self.report_error(
                            &format!("Number out of range for type {}", expected),
                            range,
                        );
                        GiltType::Unknown
                    }
                } else {
                    GiltType::U32
                }
            }
            ExpressionType::Float(_) => {
                if let Some(expected) = expected_type {
                    if expected.is_float() {
                        expected.clone()
                    } else {
                        self.report_error(
                            &format!("Float value does not match expected type {}", expected),
                            range,
                        );
                        GiltType::Unknown
                    }
                } else {
                    GiltType::F32
                }
            }
        }
    }

    fn report_error(&mut self, message: &str, range: Range) {
        self.diagnostics.push(Diagnostic::new_error(message, range));
    }

    fn report_warning(&mut self, message: &str, range: Range) {
        self.diagnostics
            .push(Diagnostic::new_warning(message, range));
    }
}
