use std::sync::{LazyLock, Mutex};

use tree_sitter::Range;

use crate::error_handling::diagnostic::Diagnostic;

pub type StaticDiagnosticRegistry = LazyLock<Mutex<DiagnosticRegistry>>;

static DIAGNOSTIC_REGISTRY: StaticDiagnosticRegistry =
    LazyLock::new(|| Mutex::new(DiagnosticRegistry::new()));

pub fn get_diagnostic_registry() -> &'static StaticDiagnosticRegistry {
    &DIAGNOSTIC_REGISTRY
}

pub fn register_internal_error(message: &str, range: Range) {
    DIAGNOSTIC_REGISTRY
        .lock()
        .unwrap()
        .register(Diagnostic::new_internal_error(message, range));
}

pub fn register_error(message: &str, range: Range) {
    DIAGNOSTIC_REGISTRY
        .lock()
        .unwrap()
        .register(Diagnostic::new_error(message, range));
}

pub fn register_warning(message: &str, range: Range) {
    DIAGNOSTIC_REGISTRY
        .lock()
        .unwrap()
        .register(Diagnostic::new_warning(message, range));
}

pub fn register_info(message: &str, range: Range) {
    DIAGNOSTIC_REGISTRY
        .lock()
        .unwrap()
        .register(Diagnostic::new_info(message, range));
}

pub fn get_diagnostics() -> Vec<Diagnostic> {
    DIAGNOSTIC_REGISTRY.lock().unwrap().diagnostics().to_vec()
}

pub struct DiagnosticRegistry {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticRegistry {
    pub fn new() -> Self {
        DiagnosticRegistry {
            diagnostics: Vec::new(),
        }
    }

    pub fn register(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
