//! Json parser

use std::{borrow::Cow, collections::HashMap, fmt::{Display, Write}};

mod lexer;
mod parser;

#[cfg(feature = "bindings")]
pub mod export;

pub type Result<T> = std::result::Result<T,Cow<'static,str>>;

#[derive(Debug)]
pub enum Json {
    Array(Vec<Json>),
    Object(HashMap<String,Json>),
    String(String),
    Number(f64),
    True, False, Null,
}

impl Json {
    pub fn deserialize(text: &str) -> Result<Json> {
        let mut tokens = lexer::tokenize(text)?;
        parser::parse(&mut tokens)
    }
    pub fn serialize(&self, out: &mut dyn Write) -> std::fmt::Result {
        match self {
            Json::Array(elements) => {
                out.write_char('[')?;
                for i in 0..elements.len() {
                    elements[i].serialize(out)?;
                    if i < elements.len() -1 {
                        out.write_char(',')?;
                    }
                }
                out.write_char(']')?;
            },
            Json::Object(obj) => {
                out.write_char('{')?;
                let mut first = true;
                for (k,v) in obj {
                    if !first {
                        out.write_char(',')?;
                    }
                    first = false;
                    out.write_str(k)?;
                    out.write_char(':')?;
                    v.serialize(out)?;
                }
                out.write_char('}')?;
            },
            Json::String(s) => { write!(out, "\"{s}\"")?; },
            Json::Number(n) => { write!(out, "{n}")?; },
            Json::True => { out.write_str("true")? },
            Json::False => { out.write_str("false")? },
            Json::Null => { out.write_str("null")? },
        }
        Ok(())
    }
}

impl Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        self.serialize(&mut string).unwrap();
        write!(f, "{}", string)
    }
}
