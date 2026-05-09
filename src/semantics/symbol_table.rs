use rustc_hash::FxHashMap;

use crate::{error_handling::diagnostic_kind::DiagnosticKind, semantics::types::GiltType};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub is_const: bool,
    pub symbol_type: GiltType,
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

    pub fn define(&mut self, symbol: Symbol) -> Result<(), DiagnosticKind> {
        // if exists anywhere already, return error (no shadowing)
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&symbol.name) {
                return Err(DiagnosticKind::VariableRedeclaration(symbol.name));
            }
        }

        // otherwise, insert into top scope
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.insert(symbol.name.clone(), symbol);
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
