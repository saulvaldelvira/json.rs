mod cursor;
pub mod span;

use cursor::Cursor;
use span::FilePosition;
pub use span::Span;

use crate::prelude::*;

use crate::Result;

pub mod token;
use token::{Token, TokenKind};

struct Lexer<'a> {
    c: Cursor<'a>,
}

pub fn tokenize(text: &str) -> Result<Box<[Token]>> {
    Lexer {
        c: Cursor::new(text),
    }
    .tokenize()
}

impl Lexer<'_> {
    fn tokenize(&mut self) -> Result<Box<[Token]>> {
        let mut tokens: Vec<Token> = Vec::new();
        while !self.c.is_finished() {
            self.c.step();
            if let Some(t) = self.scan_token()? {
                tokens.push(t);
            }
        }
        Ok(tokens.into_boxed_slice())
    }
    #[allow(clippy::unnecessary_wraps)]
    fn add_token(&self, token_type: TokenKind) -> Result<Option<Token>> {
        Ok(Some(Token::new(token_type, self.c.get_span())))
    }
    fn scan_token(&mut self) -> Result<Option<Token>> {
        match self.c.advance() {
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            '[' => self.add_token(TokenKind::LSquareBracket),
            ']' => self.add_token(TokenKind::RSquareBracket),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ':' => self.add_token(TokenKind::Colon),
            '/' => {
                if self.c.match_next('/') {
                    self.comment()
                } else if self.c.match_next('*') {
                    self.ml_comment()
                } else {
                    Err("Unexpected character".into())
                }
            }
            '"' => self.string(),
            ' ' | '\n' | '\r' | '\t' => Ok(None), // Ignore whitespace.
            c => {
                if c.is_ascii_digit() {
                    self.number()
                } else if c.is_ascii_alphabetic() {
                    self.keyword()
                } else {
                    let mut msg = "Unexpected character [".to_string();
                    msg += &c.to_string();
                    msg += "]";
                    self.error(&msg)?;
                    Ok(None)
                }
            }
        }
    }
    #[allow(clippy::unnecessary_wraps)]
    fn comment(&mut self) -> Result<Option<Token>> {
        self.c.advance_while(|c| *c != '\n');
        Ok(None)
    }
    fn ml_comment(&mut self) -> Result<Option<Token>> {
        while self.c.advance() != '*' || self.c.peek() != '/' {
            if self.c.is_finished() {
                self.error("Non terminated comment block.")?;
            }
        }
        self.c.advance(); /* Consume the / */
        Ok(None)
    }
    fn string(&mut self) -> Result<Option<Token>> {
        let mut scaping = false;
        loop {
            let c = self.c.peek();
            if c == '"' && !scaping {
                break;
            }
            scaping = c == '\\';

            self.c.advance();

            if self.c.is_finished() {
                return self.error("Unterminated string");
            }
        }
        self.c.advance();
        self.add_token(TokenKind::String)
    }
    fn number(&mut self) -> Result<Option<Token>> {
        self.c.advance_while(char::is_ascii_digit);
        if self.c.peek() == '.' && self.c.peek_next().is_ascii_digit() {
            self.c.advance();
            self.c.advance_while(char::is_ascii_digit);
        }
        self.add_token(TokenKind::Number)
    }
    fn keyword(&mut self) -> Result<Option<Token>> {
        self.c.advance_while(char::is_ascii_alphanumeric);
        let lexem = self.c.current_lexem();
        let token_type = match lexem {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            _ => return Err(format!("Unknown keyword '{lexem}'").into()),
        };
        self.add_token(token_type)
    }
    fn error(&mut self, msg: &str) -> Result<Option<Token>> {
        let FilePosition {
            start_line,
            start_col,
            ..
        } = self.c.file_pos();
        let msg = format!("[{start_line}:{start_col}] {msg}");
        Err(msg.into())
    }
}

#[cfg(test)]
mod test;
