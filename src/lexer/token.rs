use core::fmt;

use crate::lexer::span::Span;
use crate::prelude::*;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum TokenType {
    /* Single-character tokens. */
    LSquareBracket, RSquareBracket, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Colon, String, Number,
    False, True, Null
}

#[derive(Debug)]
pub struct Token {
    lexem: Option<String>,
    token_type: TokenType,
    span: Span,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Token {
    pub fn new(lexem: &str, token_type: TokenType, span: Span) -> Self {
        let lexem = Some(lexem.to_owned());
        Self{ lexem, token_type, span }
    }
    pub fn get_type(&self) -> TokenType { self.token_type }
    pub fn take_lexem(&mut self) -> String {
        self.lexem.take()
            .expect("Cannot take lexem of the token. Lexem is None.")
    }
    pub fn get_lexem(&self) -> &str {
        self.lexem.as_deref().unwrap_or("")
    }
    pub fn span(&self) -> Span { self.span }
}
