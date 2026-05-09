use crate::semantics::types::GiltType;

#[derive(Debug, Clone)]
pub enum DiagnosticKind {
    // internal errors
    UTF8ConversionFailed,
    MissingFieldNamed(String),
    MissingFieldNumbered(u32),
    NonFloatNumberInFloatExpression(GiltType),

    // errors
    TypeMismatch { expected: GiltType, found: GiltType },
    UncoercibleType { expected: GiltType, found: GiltType },
    AssigningToConstant,
    ScopelessPut,
    MixedTerminators,
    MultipleTypesReturned,
    UndefinedIdentifier(String),
    NumberOutOfRangeForType(GiltType),
    VariableRedeclaration(String),

    // warnings
    UnreachableCode,
}

impl DiagnosticKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::UTF8ConversionFailed => "UTF8ConversionFailed",
            Self::MissingFieldNamed(_) => "MissingFieldNamed",
            Self::MissingFieldNumbered(_) => "MissingFieldNumbered",
            Self::NonFloatNumberInFloatExpression(_) => "NonFloatNumberInFloatExpression",
            Self::TypeMismatch { .. } => "TypeMismatch",
            Self::UncoercibleType { .. } => "UncoercibleType",
            Self::AssigningToConstant => "AssigningToConstant",
            Self::ScopelessPut => "ScopelessPut",
            Self::MixedTerminators => "MixedTerminators",
            Self::MultipleTypesReturned => "MultipleTypesReturned",
            Self::UndefinedIdentifier(_) => "UndefinedIdentifier",
            Self::NumberOutOfRangeForType(_) => "NumberOutOfRangeForType",
            Self::VariableRedeclaration(_) => "VariableRedeclaration",
            Self::UnreachableCode => "UnreachableCode",
        }
    }
}
