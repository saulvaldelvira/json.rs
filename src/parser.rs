use crate::lexer::span::FilePosition;
use crate::lexer::Span;
use crate::prelude::*;

use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::Json;
use crate::JsonConfig;
use crate::Result;

struct Parser<'a> {
    tokens: &'a [Token],
    src: &'a str,
    curr: usize,
    conf: JsonConfig,
    depth: u32,
}

impl<'a> Parser<'a> {
    fn parse(&mut self) -> Result<Json> {
        self.value()
    }
    fn is_finished(&self) -> bool {
        self.curr >= self.tokens.len()
    }
    fn error<T>(&mut self, msg: impl Into<Cow<'static,str>>) -> Result<T> {
        let FilePosition { start_line, start_col, .. } = self.previous()?.span().file_position(self.src);
        let msg = format!("[{start_line}:{start_col}]: {}", msg.into());
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
        if self.match_type(TokenKind::LSquareBracket) {
            enter!( self.array() )
        } else if self.match_type(TokenKind::LeftBrace) {
            enter!( self.object() )
        } else if self.match_type(TokenKind::Number) {
            self.number()
        } else if self.match_type(TokenKind::String) {
            self.string()
        } else if self.match_type(TokenKind::True) {
            Ok( Json::True )
        } else if self.match_type(TokenKind::False) {
            Ok( Json::False )
        } else if self.match_type(TokenKind::Null) {
            Ok( Json::Null )
        } else {
           self.error("Unknown token")
        }
    }
    fn array(&mut self) -> Result<Json> {
        let mut elems = Vec::new();
        while !self.check(TokenKind::RSquareBracket) {
            if self.is_finished() { break }
            if !elems.is_empty() {
                self.consume(TokenKind::Comma, "Expected comma after element")?;
            }
            if self.peek()?.get_type() == TokenKind::RSquareBracket {
                if self.conf.recover_from_errors {
                    continue
                } else {
                   return self.error("Trailing comma on list");
                }
            }
            let json = self.value()?;
            elems.push(json);
        }
        self.consume(TokenKind::RSquareBracket, "Unclosed '['")?;
        Ok( elems.into() )
    }
    fn object(&mut self) -> Result<Json> {
        let mut elems = Map::new();
        while !self.check(TokenKind::RightBrace) {
            if self.is_finished() { break }
            if !elems.is_empty() {
                self.consume(TokenKind::Comma, "Expected comma after element")?;
            }

            if ! self.check(TokenKind::String) {
                let msg = match self.previous().unwrap().get_type() {
                    TokenKind::Comma => {
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
            let span = self.advance()?.span();
            let key = self.owned_lexem_strip(span);

            self.consume(TokenKind::Colon, "Expected ':'")?;
            let json = self.value()?;
            elems.insert(key.into(),json);
        }
        self.consume(TokenKind::RightBrace, "Unclosed '{'")?;
        Ok( Json::Object(elems) )
    }
    fn owned_lexem_strip(&self, span: Span) -> Box<str> {
        let slice = span.slice(self.src);
        let slice = slice.strip_prefix("\"").unwrap_or(slice);
        let slice = slice.strip_suffix("\"").unwrap_or(slice);
        Box::from(slice)
    }
    fn number(&mut self) -> Result<Json> {
        let n: f64 = self.previous()?.span().slice(self.src).parse().unwrap();
        Ok( Json::Number(n) )
    }
    fn string(&mut self) -> Result<Json> {
        let s = self.previous()?.span();
        let s = self.owned_lexem_strip(s);
        Ok( Json::String(s) )
    }
    fn consume(&mut self, t: TokenKind, msg: &'static str) -> Result<&Token> {
        if self.check(t) { return self.advance(); }
        self.error(msg)
    }
    fn match_type(&mut self, t: TokenKind) -> bool {
        if self.check(t) {
            self.advance().unwrap();
            return true;
        }
        false
    }
    fn check(&mut self, t: TokenKind) -> bool {
        if self.is_finished() { return false; }
        self.peek().unwrap().get_type() == t
    }
    fn advance(&mut self) -> Result<&Token> {
        if !self.is_finished() {
            self.curr += 1;
        }
        self.previous()
    }
    fn peek(&mut self) -> Result<&Token> {
        self.tokens.get(self.curr)
                   .ok_or_else(|| "Index should be valid when calling peek".into())
    }
    fn previous(&mut self) -> Result<&Token> {
        self.tokens.get(self.curr - 1)
                   .ok_or_else(|| "Index should be valid when calling peek".into())
    }
}

pub fn parse(src: &str, tokens: &[Token], conf: JsonConfig) -> Result<Json> {
    Parser {
        tokens,
        src,
        curr: 0,
        depth: 0,
        conf,
    }.parse()
}
