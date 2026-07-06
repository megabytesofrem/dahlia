use std::ops::Range;

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum TokenKind {
    // Symbols
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("=")]
    Equals,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,
    #[token(":")]
    Colon,
    #[token("&")]
    Ampersand,

    // Multi-character symbols
    #[token("+=")]
    PlusEquals,
    #[token("-=")]
    MinusEquals,
    #[token("*=")]
    StarEquals,
    #[token("/=")]
    SlashEquals,
    #[token("==")]
    DoubleEqual,
    #[token("!=")]
    BangEqual,
    #[token("<")]
    LessThan,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    GreaterThan,
    #[token(">=")]
    GreaterEqual,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Bang,

    // Keywords
    #[token("arena")]
    Arena,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("fn")]
    Fn,
    #[token("struct")]
    Struct,
    #[token("const")]
    Const,
    #[token("var")]
    Var,
    #[token("do")]
    Do,
    #[token("end")]
    End,

    // Reserved types

    // Use u8 to store the size of the type, while saving bytes
    #[regex(r"u(?:8|16|32|64)", |lex| lex.slice()[1..].parse::<u8>().ok(), priority = 3)]
    UIntType(u8),
    #[regex(r"i(?:8|16|32|64)", |lex| lex.slice()[1..].parse::<u8>().ok(), priority = 3)]
    IntType(u8),
    #[regex(r"f(?:32|64)", |lex| lex.slice()[1..].parse::<u8>().ok(), priority = 3)]
    FloatType(u8),

    #[token("char")]
    CharType,
    #[token("str")]
    StrType,
    #[token("bool")]
    BoolType,
    #[token("void")]
    VoidType,

    // For pointer/array type, don't store the lexed value to save on memory
    #[regex(r"\*[a-zA-Z0-9_]+")]
    PointerType,
    #[regex(r"[a-zA-Z0-9_]+\[[0-9]*\]")] // i32[3], f32[10], str[]
    ArrayType,

    // Literals don't store the lexed value to save on memory
    #[regex(r"[0-9]+")]
    IntLit,
    #[regex(r"[0-9]+\.[0-9]+")]
    FloatLit,
    #[regex(r"'([^'\\]|\\.)'")]
    CharLit,
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit,
    #[regex(r"true|false")]
    BoolLit,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 2)]
    Ident,

    // Comments
    #[regex(r"//[^\n]*", logos::skip, allow_greedy = true)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip, allow_greedy = true)]
    Comment,

    // Documentation comments are preserved in the AST
    #[regex(r"/\+([^+]|\+[^/])*\+/", allow_greedy = true)]
    DocComment,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub kind: Result<TokenKind, ()>,
    pub lexeme: &'a str,
    pub span: Range<usize>,
}

pub struct Lexer<'a> {
    lexer: logos::Lexer<'a, TokenKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> std::iter::Peekable<Self> {
        Self {
            lexer: TokenKind::lexer(source),
        }
        .peekable()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.lexer.next()?;
        let span = self.lexer.span();
        let lexeme = self.lexer.slice();

        Some(Token { kind, lexeme, span })
    }
}

// Tests
// -------
mod lexer_tests {
    use super::*;

    #[test]
    fn test_comment_lexer() {
        let source = r#"
            // This is a comment
            var x: i32 = 10 // This is another comment

            /* 
             * Multi
             * line
             * comment
            */

            /+ Nested 
               /* Multi-line
                  comment */
            +/
            const y: f32 = 3.14
        "#;

        let mut lexer = Lexer::new(source);

        while let Some(token) = lexer.next() {
            println!("{:?}", token);
        }
    }

    #[test]
    fn test_lexer() {
        let source = r#"
            var x: i32 = 10
            const y: f32 = 3.14
            if x > 5 then
              x += 1
            else
              x -= 1;
            end
        "#;

        let mut lexer = Lexer::new(source);

        while let Some(token) = lexer.next() {
            println!("{:?}", token);
        }
    }
}
