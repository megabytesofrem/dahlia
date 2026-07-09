use crate::{
    ast::{DottedName, Expr, Literal, Stmt, ToplevelStmt, TypedIdentifier, types::Type},
    lexer::{Lexer, Token, TokenKind},
};

use std::iter::Peekable;

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    pub position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Peekable<Lexer<'a>>) -> Self {
        Self { lexer, position: 0 }
    }

    pub fn peek(&mut self) -> Option<&Token<'a>> {
        self.lexer.peek()
    }

    pub fn next(&mut self) -> Option<Token<'a>> {
        let token = self.lexer.next();
        if let Some(ref t) = token {
            self.position = t.span.end;
        }
        token
    }

    /// Expect a specific token kind and consume it if found, otherwise return an error.
    pub fn expect(&mut self, token_kind: TokenKind) -> Result<Token<'a>, String> {
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

    /// Optionally consume a specific token kind if found, otherwise do nothing.
    #[allow(dead_code)]
    pub fn optional(&mut self, token_kind: TokenKind) -> Option<Token<'a>> {
        match self.peek() {
            Some(token) if token.kind == token_kind => self.next(),
            _ => None,
        }
    }

    /// `Parser::expect` but generalized to work on any of a set of token kinds.
    #[allow(dead_code)]
    pub fn any_of(&mut self, token_kinds: &[TokenKind]) -> Result<Token<'a>, String> {
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
    #[allow(dead_code)]
    pub fn none_of(&mut self, token_kinds: &[TokenKind]) -> Result<Token<'a>, String> {
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

    #[allow(dead_code)]
    pub fn until<T, F>(&mut self, closer: TokenKind, mut parse_element: F) -> Result<Vec<T>, String>
    where
        F: FnMut(&mut Self) -> Result<T, String>,
    {
        let mut items = Vec::new();

        while let Some(token) = self.peek() {
            if token.kind == closer {
                break;
            }

            items.push(parse_element(self)?);
        }

        Ok(items)
    }

    #[allow(dead_code)]
    pub fn until_any1<T, F>(
        &mut self,
        closers: &[TokenKind],
        mut parse_element: F,
    ) -> Result<Vec<T>, String>
    where
        F: FnMut(&mut Self) -> Result<T, String>,
    {
        let mut items = Vec::new();

        while let Some(token) = self.peek() {
            if closers.contains(&token.kind) {
                break;
            }

            items.push(parse_element(self)?);
        }

        Ok(items)
    }

    #[allow(dead_code)]
    pub fn between<T, F>(
        &mut self,
        start: TokenKind,
        end: TokenKind,
        mut parse_element: F,
    ) -> Result<Vec<T>, String>
    where
        F: FnMut(&mut Self) -> Result<T, String>,
    {
        let mut items = Vec::new();

        // Some callers (like `parse_primary` and `parse_infix`) already consume the start
        // token before calling this, so we only consume `start` if it's actually the next token.
        if let Some(t) = self.peek() {
            if t.kind == start {
                self.next();
            }
        }

        while let Some(token) = self.peek() {
            if token.kind == end {
                break;
            }

            items.push(parse_element(self)?);
        }

        self.expect(end)?;

        Ok(items)
    }

    #[allow(dead_code)]
    pub fn between_delimited_by<T, F>(
        &mut self,
        start: TokenKind,
        end: TokenKind,
        delimiter: TokenKind,
        mut parse_element: F,
    ) -> Result<Vec<T>, String>
    where
        F: FnMut(&mut Self) -> Result<T, String>,
    {
        let mut items = Vec::new();
        let position = self.position;

        // Some callers (like `parse_primary` and `parse_infix`) already consume the start
        // token before calling this, so we only consume `start` if it's actually the next token.
        if let Some(t) = self.peek() {
            if t.kind == start {
                self.next();
            }
        }

        // Before parsing elements, check if the list is completely empty (e.g. `[]`)
        if let Some(token) = self.peek() {
            if token.kind == end {
                self.expect(end)?;
                return Ok(items);
            }
        }

        // Parse the first item
        items.push(parse_element(self)?);

        while let Some(token) = self.peek() {
            if token.kind == end {
                break;
            }

            if token.kind == delimiter {
                self.next(); // Consume the delimiter

                // Allow trailing delimiters: if the very next token is the end (e.g. `[1, 2,]`),
                // we break without throwing an error about missing an element
                if let Some(next_tok) = self.peek() {
                    if next_tok.kind == end {
                        break;
                    }
                }

                // Parse the subsequent item
                items.push(parse_element(self)?);
            } else {
                return Err(format!(
                    "Expected delimiter {:?} or end {:?} but found {:?} at position {}",
                    delimiter, end, token.kind, position
                ));
            }
        }

        self.expect(end)?;

        Ok(items)
    }

    #[allow(dead_code)]
    pub fn delimited_by<T, F>(
        &mut self,
        delimiter: TokenKind,
        mut parse_element: F,
    ) -> Result<Vec<T>, String>
    where
        F: FnMut(&mut Self) -> Result<T, String>,
    {
        let mut items = Vec::new();

        // Check if there's anything to parse at all. If the current token isn't a delimiter
        // and there's a token present, try parsing the first element.
        if self.peek().is_some() {
            items.push(parse_element(self)?);

            while let Some(token) = self.peek() {
                if token.kind == delimiter {
                    self.next(); // Consume the delimiter

                    // After the delimiter, parse the next element
                    items.push(parse_element(self)?);
                } else {
                    break;
                }
            }
        }

        Ok(items)
    }

    #[allow(dead_code)]
    pub fn delimited_by_any1<T, F>(
        &mut self,
        delimiters: &[TokenKind],
        mut parse_element: F,
    ) -> Result<Vec<T>, String>
    where
        F: FnMut(&mut Self) -> Result<T, String>,
    {
        let mut items = Vec::new();

        // Check if there's anything to parse at all. If the current token isn't a delimiter
        // and there's a token present, try parsing the first element.
        if self.peek().is_some() {
            items.push(parse_element(self)?);

            while let Some(token) = self.peek() {
                if delimiters.contains(&token.kind) {
                    self.next(); // Consume the delimiter

                    // After the delimiter, parse the next element
                    items.push(parse_element(self)?);
                } else {
                    break;
                }
            }
        }

        Ok(items)
    }

    // Parser Rules
    // ----------------
    // NOTE: The parser rules are implemented following the language grammar (see GRAMMAR.md).
    //       Each rule is represented as a function in a recursive-descent parser
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

    fn allocator_type(&mut self) -> Result<Type, String> {
        self.expect(TokenKind::AllocatorType)?;
        let name = self.identifier()?;
        self.expect(TokenKind::LParen)?;
        let size_token = self.expect(TokenKind::IntLit)?;
        let size = size_token.lexeme.parse::<usize>().map_err(|_| {
            format!(
                "Expected an allocator size but found {:?} at position {}",
                size_token.lexeme, self.position
            )
        })?;
        self.expect(TokenKind::RParen)?;

        Ok(Type::Allocator { name, size })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let prev_position = self.position;

        if let Some(token) = self.peek() {
            let token_kind_type = Type::from_token_kind(&token.kind);

            match token.clone().kind {
                TokenKind::Star => self.pointer_type(),

                TokenKind::AllocatorType => self.allocator_type(),

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
    // NOTE: Precedence climbing (Pratt) is implemented in pratt.rs, and the rest of the expression
    //       parsing that is not tied to precedence climbing is here.

    pub fn if_expression(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::If)?;

        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let else_branch = if let Some(token) = self.peek() {
            if token.kind == TokenKind::Else {
                self.next(); // Consume the "else" token
                Some(self.parse_block()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Expr::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        })
    }

    pub fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        let is_literal = match token.kind {
            TokenKind::IntLit
            | TokenKind::FloatLit
            | TokenKind::BoolLit
            | TokenKind::StringLit
            | TokenKind::CharLit
            | TokenKind::LBracket => true,
            _ => false,
        };

        if is_literal {
            let literal = self.parse_literal(token)?;
            return Ok(Expr::Literal(literal));
        } else {
            match token.kind {
                TokenKind::Ident => {
                    let name = token.lexeme.to_string();
                    let identifier = TypedIdentifier {
                        name,
                        type_: None, // Type information can be added later if needed
                    };

                    return Ok(Expr::Identifier(identifier));
                }
                TokenKind::If => {
                    let if_expr = self.if_expression()?;
                    return Ok(if_expr);
                }

                TokenKind::LParen => {
                    let expr = self.parse_expression()?;
                    self.expect(TokenKind::RParen)?;

                    return Ok(expr);
                }
                _ => {
                    return Err(format!(
                        "Expected a primary expression but found {:?} at position {}",
                        token.kind, self.position
                    ));
                }
            }
        }
    }

    pub fn parse_literal(&mut self, token: Token) -> Result<Literal, String> {
        match token.kind {
            TokenKind::IntLit => {
                let value = token
                    .lexeme
                    .parse::<i64>()
                    .map_err(|_| "Invalid integer literal".to_string())?;
                Ok(Literal::Int(value))
            }
            TokenKind::FloatLit => {
                let value = token
                    .lexeme
                    .parse::<f64>()
                    .map_err(|_| "Invalid float literal".to_string())?;
                Ok(Literal::Float(value))
            }
            TokenKind::BoolLit => {
                let value = token.lexeme == "true";
                Ok(Literal::Bool(value))
            }
            TokenKind::StringLit => {
                let value = token.lexeme.to_string();
                Ok(Literal::String(value))
            }
            TokenKind::CharLit => {
                let value = token.lexeme.chars().next().ok_or_else(|| {
                    format!(
                        "Invalid character literal: {:?} at position {}",
                        token.lexeme, self.position
                    )
                })?;
                Ok(Literal::Char(value))
            }
            TokenKind::LBracket => {
                let elements = self.between_delimited_by(
                    TokenKind::LBracket,
                    TokenKind::RBracket,
                    TokenKind::Comma,
                    |p| p.parse_expression(),
                )?;

                Ok(Literal::Array {
                    elements,
                    element_type: None,
                })
            }

            _ => Err(format!(
                "Expected a literal but found {:?} at position {}",
                token.kind, self.position
            )),
        }
    }

    // ----------------------

    // Statements
    // ----------------------
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let statements = self.between(TokenKind::LBrace, TokenKind::RBrace, |p| {
            p.parse_statement()
        })?;

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        let token = self
            .peek()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        match token.kind {
            TokenKind::Var => self.var_declaration(),
            TokenKind::Const => self.const_declaration(),
            TokenKind::Ident => self.var_assign(),
            TokenKind::For => self.for_loop(),
            TokenKind::While => self.while_loop(),
            TokenKind::Defer => self.defer_statement(),
            TokenKind::New => self.new_statement(),
            TokenKind::Return => self.return_statement(),
            TokenKind::Break => self.break_statement(),

            _ => self.stmt_expr(),
        }
    }

    fn stmt_expr(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expression()?;
        Ok(Stmt::Expr(expr))
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        // var_declaration:
        //    | "var" typed_identifier "=" expr

        self.expect(TokenKind::Var)?;

        let (name, type_) = self.typed_identifier()?;
        let value = self.parse_expression()?;

        Ok(Stmt::VarDeclaration {
            name: TypedIdentifier { name, type_ },
            value,
        })
    }

    fn var_assign(&mut self) -> Result<Stmt, String> {
        // var_assign:
        //    | identifier "=" expr

        let ident = self.identifier()?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;

        Ok(Stmt::Assign { name: ident, value })
    }

    fn const_declaration(&mut self) -> Result<Stmt, String> {
        // const_declaration:
        //    | "const" typed_identifier "=" expr

        self.expect(TokenKind::Const)?;

        let (name, type_) = self.typed_identifier()?;
        let value = self.parse_expression()?;

        Ok(Stmt::ConstDeclaration {
            name: TypedIdentifier { name, type_ },
            value,
        })
    }

    fn for_loop(&mut self) -> Result<Stmt, String> {
        // for_loop:
        //    | "for" typed_identifier ":" expr "{" stmt* "}"

        self.expect(TokenKind::For)?;

        let iterator = self.typed_identifier()?;
        self.expect(TokenKind::Colon)?;
        let iterable = self.parse_expression()?;

        let body = self.parse_block()?;

        Ok(Stmt::For {
            iterator: TypedIdentifier {
                name: iterator.0,
                type_: iterator.1,
            },
            iterable,
            body,
        })
    }

    fn while_loop(&mut self) -> Result<Stmt, String> {
        // while_loop:
        //    | "while" expr "{" stmt* "}"

        self.expect(TokenKind::While)?;

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt::While { condition, body })
    }

    fn defer_statement(&mut self) -> Result<Stmt, String> {
        // defer_statement:
        //    | "defer" stmt

        self.expect(TokenKind::Defer)?;

        let stmt = self.parse_statement()?;

        Ok(Stmt::Defer {
            stmt: Box::new(stmt),
        })
    }

    fn new_statement(&mut self) -> Result<Stmt, String> {
        // new_statement:
        //    | "new" name "(" [ expr ("," expr)* ] ")"
        //    | "new" type "[" expr "]"

        todo!()
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        // return_statement:
        //    | "return" expr?

        self.expect(TokenKind::Return)?;

        let value = if let Some(token) = self.peek() {
            if token.kind != TokenKind::RBrace {
                Some(self.parse_expression()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Stmt::Return { value })
    }

    fn break_statement(&mut self) -> Result<Stmt, String> {
        // break_statement:
        //    | "break"

        self.expect(TokenKind::Break)?;

        Ok(Stmt::Break)
    }

    // Toplevel Statements
    // ----------------------
    fn struct_member(&mut self) -> Result<TypedIdentifier, String> {
        let (name, type_) = self.typed_identifier()?;
        Ok(TypedIdentifier { name, type_ })
    }

    fn struct_declaration(&mut self) -> Result<ToplevelStmt, String> {
        // struct_declaration:
        //    | "struct" identifier "{" (struct_member ",")* "}"

        self.expect(TokenKind::Struct)?;
        let name = self.identifier()?;
        self.expect(TokenKind::LBrace)?;
        let members = self.delimited_by(TokenKind::Comma, |p| p.struct_member())?;
        self.expect(TokenKind::RBrace)?;

        Ok(ToplevelStmt::StructDeclaration {
            name,
            fields: members,
        })
    }

    fn enum_member(&mut self) -> Result<String, String> {
        let name = self.identifier()?;
        Ok(name)
    }

    fn enum_declaration(&mut self) -> Result<ToplevelStmt, String> {
        // enum_declaration:
        //    | "enum" identifier "{" (enum_member ",")* "}"

        self.expect(TokenKind::Enum)?;
        let name = self.identifier()?;
        self.expect(TokenKind::LBrace)?;
        let members = self.delimited_by(TokenKind::Comma, |p| p.enum_member())?;
        self.expect(TokenKind::RBrace)?;

        Ok(ToplevelStmt::EnumDeclaration {
            name,
            variants: members
                .into_iter()
                .map(|member| TypedIdentifier {
                    name: member,
                    type_: None,
                })
                .collect(),
        })
    }

    fn fn_declaration(&mut self) -> Result<ToplevelStmt, String> {
        // fn_declaration:
        //    | "fn" identifier "(" [ typed_identifier ("," typed_identifier)* ] ")" "{" stmt* "}"

        self.expect(TokenKind::Fn)?;
        let name = self.identifier()?;
        self.expect(TokenKind::LParen)?;
        let params = self.delimited_by(TokenKind::Comma, |p| p.typed_identifier())?;
        self.expect(TokenKind::RParen)?;
        let body = self.parse_block()?;

        Ok(ToplevelStmt::FnDeclaration {
            name: TypedIdentifier { name, type_: None },
            params: params
                .into_iter()
                .map(|(name, type_)| TypedIdentifier { name, type_ })
                .collect(),
            body,
        })
    }

    pub fn parse_toplevel_stmt(&mut self) -> Result<ToplevelStmt, String> {
        let token = self
            .peek()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        match token.kind {
            TokenKind::Fn => self.fn_declaration(),
            TokenKind::Struct => self.struct_declaration(),
            TokenKind::Enum => self.enum_declaration(),
            _ => {
                let stmt = self.parse_statement()?;
                Ok(ToplevelStmt::Stmt(stmt))
            }
        }
    }

    // AST
    // ----------------------
    pub fn parse(&mut self) -> Result<Vec<ToplevelStmt>, String> {
        let mut toplevel_stmts = Vec::new();

        while self.peek().is_some() {
            let stmt = self.parse_toplevel_stmt()?;
            toplevel_stmts.push(stmt);
        }

        Ok(toplevel_stmts)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}

// Tests
// -------

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::lexer::Lexer;

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

        let array = parser.parse_expression();
        assert!(array.is_ok());
    }

    #[test]
    fn parse_literal() {
        let source = "42 3.14 true \"hello, world\"";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let literal = parser.parse_expression();
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

    #[test]
    fn parse_struct_declaration() {
        let source = "struct Point { x: i32, y: i32 }";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let struct_decl = parser.struct_declaration();
        assert!(struct_decl.is_ok());
    }

    #[test]
    fn parse_fn_declaration() {
        let source = "fn add(a: i32, b: i32) { return a + b }";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        let fn_decl = parser.fn_declaration();
        assert!(fn_decl.is_ok());
    }
}
