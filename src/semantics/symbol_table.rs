use rustc_hash::FxHashMap;

use crate::{error_handling::diagnostic_kind::DiagnosticKind, semantics::types::GiltType};

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub ty: GiltType,
    pub is_const: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub params: Vec<(String, GiltType)>,
    pub return_type: GiltType,
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(VariableInfo),
    Function(FunctionInfo),
}

impl Symbol {
    pub fn as_variable(&self) -> Option<&VariableInfo> {
        if let Self::Variable(info) = self {
            Some(info)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&FunctionInfo> {
        if let Self::Function(info) = self {
            Some(info)
        } else {
            None
        }
    }
}

pub struct SymbolTable {
    // each entry in the vec is a new scope level
    scopes: Vec<FxHashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![FxHashMap::default()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, symbol: Symbol, name: String) -> Result<(), DiagnosticKind> {
        // if exists anywhere already, return error (no shadowing)
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name) {
                return Err(DiagnosticKind::SymbolRedeclaration(name.clone()));
            }
        }

        // otherwise,
        // insert into top scope
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.insert(name, symbol);

        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        // iter from inside out
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }
}
