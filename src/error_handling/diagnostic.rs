use tree_sitter::Range;

use crate::error_handling::diagnostic_kind::DiagnosticKind;

#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    InternalError,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub kind: DiagnosticKind,
    pub span: Range,
    pub loc: u32,
    pub file: String,
}

impl Diagnostic {
    pub fn new_internal_error(kind: DiagnosticKind, span: Range, loc: u32, file: &str) -> Self {
        Diagnostic {
            severity: DiagnosticSeverity::InternalError,
            kind,
            span,
            loc,
            file: file.to_string(),
        }
    }

    pub fn new_error(kind: DiagnosticKind, span: Range, loc: u32, file: &str) -> Self {
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            kind,
            span,
            loc,
            file: file.to_string(),
        }
    }

    pub fn new_warning(kind: DiagnosticKind, span: Range, loc: u32, file: &str) -> Self {
        Diagnostic {
            severity: DiagnosticSeverity::Warning,
            kind,
            span,
            loc,
            file: file.to_string(),
        }
    }

    pub fn new_info(kind: DiagnosticKind, span: Range, loc: u32, file: &str) -> Self {
        Diagnostic {
            severity: DiagnosticSeverity::Info,
            kind,
            span,
            loc,
            file: file.to_string(),
        }
    }
}
