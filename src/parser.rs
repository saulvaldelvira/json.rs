use std::borrow::Cow;
use std::collections::HashMap;

use crate::lexer::token::Token;
use crate::lexer::token::TokenType;
use crate::Json;
use crate::JsonConfig;
use crate::Result;

struct Parser<'a> {
    tokens: &'a mut [Token],
    curr: usize,
    conf: JsonConfig,
    depth: u32,
}

trait StripQuotes {
    fn strip_quotes(&mut self);
}

impl StripQuotes for String {
    fn strip_quotes(&mut self) {
        self.pop();
        if !self.is_empty() {
            self.remove(0);
        }
    }
}

impl<'a> Parser<'a> {
    fn parse(&mut self) -> Result<Json> {
        self.value()
    }
    fn is_finished(&self) -> bool {
        self.curr >= self.tokens.len()
    }
    fn error<T>(&mut self, msg: impl Into<Cow<'static,str>>) -> Result<T> {
        let msg = format!("{}: {}", self.previous()?.span(), msg.into());
        Err(msg.into())
    }
    fn value(&mut self) -> Result<Json> {
        if self.depth > self.conf.max_depth {
            return self.error("Max depth reached")
        }
        macro_rules! enter {
            ($c:expr) => {
                {
                    self.depth += 1;
                    let j = $c;
                    self.depth -= 1;
                    j
                }
            };
        }
        if self.match_type(TokenType::LSquareBracket) {
            enter!( self.array() )
        } else if self.match_type(TokenType::LeftBrace) {
            enter!( self.object() )
        } else if self.match_type(TokenType::Number) {
            self.number()
        } else if self.match_type(TokenType::String) {
            self.string()
        } else if self.match_type(TokenType::True) {
            Ok( Json::True )
        } else if self.match_type(TokenType::False) {
            Ok( Json::False )
        } else if self.match_type(TokenType::Null) {
            Ok( Json::Null )
        } else {
           self.error("Unknown token")
        }
    }
    fn array(&mut self) -> Result<Json> {
        let mut elems = Vec::new();
        while !self.check(TokenType::RSquareBracket) {
            if self.is_finished() { break }
            if !elems.is_empty() {
                self.consume(TokenType::Comma, "Expected comma after element")?;
            }
            if self.peek()?.get_type() == TokenType::RSquareBracket {
                if self.conf.recover_from_errors {
                    continue
                } else {
                   return self.error("Trailing comma on list");
                }
            }
            let json = self.value()?;
            elems.push(json);
        }
        self.consume(TokenType::RSquareBracket, "Unclosed '['")?;
        Ok( elems.into() )
    }
    fn object(&mut self) -> Result<Json> {
        let mut elems = HashMap::new();
        while !self.check(TokenType::RightBrace) {
            if self.is_finished() { break }
            if !elems.is_empty() {
                self.consume(TokenType::Comma, "Expected comma after element")?;
            }

            if ! self.check(TokenType::String) {
                let msg = match self.previous().unwrap().get_type() {
                    TokenType::Comma => {
                        if self.conf.recover_from_errors {
                            continue
                        } else {
                            "Trailing comma in object"
                        }
                    },
                    _ => "Expected STRING",
                };
                return self.error(msg);
            }
            let key = self.advance()?;
            let mut key = key.take_lexem();
            key.strip_quotes();

            self.consume(TokenType::Colon, "Expected ':'")?;
            let json = self.value()?;
            elems.insert(key.into(),json);
        }
        self.consume(TokenType::RightBrace, "Unclosed '{'")?;
        Ok( Json::Object(elems) )
    }
    fn number(&mut self) -> Result<Json> {
        let n: f64 = self.previous()?.get_lexem().parse().unwrap();
        Ok( Json::Number(n) )
    }
    fn string(&mut self) -> Result<Json> {
        let mut s = self.previous()?.take_lexem();
        s.strip_quotes();
        Ok( Json::String(s.into()) )
    }
    fn consume(&mut self, t: TokenType, msg: &'static str) -> Result<&mut Token> {
        if self.check(t) { return self.advance(); }
        self.error(msg)
    }
    fn match_type(&mut self, t: TokenType) -> bool {
        if self.check(t) {
            self.advance().unwrap();
            return true;
        }
        false
    }
    fn check(&mut self, t: TokenType) -> bool {
        if self.is_finished() { return false; }
        self.peek().unwrap().get_type() == t
    }
    fn advance(&mut self) -> Result<&mut Token> {
        if !self.is_finished() {
            self.curr += 1;
        }
        self.previous()
    }
    fn peek(&mut self) -> Result<&mut Token> {
        self.tokens.get_mut(self.curr)
                   .ok_or_else(|| "Index should be valid when calling peek".into())
    }
    fn previous(&mut self) -> Result<&mut Token> {
        self.tokens.get_mut(self.curr - 1)
                   .ok_or_else(|| "Index should be valid when calling peek".into())
    }
}

pub fn parse(tokens: &mut [Token], conf: JsonConfig) -> Result<Json> {
    Parser {
        tokens,
        curr: 0,
        depth: 0,
        conf,
    }.parse()
}
