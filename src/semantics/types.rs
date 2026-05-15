use std::fmt::Display;

use crate::error_handling::diagnostic_kind::DiagnosticKind;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GiltType {
    // ints
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    Usize,
    Isize,

    // floats
    F32,
    F64,

    // other
    Bool,
    Char,

    // misc
    Void,

    // comptime
    Unknown,
}

impl GiltType {
    pub fn from_string(s: &str) -> GiltType {
        match s {
            "u8" => GiltType::U8,
            "i8" => GiltType::I8,
            "u16" => GiltType::U16,
            "i16" => GiltType::I16,
            "u32" => GiltType::U32,
            "i32" => GiltType::I32,
            "u64" => GiltType::U64,
            "i64" => GiltType::I64,
            "u128" => GiltType::U128,
            "i128" => GiltType::I128,
            "usize" => GiltType::Usize,
            "isize" => GiltType::Isize,

            "f32" => GiltType::F32,
            "f64" => GiltType::F64,

            "bool" => GiltType::Bool,
            "char" => GiltType::Char,

            _ => GiltType::Unknown,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128 => true,
            _ => false,
        }
    }

    pub fn is_signed_integer(&self) -> bool {
        match self {
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 => true,
            _ => false,
        }
    }

    pub fn is_unsigned_integer(&self) -> bool {
        match self {
            Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::U128 => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::F32 | Self::F64 => true,
            _ => false,
        }
    }

    pub fn signed_int_fits(&self, int: i128) -> Result<bool, DiagnosticKind> {
        match self {
            Self::I8 => Ok(int >= i128::from(i8::MIN) && int <= i128::from(i8::MAX)),
            Self::I16 => Ok(int >= i128::from(i16::MIN) && int <= i128::from(i16::MAX)),
            Self::I32 => Ok(int >= i128::from(i32::MIN) && int <= i128::from(i32::MAX)),
            Self::I64 => Ok(int >= i128::from(i64::MIN) && int <= i128::from(i64::MAX)),
            Self::I128 => Ok(int >= i128::from(i128::MIN) && int <= i128::from(i128::MAX)),
            _ => Err(DiagnosticKind::TypeMismatch {
                expected: self.clone(),
                found: GiltType::I128,
            }),
        }
    }

    pub fn unsigned_int_fits(&self, int: u128) -> Result<bool, DiagnosticKind> {
        match self {
            Self::U8 => Ok(int <= u128::from(u8::MAX)),
            Self::U16 => Ok(int <= u128::from(u16::MAX)),
            Self::U32 => Ok(int <= u128::from(u32::MAX)),
            Self::U64 => Ok(int <= u128::from(u64::MAX)),
            Self::U128 => Ok(int <= u128::MAX),
            _ => Err(DiagnosticKind::TypeMismatch {
                expected: self.clone(),
                found: GiltType::U128,
            }),
        }
    }

    pub fn get_common_type(left: &GiltType, right: &GiltType) -> Option<GiltType> {
        if left == right {
            return Some(left.clone());
        }

        // if one can coerce into the other, the "target" is the common type
        if left.coercable_to(right) {
            Some(right.clone())
        } else if right.coercable_to(left) {
            Some(left.clone())
        } else {
            None // incompatible :(
        }
    }

    pub fn coercable_to(&self, other: &GiltType) -> bool {
        match &self {
            Self::U8 => match &other {
                Self::U8
                | Self::U16
                | Self::I16
                | Self::U32
                | Self::I32
                | Self::U64
                | Self::I64
                | Self::U128
                | Self::I128 => true,
                _ => false,
            },
            Self::I8 => match &other {
                Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 => true,
                _ => false,
            },
            Self::U16 => match &other {
                Self::U16
                | Self::U32
                | Self::I32
                | Self::U64
                | Self::I64
                | Self::U128
                | Self::I128 => true,
                _ => false,
            },
            Self::I16 => match &other {
                Self::I16 | Self::I32 | Self::I64 | Self::I128 => true,
                _ => false,
            },
            Self::U32 => match &other {
                Self::U32 | Self::U64 | Self::I64 | Self::U128 | Self::I128 => true,
                _ => false,
            },
            Self::I32 => match &other {
                Self::I32 | Self::I16 | Self::I64 | Self::I128 => true,
                _ => false,
            },
            Self::U64 => match &other {
                Self::U64 | Self::U128 | Self::I128 => true,
                _ => false,
            },
            Self::I64 => match &other {
                Self::I64 | Self::I128 => true,
                _ => false,
            },
            Self::U128 => match &other {
                Self::U128 => true,
                _ => false,
            },
            Self::I128 => match &other {
                Self::I128 => true,
                _ => false,
            },
            Self::Usize => match &other {
                Self::Usize => true,
                _ => false,
            },
            Self::Isize => match &other {
                Self::Isize => true,
                _ => false,
            },

            Self::F32 => match &other {
                Self::F32 | Self::F64 => true,
                _ => false,
            },
            Self::F64 => match &other {
                Self::F64 => true,
                _ => false,
            },

            Self::Bool => match &other {
                Self::Bool => true,
                _ => false,
            },
            Self::Char => match &other {
                Self::Char => true,
                _ => false,
            },

            Self::Void => match &other {
                Self::Void => true,
                _ => false,
            },

            Self::Unknown => match &other {
                Self::Unknown => true,
                _ => false,
            },
        }
    }
}

impl Display for GiltType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GiltType::U8 => write!(f, "u8"),
            GiltType::I8 => write!(f, "i8"),
            GiltType::U16 => write!(f, "u16"),
            GiltType::I16 => write!(f, "i16"),
            GiltType::U32 => write!(f, "u32"),
            GiltType::I32 => write!(f, "i32"),
            GiltType::U64 => write!(f, "u64"),
            GiltType::I64 => write!(f, "i64"),
            GiltType::U128 => write!(f, "u128"),
            GiltType::I128 => write!(f, "i128"),
            GiltType::Usize => write!(f, "usize"),
            GiltType::Isize => write!(f, "isize"),

            GiltType::F32 => write!(f, "f32"),
            GiltType::F64 => write!(f, "f64"),

            GiltType::Bool => write!(f, "bool"),
            GiltType::Char => write!(f, "char"),

            GiltType::Void => write!(f, "void"),

            GiltType::Unknown => write!(f, "unknown"),
        }
    }
}
