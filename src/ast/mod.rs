//! AST for the Dahlia language
//!
//! This module defines the abstract syntax tree (AST) for the Dahlia language.

pub mod operator;
pub mod types;

use crate::ast::operator::{BinaryOp, UnaryOp};
use crate::ast::types::Type;

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    UInt(u64),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Fixed-size arrays
    Array {
        elements: Vec<Literal>,
        element_type: Option<Type>,
    },
}

/// An identifier paired with a optional type
#[derive(Debug, Clone, PartialEq)]
pub struct TypedIdentifier {
    pub name: String,
    pub type_: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DottedName {
    pub base: TypedIdentifier,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),

    // An identifier paired with an optional type
    Identifier(TypedIdentifier),

    // A recursively defined dotted name, e.g. `foo.bar.baz`
    Dotted(DottedName),

    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    // Pointer dereference operator: *ptr
    Dereference {
        pointer: Box<Expr>,
    },

    // Pointer address-of operator: &var
    AddressOf {
        expr: Box<Expr>,
    },

    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // An expression in place of a statement (function calls)
    Expr(Expr),

    VarDeclaration {
        name: TypedIdentifier,
        value: Expr,
    },

    ConstDeclaration {
        name: TypedIdentifier,
        value: Expr,
    },

    Assign {
        target: Expr,
        value: Expr,
    },

    For {
        iterator: TypedIdentifier,
        iterable: Expr,
        body: Block,
    },

    While {
        condition: Expr,
        body: Block,
    },

    Return {
        value: Option<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToplevelStmt {
    // A statement *can* also be a valid top-level statement
    Stmt(Stmt),

    FnDeclaration {
        name: TypedIdentifier,
        params: Vec<TypedIdentifier>,
        body: Block,
    },

    StructDeclaration {
        name: TypedIdentifier,
        fields: Vec<TypedIdentifier>,
    },
}

/// The root of the AST, representing a complete program
#[derive(Debug, Clone, PartialEq)]
pub struct AST {
    pub statements: Vec<ToplevelStmt>,
    pub comments: Vec<String>,
}
