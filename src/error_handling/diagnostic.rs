use tree_sitter::Range;

#[derive(Debug, Clone)]
pub enum DiagnosticKind {
    InternalError,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    message: String,
    span: Range,
}

impl Diagnostic {
    pub fn new_internal_error(message: &str, span: Range) -> Self {
        Diagnostic {
            kind: DiagnosticKind::InternalError,
            message: message.to_string(),
            span,
        }
    }

    pub fn new_error(message: &str, span: Range) -> Self {
        Diagnostic {
            kind: DiagnosticKind::Error,
            message: message.to_string(),
            span,
        }
    }

    pub fn new_warning(message: &str, span: Range) -> Self {
        Diagnostic {
            kind: DiagnosticKind::Warning,
            message: message.to_string(),
            span,
        }
    }

    pub fn new_info(message: &str, span: Range) -> Self {
        Diagnostic {
            kind: DiagnosticKind::Info,
            message: message.to_string(),
            span,
        }
    }
}
