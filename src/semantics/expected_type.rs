use crate::semantics::types::GiltType;

#[derive(PartialEq)]
pub enum ExpectedType<'a> {
    // any includes void, unknown, etc
    Any,

    // specific type
    Specific(&'a GiltType),

    // any non-void
    AnyValue,
}

impl<'a> ExpectedType<'a> {
    pub fn is_specific(&self) -> bool {
        match self {
            ExpectedType::Any => false,
            ExpectedType::Specific(_) => true,
            ExpectedType::AnyValue => false,
        }
    }

    pub fn as_specific(&self) -> &GiltType {
        match self {
            ExpectedType::Any => panic!("Expected specific type, found any"),
            ExpectedType::Specific(t) => t,
            ExpectedType::AnyValue => panic!("Expected specific type, found any value"),
        }
    }

    pub fn nonvoid(&self) -> bool {
        match self {
            ExpectedType::Any => false,
            ExpectedType::Specific(_) => true,
            ExpectedType::AnyValue => true,
        }
    }
}
