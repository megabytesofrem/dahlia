//! AST for the Dahlia language
//!
//! This module defines the abstract syntax tree (AST) for the Dahlia language.

use crate::lexer::TokenKind;

/// The type of a variable, function, or expression
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
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
    pub fn from_token_kind(kind: &TokenKind) -> Option<Type> {
        match kind {
            TokenKind::U8 => Some(Type::U8),
            TokenKind::U16 => Some(Type::U16),
            TokenKind::U32 => Some(Type::U32),
            TokenKind::U64 => Some(Type::U64),
            TokenKind::I8 => Some(Type::I8),
            TokenKind::I16 => Some(Type::I16),
            TokenKind::I32 => Some(Type::I32),
            TokenKind::I64 => Some(Type::I64),
            TokenKind::F32 => Some(Type::F32),
            TokenKind::F64 => Some(Type::F64),
            TokenKind::BoolType => Some(Type::Bool),
            TokenKind::CharType => Some(Type::Char),
            TokenKind::StrType => Some(Type::Str),
            TokenKind::VoidType => Some(Type::Void),
            _ => None,
        }
    }

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

    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array { .. })
    }

    pub fn is_primitive(&self) -> bool {
        !self.is_boxed()
    }
}
