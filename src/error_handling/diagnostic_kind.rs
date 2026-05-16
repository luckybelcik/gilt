use crate::semantics::types::GiltType;

#[derive(Debug, Clone)]
pub enum DiagnosticKind {
    // syntax errors
    SyntaxError,
    MissingSemicolon,
    FunctionDeclerationMissingCodeBlock,
    StatementAtTopLevelWhenShouldntBe,

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
    ScopelessBreak,
    ScopelessReturn,
    MixedTerminators,
    MultipleTypesReturned,
    UndefinedIdentifier(String),
    NumberOutOfRangeForType(GiltType),
    SymbolRedeclaration(String),
    NonExhaustiveIfExpression,
    VoidReturnedWhenValueExpected,
    ComplicatedIfCondition,
    IncorrectSymbolType,
    NestedFunction,
    FunctionNotAtTopScope,
    IncorrectArgumentCount { expected: usize, found: usize },
    MismatchedTerminatorsInBinaryExpression,

    // warnings
    UnreachableCode,
}

impl DiagnosticKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SyntaxError => "SyntaxError",
            Self::MissingSemicolon => "MissingSemicolon",
            Self::FunctionDeclerationMissingCodeBlock => "FunctionDefinitionMissingCodeBlock",
            Self::StatementAtTopLevelWhenShouldntBe => "StatementAtTopLevelWhenShouldntBe",

            Self::UTF8ConversionFailed => "UTF8ConversionFailed",
            Self::MissingFieldNamed(_) => "MissingFieldNamed",
            Self::MissingFieldNumbered(_) => "MissingFieldNumbered",
            Self::NonFloatNumberInFloatExpression(_) => "NonFloatNumberInFloatExpression",

            Self::TypeMismatch { .. } => "TypeMismatch",
            Self::UncoercibleType { .. } => "UncoercibleType",
            Self::AssigningToConstant => "AssigningToConstant",
            Self::ScopelessPut => "ScopelessPut",
            Self::ScopelessBreak => "ScopelessBreak",
            Self::ScopelessReturn => "ScopelessReturn",
            Self::MixedTerminators => "MixedTerminators",
            Self::MultipleTypesReturned => "MultipleTypesReturned",
            Self::UndefinedIdentifier(_) => "UndefinedIdentifier",
            Self::NumberOutOfRangeForType(_) => "NumberOutOfRangeForType",
            Self::SymbolRedeclaration(_) => "SymbolRedeclaration",
            Self::NonExhaustiveIfExpression => "NonExhaustiveIfExpression",
            Self::VoidReturnedWhenValueExpected => "VoidReturnedWhenValueExpected",
            Self::ComplicatedIfCondition => "ComplicatedIfCondition",
            Self::IncorrectSymbolType => "IncorrectSymbolType",
            Self::NestedFunction => "NestedFunction",
            Self::FunctionNotAtTopScope => "FunctionNotAtTopScope",
            Self::IncorrectArgumentCount { .. } => "IncorrectArgumentCount",
            Self::MismatchedTerminatorsInBinaryExpression => {
                "MismatchedTerminatorsInBinaryExpression"
            }

            Self::UnreachableCode => "UnreachableCode",
        }
    }
}
