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
    #[token(r"u[8|16|32|64]", |lex| lex.slice().parse::<i32>().ok())]
    UnsignedType(i32),
    #[token(r"i[8|16|32|64]", |lex| lex.slice().parse::<i32>().ok())]
    IntType(i32),
    #[token(r"f[32|64]", |lex| lex.slice().parse::<i32>().ok())]
    FloatType(i32),
    #[token("str")]
    StrType,
    #[token("bool")]
    BoolType,
    #[token("void")]
    VoidType,

    // For pointer/array type, don't store the lexed value to save on memory
    #[token(r"*[\w]+")]
    PointerType,
    #[token(r"[\w]+[[0-9]*]")] // i32[3], f32[10], str[]
    ArrayType,

    // Literals don't store the lexed value to save on memory
    #[regex(r"[0-9]+")]
    IntLit,
    #[regex(r"[0-9]+\.[0-9]+")]
    FloatLit,
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit,
    #[regex(r"true|false")]
    BoolLit,
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
