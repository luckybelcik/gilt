use tree_sitter::Node;

use crate::error_handling::{diagnostic::Diagnostic, diagnostic_kind::DiagnosticKind};

pub fn validate_syntax(node: &Node) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut cursor = node.walk();

    // Perform a depth-first search for garbage in the tree
    let mut stack = vec![node.clone()];
    while let Some(current) = stack.pop() {
        if current.is_error() {
            diagnostics.push(Diagnostic::new_syntax_error(
                DiagnosticKind::SyntaxError,
                current.range(),
                line!(),
                file!(),
            ));
        }

        if current.is_missing() {
            diagnostics.push(Diagnostic::new_syntax_error(
                DiagnosticKind::MissingSemicolon,
                current.range(),
                line!(),
                file!(),
            ));
        }

        for child in current.children(&mut cursor) {
            stack.push(child);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
