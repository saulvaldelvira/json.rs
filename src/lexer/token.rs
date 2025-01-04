use core::fmt;

use crate::lexer::span::Span;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum TokenKind {
    /* Single-character tokens. */
    LSquareBracket, RSquareBracket, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Colon, String, Number,
    False, True, Null
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self{ kind, span }
    }
    pub fn get_type(&self) -> TokenKind { self.kind }
    pub fn span(&self) -> Span { self.span }
}
