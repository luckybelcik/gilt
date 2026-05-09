use crate::semantics::types::GiltType;

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

    pub fn confirmed_nonvoid(&self) -> bool {
        match self {
            ExpectedType::Any => false,
            ExpectedType::Specific(_) => true,
            ExpectedType::AnyValue => true,
        }
    }
}
