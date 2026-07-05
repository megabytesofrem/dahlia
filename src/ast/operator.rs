//! AST for the Dahlia language
//!
//! This module defines the abstract syntax tree (AST) for the Dahlia language.

use crate::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Star,
    Slash,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    DoubleEqual,
    BangEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl TokenKind {
    /// Convert a TokenKind to a BinaryOp, if this is a valid operator token.
    pub fn to_binary_op(&self) -> Option<BinaryOp> {
        match self {
            TokenKind::Plus => Some(BinaryOp::Plus),
            TokenKind::Minus => Some(BinaryOp::Minus),
            TokenKind::Star => Some(BinaryOp::Star),
            TokenKind::Slash => Some(BinaryOp::Slash),
            TokenKind::PlusEquals => Some(BinaryOp::PlusEquals),
            TokenKind::MinusEquals => Some(BinaryOp::MinusEquals),
            TokenKind::StarEquals => Some(BinaryOp::StarEquals),
            TokenKind::SlashEquals => Some(BinaryOp::SlashEquals),
            TokenKind::DoubleEqual => Some(BinaryOp::DoubleEqual),
            TokenKind::BangEqual => Some(BinaryOp::BangEqual),
            TokenKind::LessThan => Some(BinaryOp::LessThan),
            TokenKind::LessEqual => Some(BinaryOp::LessEqual),
            TokenKind::GreaterThan => Some(BinaryOp::GreaterThan),
            TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
            _ => None,
        }
    }

    /// Convert a TokenKind to a UnaryOp, if this is a valid operator token.
    pub fn to_unary_op(&self) -> Option<UnaryOp> {
        match self {
            TokenKind::Minus => Some(UnaryOp::Negate),
            TokenKind::Bang => Some(UnaryOp::Not),
            _ => None,
        }
    }
}
