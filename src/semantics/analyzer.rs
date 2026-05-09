use rustc_hash::FxHashMap;
use tree_sitter::Range;

use crate::{
    ast::{
        expression::{Expression, ExpressionType},
        statement::{Statement, StatementType},
    },
    error_handling::{diagnostic::Diagnostic, diagnostic_kind::DiagnosticKind},
    semantics::{
        symbol_table::{Symbol, SymbolTable},
        types::GiltType,
    },
};

pub struct SemanticAnalyzer {
    pub symbols: SymbolTable,
    pub diagnostics: Vec<Diagnostic>,
    loop_depth: usize,
    block_depth: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            diagnostics: Vec::new(),
            loop_depth: 0,
            block_depth: 0,
        }
    }

    pub fn analyze(
        &mut self,
        program: Vec<Statement<()>>,
    ) -> (Vec<Statement<GiltType>>, &Vec<Diagnostic>) {
        let typed_program: Vec<Statement<GiltType>> = program
            .into_iter()
            .map(|stmt| self.check_statement(stmt, None))
            .collect();

        (typed_program, &self.diagnostics)
    }

    pub fn check_statement(
        &mut self,
        stmt: Statement<()>,
        expected_type: Option<&GiltType>,
    ) -> Statement<GiltType> {
        match stmt.kind {
            StatementType::VariableDecl {
                is_const,
                name,
                type_ann,
                value,
            } => {
                let expected_type = if let Some(ex) = &type_ann {
                    Some(&GiltType::from_string(&ex))
                } else {
                    None
                };

                let expr_typed = self.check_expression(value, expected_type);
                let type_ = expr_typed.metadata.clone();

                self.symbols
                    .define(Symbol {
                        name: name.clone(),
                        is_const,
                        symbol_type: type_,
                    })
                    .unwrap_or_else(|err| self.report_error(err, stmt.range, line!()));

                Statement::new(
                    StatementType::VariableDecl {
                        is_const,
                        name,
                        type_ann,
                        value: expr_typed,
                    },
                    stmt.range,
                    GiltType::Void,
                )
            }
            StatementType::Assignment { name, value } => {
                let symbol = self.symbols.resolve(&name).cloned();

                if symbol.is_none() {
                    self.report_error(
                        DiagnosticKind::UndefinedIdentifier(name.clone()),
                        stmt.range,
                        line!(),
                    );
                }

                // we can unwrap the symbol here because we already checked for None above
                let symbol = symbol.unwrap();

                if symbol.is_const {
                    self.report_error(DiagnosticKind::AssigningToConstant, stmt.range, line!());
                }

                let value_typed = self.check_expression(value, Some(&symbol.symbol_type));

                if !value_typed.metadata.coercable_to(&symbol.symbol_type) {
                    self.report_error(
                        DiagnosticKind::UncoercibleType {
                            expected: value_typed.metadata.clone(),
                            found: symbol.symbol_type,
                        },
                        stmt.range,
                        line!(),
                    );
                }

                Statement::new(
                    StatementType::Assignment {
                        name,
                        value: value_typed,
                    },
                    stmt.range,
                    GiltType::Void,
                )
            }
            StatementType::Put(expression) => {
                if self.loop_depth == 0 && self.block_depth == 0 {
                    self.report_error(DiagnosticKind::ScopelessPut, expression.range(), line!());
                }

                let expr_range = expression.range();
                let expr_typed = self.check_expression(expression, expected_type);
                let type_ = expr_typed.metadata.clone();

                Statement::new(StatementType::Put(expr_typed), expr_range, type_)
            }
            StatementType::Break => {
                Statement::new(StatementType::Break, stmt.range, GiltType::Void)
            }
            StatementType::Expression(expression) => {
                let expr_range = expression.range();
                let expr_typed = self.check_expression(expression, expected_type);
                let type_ = expr_typed.metadata.clone();

                Statement::new(StatementType::Expression(expr_typed), expr_range, type_)
            }
        }
    }

    pub fn check_expression(
        &mut self,
        expr: Expression<()>,
        expected_type: Option<&GiltType>,
    ) -> Expression<GiltType> {
        match expr.expression_type {
            ExpressionType::Binary {
                left,
                operator,
                right,
            } => {
                let left_expr = if left.expression_type().is_literal() {
                    self.check_expression(*left, expected_type)
                } else {
                    self.check_expression(*left, None)
                };
                let right_expr = if right.expression_type().is_literal() {
                    self.check_expression(*right, expected_type)
                } else {
                    self.check_expression(*right, None)
                };

                let common_t = GiltType::get_common_type(&left_expr.metadata, &right_expr.metadata);

                match common_t {
                    Some(t) => {
                        // check if operator is logical or arithmetic
                        if operator.is_comparison() {
                            Expression::<GiltType>::new(
                                ExpressionType::Binary {
                                    left: Box::new(left_expr),
                                    operator: operator,
                                    right: Box::new(right_expr),
                                },
                                expr.range,
                                GiltType::Bool,
                            )
                        } else {
                            Expression::<GiltType>::new(
                                ExpressionType::Binary {
                                    left: Box::new(left_expr),
                                    operator: operator,
                                    right: Box::new(right_expr),
                                },
                                expr.range,
                                t,
                            )
                        }
                    }
                    None => {
                        self.report_error(
                            DiagnosticKind::UncoercibleType {
                                expected: left_expr.metadata.clone(),
                                found: right_expr.metadata.clone(),
                            },
                            expr.range,
                            line!(),
                        );
                        Expression::new(
                            ExpressionType::Binary {
                                left: Box::new(left_expr),
                                operator: operator,
                                right: Box::new(right_expr),
                            },
                            expr.range,
                            GiltType::Unknown,
                        )
                    }
                }
            }
            ExpressionType::Block(statements) => {
                self.block_depth += 1;
                self.symbols.enter_scope();

                let mut return_type = GiltType::Void;
                let mut typed_statements: Vec<Statement<GiltType>> = Vec::new();
                let mut seen_return_types: FxHashMap<GiltType, Range> = FxHashMap::default();
                for stmt in statements {
                    let range = stmt.range;

                    if !seen_return_types.is_empty() {
                        self.report_warning(DiagnosticKind::UnreachableCode, range, line!());
                    }

                    let typed_stmt = self.check_statement(stmt, expected_type);

                    match &typed_stmt.kind {
                        StatementType::Put(expr) => {
                            return_type = expr.metadata.clone();
                            seen_return_types.insert(return_type.clone(), range);
                        }
                        StatementType::Break => {
                            seen_return_types.insert(GiltType::Void, range);
                        }
                        _ => {}
                    }

                    typed_statements.push(typed_stmt);
                }

                self.symbols.exit_scope();
                self.block_depth -= 1;

                if seen_return_types.is_empty() {
                    Expression::new(
                        ExpressionType::Block(typed_statements),
                        expr.range,
                        GiltType::Void,
                    )
                } else {
                    if seen_return_types.len() <= 1 {
                        Expression::new(
                            ExpressionType::Block(typed_statements),
                            expr.range,
                            return_type,
                        )
                    } else if seen_return_types.contains_key(&GiltType::Void)
                        && seen_return_types.len() > 1
                    {
                        for (_, range) in &seen_return_types {
                            self.report_error(DiagnosticKind::MixedTerminators, *range, line!());
                        }
                        Expression::new(
                            ExpressionType::Block(typed_statements),
                            expr.range,
                            GiltType::Unknown,
                        )
                    } else {
                        for (_, range) in &seen_return_types {
                            self.report_error(
                                DiagnosticKind::MultipleTypesReturned,
                                *range,
                                line!(),
                            );
                        }
                        Expression::new(
                            ExpressionType::Block(typed_statements),
                            expr.range,
                            GiltType::Unknown,
                        )
                    }
                }
            }
            ExpressionType::Boolean(bool_) => {
                Expression::new(ExpressionType::Boolean(bool_), expr.range, GiltType::Bool)
            }
            ExpressionType::Identifier(identifier) => {
                if let Some(symbol) = self.symbols.resolve(&identifier) {
                    let symbol_type = symbol.symbol_type.clone();
                    if let Some(expected) = expected_type {
                        if symbol_type.coercable_to(expected) {
                            Expression::new(
                                ExpressionType::Identifier(identifier),
                                expr.range,
                                expected.clone(),
                            )
                        } else {
                            self.report_error(
                                DiagnosticKind::UncoercibleType {
                                    expected: expected.clone(),
                                    found: symbol_type,
                                },
                                expr.range,
                                line!(),
                            );
                            Expression::new(
                                ExpressionType::Identifier(identifier),
                                expr.range,
                                GiltType::Unknown,
                            )
                        }
                    } else {
                        Expression::new(
                            ExpressionType::Identifier(identifier),
                            expr.range,
                            symbol_type,
                        )
                    }
                } else {
                    self.report_error(
                        DiagnosticKind::UndefinedIdentifier(identifier.clone()),
                        expr.range,
                        line!(),
                    );
                    Expression::new(
                        ExpressionType::Identifier(identifier),
                        expr.range,
                        GiltType::Unknown,
                    )
                }
            }
            ExpressionType::NegativeInteger(int) => {
                if let Some(expected) = expected_type {
                    if expected.signed_int_fits(int).unwrap_or_else(|err| {
                        self.report_error(err, expr.range, line!());
                        false
                    }) {
                        Expression::new(
                            ExpressionType::NegativeInteger(int),
                            expr.range,
                            expected.clone(),
                        )
                    } else {
                        self.report_error(
                            DiagnosticKind::NumberOutOfRangeForType(expected.clone()),
                            expr.range,
                            line!(),
                        );
                        Expression::new(
                            ExpressionType::NegativeInteger(int),
                            expr.range,
                            GiltType::Unknown,
                        )
                    }
                } else {
                    Expression::new(
                        ExpressionType::NegativeInteger(int),
                        expr.range,
                        GiltType::I32,
                    )
                }
            }
            ExpressionType::PositiveInteger(int) => {
                if let Some(expected) = expected_type {
                    if expected.is_unsigned_integer() {
                        if expected.unsigned_int_fits(int).unwrap_or_else(|err| {
                            self.report_error(err, expr.range, line!());
                            false
                        }) {
                            Expression::new(
                                ExpressionType::PositiveInteger(int),
                                expr.range,
                                expected.clone(),
                            )
                        } else {
                            self.report_error(
                                DiagnosticKind::NumberOutOfRangeForType(expected.clone()),
                                expr.range,
                                line!(),
                            );
                            Expression::new(
                                ExpressionType::PositiveInteger(int),
                                expr.range,
                                GiltType::Unknown,
                            )
                        }
                    } else {
                        if expected.signed_int_fits(int as i128).unwrap_or_else(|err| {
                            self.report_error(err, expr.range, line!());
                            false
                        }) {
                            Expression::new(
                                ExpressionType::PositiveInteger(int),
                                expr.range,
                                expected.clone(),
                            )
                        } else {
                            self.report_error(
                                DiagnosticKind::NumberOutOfRangeForType(expected.clone()),
                                expr.range,
                                line!(),
                            );
                            Expression::new(
                                ExpressionType::PositiveInteger(int),
                                expr.range,
                                GiltType::Unknown,
                            )
                        }
                    }
                } else {
                    Expression::new(
                        ExpressionType::PositiveInteger(int),
                        expr.range,
                        GiltType::U32,
                    )
                }
            }
            ExpressionType::Float(num) => {
                if let Some(expected) = expected_type {
                    if expected.is_float() {
                        Expression::new(ExpressionType::Float(num), expr.range, expected.clone())
                    } else {
                        // internal error because this shouldn't happen
                        self.report_internal_error(
                            DiagnosticKind::NonFloatNumberInFloatExpression(expected.clone()),
                            expr.range,
                            line!(),
                        );
                        Expression::new(ExpressionType::Float(num), expr.range, GiltType::Unknown)
                    }
                } else {
                    Expression::new(ExpressionType::Float(num), expr.range, GiltType::F32)
                }
            }
        }
    }

    fn report_internal_error(&mut self, kind: DiagnosticKind, range: Range, loc: u32) {
        self.diagnostics
            .push(Diagnostic::new_internal_error(kind, range, loc, file!()));
    }

    fn report_error(&mut self, kind: DiagnosticKind, range: Range, loc: u32) {
        self.diagnostics
            .push(Diagnostic::new_error(kind, range, loc, file!()));
    }

    fn report_warning(&mut self, kind: DiagnosticKind, range: Range, loc: u32) {
        self.diagnostics
            .push(Diagnostic::new_warning(kind, range, loc, file!()));
    }
}
