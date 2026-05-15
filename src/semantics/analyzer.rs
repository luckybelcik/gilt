use rustc_hash::FxHashMap;
use tree_sitter::Range;

use crate::{
    ast::{
        expression::{Expression, ExpressionType},
        statement::{Statement, StatementType},
    },
    error_handling::{diagnostic::Diagnostic, diagnostic_kind::DiagnosticKind},
    semantics::{
        expected_type::ExpectedType,
        symbol_table::{FunctionInfo, Symbol, SymbolTable, VariableInfo},
        types::GiltType,
    },
};

enum SemanticError {
    NotFound,
    WrongType,
}

pub struct SemanticAnalyzer {
    pub symbols: SymbolTable,
    pub diagnostics: Vec<Diagnostic>,
    loop_depth: i32,
    block_depth: i32,
    func_depth: i32,
}

impl SemanticAnalyzer {
    pub fn new(save_symbol_history: bool) -> Self {
        Self {
            symbols: SymbolTable::new(save_symbol_history),
            diagnostics: Vec::new(),
            loop_depth: 0,
            block_depth: 0,
            func_depth: 0,
        }
    }

    pub fn analyze(
        &mut self,
        program: Vec<Statement<()>>,
    ) -> (Vec<Statement<GiltType>>, &Vec<Diagnostic>) {
        self.collect_definitions(&program);

        let typed_program: Vec<Statement<GiltType>> = program
            .into_iter()
            .map(|stmt| self.check_statement(stmt, &ExpectedType::Any))
            .collect();

        (typed_program, &self.diagnostics)
    }

    pub fn collect_definitions(&mut self, program: &Vec<Statement>) {
        for stmt in program {
            if let StatementType::FuncDef {
                name,
                parameters,
                return_type,
                ..
            } = &stmt.kind
            {
                let param_types = parameters
                    .iter()
                    .map(|p| (p.name.clone(), GiltType::from_string(&p.type_ann)))
                    .collect();

                let ret_ty = return_type
                    .as_ref()
                    .map(|rt| GiltType::from_string(rt))
                    .unwrap_or(GiltType::Void);

                let symbol = Symbol::Function(FunctionInfo {
                    params: param_types,
                    return_type: ret_ty,
                });

                if let Err(err) = self.symbols.define(symbol, name.clone()) {
                    self.report_error(err, stmt.range, line!());
                }
            }
            // later collect structs or globals here
        }
    }

