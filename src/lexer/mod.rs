mod cursor;
mod span;

use crate::prelude::*;

use cursor::Cursor;
pub use span::Span;

use crate::Result;

pub mod token;
use token::{Token,TokenType};

struct Lexer<'a> {
    c: Cursor<'a>,
}

pub fn tokenize(text: &str) -> Result<Vec<Token>> {
    Lexer {
        c: Cursor::new(text),
    }.tokenize()
}

impl<'a> Lexer<'a> {
    fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens:Vec<Token> = Vec::new();
        while !self.c.is_finished() {
            self.c.step();
            if let Some(t) = self.scan_token()? {
                tokens.push(t);
            }
        }
        Ok(tokens)
    }
    fn add_token(&self, token_type: TokenType) -> Result<Option<Token>> {
        Ok(Some(Token::new(
                self.c.current_lexem(),
                token_type, self.c.get_span())))
    }
    fn scan_token(&mut self) -> Result<Option<Token>> {
        match self.c.advance() {
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LSquareBracket),
            ']' => self.add_token(TokenType::RSquareBracket),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ':' => self.add_token(TokenType::Colon),
            '/' =>
                if self.c.match_next('/') {
                    self.comment()
                } else if self.c.match_next('*') {
                    self.ml_comment()
                } else {
                    Err( "Unexpected character".into() )
                },
            '"' => self.string(),
            ' ' | '\n' | '\r' | '\t' => Ok(None) , // Ignore whitespace.
            c =>
                if c.is_ascii_digit() {
                    self.number()
                } else if c.is_ascii_alphabetic() {
                    self.keyword()
                } else{
                    let mut msg = "Unexpected character [".to_string();
                    msg += &c.to_string();
                    msg += "]";
                    self.error(&msg)?;
                    Ok(None)
                }
        }
    }
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
            if c == '"' && !scaping  {
                break
            }
            scaping = c ==  '\\';

            self.c.advance();

            if self.c.is_finished() { return self.error("Unterminated string"); }
        }
        self.c.advance();
        self.add_token(TokenType::String)
    }
    fn number(&mut self) -> Result<Option<Token>> {
        self.c.advance_while(char::is_ascii_digit);
        if self.c.peek() == '.' && self.c.peek_next().is_ascii_digit() {
            self.c.advance();
            self.c.advance_while(char::is_ascii_digit);
        }
        self.add_token(TokenType::Number)
    }
    fn keyword(&mut self) -> Result<Option<Token>> {
        self.c.advance_while(|c| c.is_ascii_alphanumeric() || *c == '_');
        let lexem = self.c.current_lexem();
        let token_type = match lexem {
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            _ => return Ok(None),
        };
        self.add_token(token_type)
    }
    fn error(&mut self, msg: &str) -> Result<Option<Token>> {
        let msg = format!("[{}:{}] {}", self.c.line(), self.c.col(), msg);
        Err(msg.into())
    }
}
