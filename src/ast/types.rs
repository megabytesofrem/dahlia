//! AST for the Dahlia language
//!
//! This module defines the abstract syntax tree (AST) for the Dahlia language.

/// The type of a variable, function, or expression
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Char,
    Str,
    Void,

    Pointer(Box<Type>),

    Array {
        element_type: Box<Type>,
        size: usize,
    },
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::I32
                | Type::I64
                | Type::F32
                | Type::F64
        )
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64)
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, Type::I32 | Type::I64)
    }

    // Boxed types require managing their memory lifecycles, so they are treated specially
    // from primitive types.
    pub fn is_boxed(&self) -> bool {
        matches!(self, Type::Pointer(_) | Type::Array { .. })
    }

    pub fn is_primitive(&self) -> bool {
        !self.is_boxed()
    }
}