    pub fn check_statement(
        &mut self,
        stmt: Statement<()>,
        expected_type: &ExpectedType,
    ) -> Statement<GiltType> {
        match stmt.kind {
            StatementType::VarDecl {
                is_const,
                name,
                type_ann,
                value,
            } => {
                let expected_type = if let Some(ex) = &type_ann {
                    ExpectedType::Specific(&GiltType::from_string(&ex))
                } else {
                    ExpectedType::AnyValue
                };

                let expr_typed = self.check_expression(value, &expected_type);
                let type_ = expr_typed.metadata.clone();

                self.symbols
                    .define(
                        Symbol::Variable(VariableInfo {
                            ty: type_,
                            is_const,
                        }),
                        name.clone(),
                    )
                    .unwrap_or_else(|err| self.report_error(err, stmt.range, line!()));

                Statement::new(
                    StatementType::VarDecl {
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
                let var_info = match self.get_variable(&name) {
                    Ok(info) => info,
                    Err(e) => {
                        let kind = match e {
                            SemanticError::NotFound => {
                                DiagnosticKind::UndefinedIdentifier(name.clone())
                            }
                            SemanticError::WrongType => DiagnosticKind::IncorrectSymbolType,
                        };
                        self.report_error(kind, stmt.range, line!());

                        // return failure state
                        return Statement::new(
                            StatementType::Assignment {
                                name,
                                value: self.check_expression(value, &ExpectedType::Any),
                            },
                            stmt.range,
                            GiltType::Unknown,
                        );
                    }
                };

                if var_info.is_const {
                    self.report_error(DiagnosticKind::AssigningToConstant, stmt.range, line!());
                }

                let value_typed =
                    self.check_expression(value, &ExpectedType::Specific(&var_info.ty));

                if !value_typed.metadata.coercable_to(&var_info.ty) {
                    self.report_error(
                        DiagnosticKind::UncoercibleType {
                            expected: var_info.ty.clone(),
                            found: value_typed.metadata.clone(),
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
                    self.report_error(DiagnosticKind::ScopelessPut, stmt.range, line!());
                }

                let expr_typed = self.check_expression(expression, &expected_type);
                let type_ = expr_typed.metadata.clone();

                Statement::new(StatementType::Put(expr_typed), stmt.range, type_)
            }
            StatementType::Break => {
                if self.loop_depth == 0 && self.block_depth == 0 {
                    self.report_error(DiagnosticKind::ScopelessBreak, stmt.range, line!());
                }

                Statement::new(StatementType::Break, stmt.range, GiltType::Void)
            }
            StatementType::Return(maybe_expression) => {
                if self.func_depth == 0 {
                    self.report_error(DiagnosticKind::ScopelessReturn, stmt.range, line!());
                }

                let (expr, type_) = if let Some(expr) = maybe_expression {
                    let expr_typed = self.check_expression(expr, &expected_type);
                    let type_ = expr_typed.metadata.clone();
                    (Some(expr_typed), type_)
                } else {
                    (None, GiltType::Void)
                };

                Statement::new(StatementType::Return(expr), stmt.range, type_)
            }
            StatementType::Expression(expression) => {
                let expr_range = expression.range();
                let expr_typed = self.check_expression(expression, &expected_type);
                let type_ = expr_typed.metadata.clone();

                Statement::new(StatementType::Expression(expr_typed), expr_range, type_)
            }
            StatementType::FuncDef {
                is_public,
                name,
                parameters,
                body,
                return_type: maybe_return_type,
            } => {
                match &body.expression_type {
                    ExpressionType::Block(_) => {}
                    _ => {
                        self.report_error(
                            DiagnosticKind::FunctionDeclerationMissingCodeBlock,
                            stmt.range,
                            line!(),
                        );
                    }
                }

                if self.func_depth > 0 {
                    self.report_error(DiagnosticKind::NestedFunction, stmt.range, line!());
                }

                if self.block_depth > 0 || self.loop_depth > 0 {
                    self.report_error(DiagnosticKind::FunctionNotAtTopScope, stmt.range, line!());
                }

                self.func_depth += 1;
                self.symbols.enter_scope();

                for parameter in &parameters {
                    let parameter_type = GiltType::from_string(&parameter.type_ann);

                    let res = self.symbols.define(
                        Symbol::Variable(VariableInfo {
                            ty: parameter_type,
                            is_const: false,
                        }),
                        parameter.name.clone(),
                    );

                    if let Err(err) = res {
                        self.report_error(err, stmt.range, line!());
                    }
                }

                // we do this so that put statements dont work in functions without a block scope
                // this is necessary because checking block expressions implicity introduces a block scope
                self.block_depth -= 1;
                let body_typed = self.check_expression(body, &expected_type);
                let type_ = body_typed.metadata.clone();
                self.block_depth += 1;

                if let Some(return_type) = &maybe_return_type {
                    let return_type_gilt = GiltType::from_string(return_type);
                    if type_ != return_type_gilt {
                        self.report_error(
                            DiagnosticKind::TypeMismatch {
                                expected: return_type_gilt,
                                found: type_,
                            },
                            stmt.range,
                            line!(),
                        );
                    }
                }

                self.func_depth -= 1;
                self.symbols.exit_scope();

                Statement::new(
                    StatementType::FuncDef {
                        is_public,
                        name,
                        parameters,
                        body: body_typed,
                        return_type: maybe_return_type,
                    },
                    stmt.range,
                    type_,
                )
            }
        }
    }

    pub fn check_expression(
        &mut self,
        expr: Box<Expression<()>>,
        expected_type: &ExpectedType,
    ) -> Box<Expression<GiltType>> {
        let r;
        match expr.expression_type {
            ExpressionType::Binary {
                left,
                operator,
                right,
            } => {
                let left_expr = if left.expression_type().is_literal() {
                    self.check_expression(left, expected_type)
                } else {
                    self.check_expression(left, &ExpectedType::AnyValue)
                };
                let right_expr = if right.expression_type().is_literal() {
                    self.check_expression(right, expected_type)
                } else {
                    self.check_expression(right, &ExpectedType::AnyValue)
                };

                let common_t = GiltType::get_common_type(&left_expr.metadata, &right_expr.metadata);

                match common_t {
                    Some(t) => {
                        // check if operator is logical or arithmetic
                        if operator.is_comparison() {
                            r = Expression::<GiltType>::new(
                                ExpressionType::Binary {
                                    left: left_expr,
                                    operator: operator,
                                    right: right_expr,
                                },
                                expr.range,
                                GiltType::Bool,
                            );
                        } else {
                            r = Expression::<GiltType>::new(
                                ExpressionType::Binary {
                                    left: left_expr,
                                    operator: operator,
                                    right: right_expr,
                                },
                                expr.range,
                                t,
                            );
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
                        r = Expression::new(
                            ExpressionType::Binary {
                                left: left_expr,
                                operator: operator,
                                right: right_expr,
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
                        StatementType::Return(mayb_expr) => {
                            if let Some(expr) = mayb_expr {
                                return_type = expr.metadata.clone();
                                seen_return_types.insert(return_type.clone(), range);
                            }
                        }
                        _ => {}
                    }

                    typed_statements.push(typed_stmt);
                }

                self.symbols.exit_scope();
                self.block_depth -= 1;

                if seen_return_types.is_empty() {
                    r = Expression::new(
                        ExpressionType::Block(typed_statements),
                        expr.range,
                        GiltType::Void,
                    )
                } else {
                    if seen_return_types.len() <= 1 {
                        r = Expression::new(
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
                        r = Expression::new(
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
                        r = Expression::new(
                            ExpressionType::Block(typed_statements),
                            expr.range,
                            GiltType::Unknown,
                        )
                    }
                }

                // sanity check: don't allow r to be void if we expect anyvalue
                if expected_type.nonvoid() && r.metadata == GiltType::Void {
                    self.report_error(
                        DiagnosticKind::VoidReturnedWhenValueExpected,
                        expr.range,
                        line!(),
                    );
                }
            }
            ExpressionType::Boolean(bool_) => {
                r = Expression::new(ExpressionType::Boolean(bool_), expr.range, GiltType::Bool)
            }
            ExpressionType::Identifier(identifier) => {
                let var_info = match self.get_variable(&identifier) {
                    Ok(info) => info,
                    Err(e) => {
                        let kind = match e {
                            SemanticError::NotFound => {
                                DiagnosticKind::UndefinedIdentifier(identifier.clone())
                            }
                            SemanticError::WrongType => DiagnosticKind::IncorrectSymbolType,
                        };
                        self.report_error(kind, expr.range, line!());

                        // return failure state
                        return Box::new(Expression::new(
                            ExpressionType::Identifier(identifier),
                            expr.range,
                            GiltType::Unknown,
                        ));
                    }
                };

                let symbol_type = var_info.ty.clone();
                if let ExpectedType::Specific(expected) = expected_type {
                    if symbol_type.coercable_to(expected) {
                        r = Expression::new(
                            ExpressionType::Identifier(identifier),
                            expr.range,
                            **expected,
                        )
                    } else {
                        self.report_error(
                            DiagnosticKind::UncoercibleType {
                                expected: **expected,
                                found: symbol_type,
                            },
                            expr.range,
                            line!(),
                        );
                        r = Expression::new(
                            ExpressionType::Identifier(identifier),
                            expr.range,
                            GiltType::Unknown,
                        )
                    }
                } else {
                    r = Expression::new(
                        ExpressionType::Identifier(identifier),
                        expr.range,
                        symbol_type,
                    )
                }
            }
            ExpressionType::NegativeInteger(int) => {
                if let ExpectedType::Specific(expected) = expected_type {
                    if expected.signed_int_fits(int).unwrap_or_else(|err| {
                        self.report_error(err, expr.range, line!());
                        false
                    }) {
                        r = Expression::new(
                            ExpressionType::NegativeInteger(int),
                            expr.range,
                            **expected,
                        )
                    } else {
                        self.report_error(
                            DiagnosticKind::NumberOutOfRangeForType(**expected),
                            expr.range,
                            line!(),
                        );
                        r = Expression::new(
                            ExpressionType::NegativeInteger(int),
                            expr.range,
                            GiltType::Unknown,
                        )
                    }
                } else {
                    r = Expression::new(
                        ExpressionType::NegativeInteger(int),
                        expr.range,
                        GiltType::I32,
                    )
                }
            }
            ExpressionType::PositiveInteger(int) => {
                if let ExpectedType::Specific(expected) = expected_type {
                    if expected.is_unsigned_integer() {
                        if expected.unsigned_int_fits(int).unwrap_or_else(|err| {
                            self.report_error(err, expr.range, line!());
                            false
                        }) {
                            r = Expression::new(
                                ExpressionType::PositiveInteger(int),
                                expr.range,
                                **expected,
                            )
                        } else {
                            self.report_error(
                                DiagnosticKind::NumberOutOfRangeForType(**expected),
                                expr.range,
                                line!(),
                            );
                            r = Expression::new(
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
                            r = Expression::new(
                                ExpressionType::PositiveInteger(int),
                                expr.range,
                                **expected,
                            )
                        } else {
                            self.report_error(
                                DiagnosticKind::NumberOutOfRangeForType(**expected),
                                expr.range,
                                line!(),
                            );
                            r = Expression::new(
                                ExpressionType::PositiveInteger(int),
                                expr.range,
                                GiltType::Unknown,
                            )
                        }
                    }
                } else {
                    r = Expression::new(
                        ExpressionType::PositiveInteger(int),
                        expr.range,
                        GiltType::U32,
                    )
                }
            }
            ExpressionType::Float(num) => {
                if let ExpectedType::Specific(expected) = expected_type {
                    if expected.is_float() {
                        r = Expression::new(ExpressionType::Float(num), expr.range, **expected)
                    } else {
                        // internal error because this shouldn't happen
                        self.report_internal_error(
                            DiagnosticKind::NonFloatNumberInFloatExpression(**expected),
                            expr.range,
                            line!(),
                        );
                        r = Expression::new(
                            ExpressionType::Float(num),
                            expr.range,
                            GiltType::Unknown,
                        )
                    }
                } else {
                    r = Expression::new(ExpressionType::Float(num), expr.range, GiltType::F32)
                }
            }
            ExpressionType::If {
                condition,
                consequence,
                alternative,
            } => {
                let cond_typed = self.check_expression(condition, &ExpectedType::AnyValue);

                match &cond_typed.expression_type {
                    ExpressionType::Block(_)
                    | ExpressionType::If {
                        condition: _,
                        consequence: _,
                        alternative: _,
                    } => {
                        self.report_error(
                            DiagnosticKind::ComplicatedIfCondition,
                            expr.range,
                            line!(),
                        );
                    }

                    _ => {}
                }

                if cond_typed.metadata != GiltType::Bool {
                    self.report_error(
                        DiagnosticKind::TypeMismatch {
                            expected: GiltType::Bool,
                            found: cond_typed.metadata.clone(),
                        },
                        expr.range,
                        line!(),
                    );
                }

                let consequence_typed = self.check_expression(consequence, expected_type);
                let consequence_type = consequence_typed.metadata.clone();

                let alternative_typed = if let Some(alt) = alternative {
                    let alt_typed = self.check_expression(alt, expected_type);

                    if consequence_type != alt_typed.metadata {
                        self.report_error(
                            DiagnosticKind::TypeMismatch {
                                expected: consequence_type.clone(),
                                found: alt_typed.metadata.clone(),
                            },
                            expr.range,
                            line!(),
                        );
                    }
                    Some(alt_typed)
                } else {
                    None
                };

                if expected_type.nonvoid() && alternative_typed.is_none() {
                    self.report_error(
                        DiagnosticKind::NonExhaustiveIfExpression,
                        expr.range,
                        line!(),
                    );
                }

                r = Expression::new(
                    ExpressionType::If {
                        condition: cond_typed,
                        consequence: consequence_typed,
                        alternative: alternative_typed,
                    },
                    expr.range,
                    consequence_type,
                );
            }
            ExpressionType::FuncCall { name, arguments } => {
                let func_info = match self.get_function(&name) {
                    Ok(info) => info,
                    Err(e) => {
                        let kind = match e {
                            SemanticError::NotFound => {
                                DiagnosticKind::UndefinedIdentifier(name.clone())
                            }
                            SemanticError::WrongType => DiagnosticKind::IncorrectSymbolType,
                        };
                        self.report_error(kind, expr.range, line!());

                        // return failure state
                        return Box::new(Expression::new(
                            ExpressionType::FuncCall {
                                name,
                                arguments: vec![],
                            },
                            expr.range,
                            GiltType::Unknown,
                        ));
                    }
                };

                if expected_type.is_specific()
                    && !func_info
                        .return_type
                        .coercable_to(expected_type.as_specific())
                {
                    self.report_error(
                        DiagnosticKind::UncoercibleType {
                            expected: expected_type.as_specific().clone(),
                            found: func_info.return_type.clone(),
                        },
                        expr.range,
                        line!(),
                    );

                    return Box::new(Expression::new(
                        ExpressionType::FuncCall {
                            name,
                            arguments: vec![],
                        },
                        expr.range,
                        GiltType::Unknown,
                    ));
                }

                let param_types = func_info
                    .params
                    .iter()
                    .map(|(_, ty)| ty)
                    .collect::<Vec<&GiltType>>();

                if param_types.len() != arguments.len() {
                    self.report_error(
                        DiagnosticKind::IncorrectArgumentCount {
                            expected: param_types.len(),
                            found: arguments.len(),
                        },
                        expr.range,
                        line!(),
                    );

                    return Box::new(Expression::new(
                        ExpressionType::FuncCall {
                            name,
                            arguments: vec![],
                        },
                        expr.range,
                        GiltType::Unknown,
                    ));
                }

                let mut typed_arguments = vec![];
                let mut i = 0;
                for expr in arguments {
                    let typed_expr = self
                        .check_expression(Box::new(expr), &ExpectedType::Specific(param_types[i]));

                    typed_arguments.push(*typed_expr);
                    i += 1;
                }

                r = Expression::new(
                    ExpressionType::FuncCall {
                        name,
                        arguments: typed_arguments,
                    },
                    expr.range,
                    func_info.return_type,
                )
            }
        }

        Box::new(r)
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

    fn get_variable(&self, name: &str) -> Result<VariableInfo, SemanticError> {
        match self.symbols.resolve(name) {
            Some(Symbol::Variable(v)) => Ok(v.clone()),
            Some(_) => Err(SemanticError::WrongType),
            None => Err(SemanticError::NotFound),
        }
    }

    fn get_function(&self, name: &str) -> Result<FunctionInfo, SemanticError> {
        match self.symbols.resolve(name) {
            Some(Symbol::Function(f)) => Ok(f.clone()),
            Some(_) => Err(SemanticError::WrongType),
            None => Err(SemanticError::NotFound),
        }
    }
}
