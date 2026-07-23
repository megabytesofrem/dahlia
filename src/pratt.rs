use crate::{ast::Expr, lexer::TokenKind, parser::Parser};

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

    #[allow(dead_code)]
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
            TokenKind::LParen | TokenKind::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

#[allow(dead_code)]
impl<'a> Parser<'a> {
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
        let is_prefix = {
            let token = self
                .peek()
                .ok_or_else(|| "Unexpected end of input".to_string())?;

            match token.kind {
                TokenKind::Minus | TokenKind::Bang | TokenKind::Ampersand | TokenKind::Star => true,
                _ => false,
            }
        };

        if is_prefix {
            let token = self.next().unwrap();
            let op = token.kind.clone().to_unary_op().ok_or_else(|| {
                format!(
                    "Expected a unary operator but found {:?} at position {}",
                    token.kind, self.position
                )
            })?;

            // Parse the right hand side of the unary operator with the highest precedence
            let rhs = self.parse_expression_impl(Precedence::Unary)?;

            return Ok(Expr::UnaryOp {
                op,
                expr: Box::new(rhs),
            });
        } else {
            return self.parse_primary();
        }
    }

    fn parse_infix(&mut self, lhs: Expr) -> Result<Expr, String> {
        let token = self
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        // Special cases for function calls, array indexing, etc
        // Parse the prefixes *before* the binary operator for cases like `*a + 1`

        match token.kind {
            TokenKind::LParen => {
                return self.parse_call_expression(lhs);
            }

            TokenKind::Dot => {
                return self.parse_index_expression(lhs);
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
        let args = self.between_delimited_by(
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Comma,
            |p| p.parse_expression_impl(Precedence::None),
        )?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }
    // -- End of Pratt parser implementation --
}
