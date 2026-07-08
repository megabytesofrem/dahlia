use crate::{
    ast::{Expr, Literal, TypedIdentifier, types::Type},
    lexer::{Lexer, Token, TokenKind},
};

use std::iter::Peekable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 10,
    LogicalOr = 20,
    LogicalAnd = 30,
    Equality = 40,
    Comparison = 50,
    Term = 60,
    Factor = 70,
    Unary = 80,
    Call = 90,
    Primary = 100,
}

impl Precedence {
    pub fn from_token_kind(kind: &TokenKind) -> Precedence {
        match kind {
            // Adjust according to your language's grammar rules
            TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::StarEquals
            | TokenKind::SlashEquals => Precedence::Assignment,
            TokenKind::Or => Precedence::LogicalOr,
            TokenKind::And => Precedence::LogicalAnd,
            TokenKind::DoubleEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::LessThan
            | TokenKind::LessEqual
            | TokenKind::GreaterThan
            | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::LParen => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Peekable<Lexer<'a>>) -> Self {
        Self { lexer, position: 0 }
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.lexer.peek()
    }

    fn next(&mut self) -> Option<Token<'a>> {
        let token = self.lexer.next();
        if let Some(ref t) = token {
            self.position = t.span.end;
        }
        token
    }

    /// Expect a specific token kind and consume it if found, otherwise return an error.
    fn expect(&mut self, token_kind: TokenKind) -> Result<Token<'a>, String> {
        match self.next() {
            Some(token) if token.kind == token_kind => Ok(token),

            // Unexpected token or end of input
            Some(token) => Err(format!(
                "Expected token {:?} but found {:?} at position {}",
                token_kind, token.kind, self.position
            )),
            None => Err(format!(
                "Expected token {:?} but found end of input at position {}",
                token_kind, self.position
            )),
        }
    }

    /// `Parser::expect` but generalized to work on any of a set of token kinds.
    fn any_of(&mut self, token_kinds: &[TokenKind]) -> Result<Token<'a>, String> {
        match self.next() {
            Some(token) if token_kinds.contains(&token.kind) => Ok(token),

            // Unexpected token or end of input
            Some(token) => Err(format!(
                "Expected one of {:?} but found {:?} at position {}",
                token_kinds, token.kind, self.position
            )),
            None => Err(format!(
                "Expected one of {:?} but found end of input at position {}",
                token_kinds, self.position
            )),
        }
    }

    /// `Parser::expect` but generalized to work on none of a set of token kinds.
    /// Inverse function of `Parser::any_of`.
    fn none_of(&mut self, token_kinds: &[TokenKind]) -> Result<Token<'a>, String> {
        match self.next() {
            Some(token) if !token_kinds.contains(&token.kind) => Ok(token),

            // Unexpected token or end of input
            Some(token) => Err(format!(
                "Expected none of {:?} but found {:?} at position {}",
                token_kinds, token.kind, self.position
            )),
            None => Err(format!(
                "Expected none of {:?} but found end of input at position {}",
                token_kinds, self.position
            )),
        }
    }

    // Parser Rules
    // ----------------
    // NB: The parser rules are implemented following the language grammar (see GRAMMAR.md).
    // Each rule is represented as a function in a recursive-descent parser
    //

    fn identifier(&mut self) -> Result<String, String> {
        self.expect(TokenKind::Ident)
            .map(|token| token.lexeme.to_string())
    }

    fn typed_identifier(&mut self) -> Result<(String, Option<Type>), String> {
        let name = self.identifier()?;
        let type_ = if let Some(token) = self.peek() {
            if token.kind == TokenKind::Colon {
                self.next(); // Consume the colon
                Some(self.parse_type()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok((name, type_))
    }

    fn numeric_literal(&mut self) -> Result<String, String> {
        self.any_of(&[TokenKind::IntLit, TokenKind::FloatLit])
            .map(|token| token.lexeme.to_string())
    }

    fn bool_literal(&mut self) -> Result<String, String> {
        self.expect(TokenKind::BoolLit)
            .map(|token| token.lexeme.to_string())
    }

    fn array_literal(&mut self) -> Result<Vec<String>, String> {
        self.expect(TokenKind::LBracket)?;
        let mut elements = Vec::new();

        let prev_position = self.position;

        while let Some(token) = self.peek() {
            if token.kind == TokenKind::RBracket {
                break;
            }

            // Parse the next element
            let element = self.numeric_literal()?;
            elements.push(element);

            // Expect a comma or closing bracket
            match self.peek() {
                Some(token) if token.kind == TokenKind::Comma => {
                    self.next(); // Consume the comma
                }
                Some(token) if token.kind == TokenKind::RBracket => break,
                _ => {
                    return Err(format!(
                        "Expected ',' or ']' but found {:?} at position {}",
                        self.peek().map(|t| &t.kind),
                        prev_position
                    ));
                }
            }
        }

        self.expect(TokenKind::RBracket)?;
        Ok(elements)
    }

    // Type Parsing
    // ----------------
    fn base_type(&mut self) -> Result<Type, String> {
        let type_token = self.any_of(&[
            TokenKind::U8,
            TokenKind::U16,
            TokenKind::U32,
            TokenKind::U64,
            TokenKind::I8,
            TokenKind::I16,
            TokenKind::I32,
            TokenKind::I64,
            TokenKind::F32,
            TokenKind::F64,
            TokenKind::CharType,
            TokenKind::StrType,
            TokenKind::BoolType,
            TokenKind::VoidType,
        ])?;

        match type_token.kind {
            TokenKind::U8 => Ok(Type::U8),
            TokenKind::U16 => Ok(Type::U16),
            TokenKind::U32 => Ok(Type::U32),
            TokenKind::U64 => Ok(Type::U64),
            TokenKind::I8 => Ok(Type::I8),
            TokenKind::I16 => Ok(Type::I16),
            TokenKind::I32 => Ok(Type::I32),
            TokenKind::I64 => Ok(Type::I64),
            TokenKind::F32 => Ok(Type::F32),
            TokenKind::F64 => Ok(Type::F64),
            TokenKind::CharType => Ok(Type::Char),
            TokenKind::StrType => Ok(Type::Str),
            TokenKind::BoolType => Ok(Type::Bool),
            TokenKind::VoidType => Ok(Type::Void),
            _ => Err(format!(
                "Expected a base type but found {:?} at position {}",
                type_token.kind, self.position
            )),
        }
    }

    fn pointer_type(&mut self) -> Result<Type, String> {
        self.expect(TokenKind::Star)?;
        let base_type = self.parse_type()?;
        Ok(Type::Pointer(Box::new(base_type)))
    }

    fn array_type(&mut self) -> Result<Type, String> {
        let element_type = self.parse_type()?;
        self.expect(TokenKind::LBracket)?;
        let size_token = self.expect(TokenKind::IntLit)?;
        let size = size_token.lexeme.parse::<usize>().map_err(|_| {
            format!(
                "Expected an array size but found {:?} at position {}",
                size_token.lexeme, self.position
            )
        })?;
        self.expect(TokenKind::RBracket)?;

        Ok(Type::Array {
            element_type: Box::new(element_type),
            size,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let prev_position = self.position;

        if let Some(token) = self.peek() {
            let token_kind_type = Type::from_token_kind(&token.kind);

            match token.clone().kind {
                TokenKind::Star => self.pointer_type(),

                // Primitive types (u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, bool, char, str, void)
                _primitive if token_kind_type.clone().map(|t| t.is_primitive()) == Some(true) => {
                    self.base_type()
                }

                // Array types (e.g., i32[3], f32[10], str[])
                _array if token_kind_type.clone().map(|t| t.is_array()) == Some(true) => {
                    self.array_type()
                }

                _ => Err(format!(
                    "Expected a type but found {:?} at position {}",
                    token.kind, prev_position
                )),
            }
        } else {
            Err(format!(
                "Expected a type but found end of input at position {}",
                prev_position
            ))
        }
    }

    // Expressions
    // ----------------
    // Our Pratt parser that climbs the precedence levels, one by one

    /// Parse an expression with a default precedence of `Precedence::None`
    pub fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_expression_impl(Precedence::None)
    }

    /// Parse an expression with a given precedence level
    fn parse_expression_impl(&mut self, prec: Precedence) -> Result<Expr, String> {
        let mut lhs = self.parse_prefix()?;

        while let Some(next_token) = self.peek() {
            let next_precedence = Precedence::from_token_kind(&next_token.kind);

            // If the next tokens precedence is less than or equal to the current precedence,
            // we stop parsing and return the current lhs expression.
            if (next_precedence as u8) <= (prec as u8) {
                break;
            }

            lhs = self.parse_infix(lhs)?;
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let token = self
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        match token.kind {
            TokenKind::IntLit => {
                let value = token
                    .lexeme
                    .parse::<i64>()
                    .map_err(|_| "Invalid integer literal".to_string())?;

                Ok(Expr::Literal(Literal::Int(value)))
            }
            TokenKind::FloatLit => {
                let value = token
                    .lexeme
                    .parse::<f64>()
                    .map_err(|_| "Invalid float literal".to_string())?;

                Ok(Expr::Literal(Literal::Float(value)))
            }
            TokenKind::BoolLit => {
                let value = token.lexeme == "true";
                Ok(Expr::Literal(Literal::Bool(value)))
            }
            TokenKind::StringLit => {
                let value = token.lexeme.to_string();
                Ok(Expr::Literal(Literal::String(value)))
            }
            TokenKind::Ident => {
                let name = token.lexeme.to_string();
                let identifier = TypedIdentifier {
                    name,
                    type_: None, // Type information can be added later if needed
                };

                Ok(Expr::Identifier(identifier))
            }
            TokenKind::LParen => {
                let expr = self.parse_expression_impl(Precedence::None)?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }

            TokenKind::Minus | TokenKind::Bang | TokenKind::Ampersand | TokenKind::Star => {
                let op = token.kind.clone().to_unary_op().ok_or_else(|| {
                    format!(
                        "Expected a unary operator but found {:?} at position {}",
                        token.kind, self.position
                    )
                })?;

                // Parse the right hand side of the unary operator with the highest precedence
                let rhs = self.parse_expression_impl(Precedence::Unary)?;
                Ok(Expr::UnaryOp {
                    op,
                    expr: Box::new(rhs),
                })
            }

            _ => Err(format!("Expected expression, but found {:?}", token.kind)),
        }
    }

    fn parse_infix(&mut self, lhs: Expr) -> Result<Expr, String> {
        let token = self
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        // Parse special cases for function calls, array indexing, etc
        match token.kind {
            TokenKind::LParen => {
                return self.parse_call_expression(lhs);
            }

            TokenKind::LBracket => {
                return self.parse_index_expression(lhs);
            }

            TokenKind::Star => {
                return self.parse_pointer_dereference(lhs);
            }

            TokenKind::Ampersand => {
                return self.parse_pointer_address_of(lhs);
            }

            _ => {}
        }

        // Parse binary operators
        if let Some(op) = token.kind.clone().to_binary_op() {
            let precedence = Precedence::from_token_kind(&token.kind);

            // For left-associative operators, we use the current precedence level for rhs
            // For right-associative operators, we use one level lower precedence for rhs
            let rhs = self.parse_expression_impl(precedence)?;

            return Ok(Expr::BinaryOp {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            });
        }

        Err(format!(
            "Expected infix operator, but found {:?}",
            token.kind
        ))
    }

    fn parse_pointer_dereference(&mut self, pointer: Expr) -> Result<Expr, String> {
        // *ptr
        Ok(Expr::Dereference {
            pointer: Box::new(pointer),
        })
    }

    fn parse_pointer_address_of(&mut self, expr: Expr) -> Result<Expr, String> {
        // &var
        Ok(Expr::AddressOf {
            expr: Box::new(expr),
        })
    }

    fn parse_index_expression(&mut self, array: Expr) -> Result<Expr, String> {
        // arr.[0]
        self.expect(TokenKind::LBracket)?;
        let index = self.parse_expression_impl(Precedence::None)?;
        self.expect(TokenKind::RBracket)?;

        Ok(Expr::Index {
            array: Box::new(array),
            index: Box::new(index),
        })
    }

    fn parse_call_expression(&mut self, callee: Expr) -> Result<Expr, String> {
        // foo(1, 2, 3)
        let mut args = Vec::new();

        // If the next token is not a closing parenthesis, we have arguments to parse
        if let Some(token) = self.peek() {
            if token.kind != TokenKind::RParen {
                loop {
                    let arg = self.parse_expression_impl(Precedence::None)?;
                    args.push(arg);

                    // If the next token is a comma, we have more arguments to parse
                    if let Some(token) = self.peek() {
                        if token.kind == TokenKind::Comma {
                            self.next();
                            continue;
                        }
                    }

                    // If the next token is not a comma, we expect a closing parenthesis
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }
    // -- End of Pratt parser implementation --

    // ----------------------
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

// Tests
// -------
mod parser_tests {
    use super::*;

    #[test]
    fn parse_expression() {
        let source = "3 + 4 * 2 / ( 1 - 5 ) ^ 2 ^ 3";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let expr = parser.parse_expression();
        assert!(expr.is_ok());
    }

    #[test]
    fn parse_array_literal() {
        let source = "[1, 2, 3, 4]";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let array = parser.array_literal();
        assert!(array.is_ok());
    }

    #[test]
    fn parse_literal() {
        let source = "42 3.14 true \"hello, world\"";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let literal = parser.numeric_literal();
        assert!(literal.is_ok());
    }

    #[test]
    fn parse_function_call() {
        let source = "foo(1, 2, 3)";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let expr = parser.parse_expression();
        assert!(expr.is_ok());
    }

    #[test]
    fn parse_index_expression() {
        let source = "arr.[0]";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let expr = parser.parse_expression();
        assert!(expr.is_ok());
    }
}
