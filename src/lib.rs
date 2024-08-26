//! Json parser
//!
//! # Example
//! ```
//! use json::Json;
//!
//! let j = Json::deserialize(r#"{
//!     "array" : [ 1, 2, "3", null ],
//!     "true" : true,
//!     "nested" : {
//!         "inner" : []
//!     }
//! }"#).unwrap();
//!
//! let Json::Object(map) = j else { panic!() };
//! assert!(
//!     matches!(
//!         map.get("true"),
//!         Some(Json::True)));
//! ```

use std::{borrow::Cow, collections::HashMap, fmt::{Display, Write}};

mod lexer;
mod parser;

#[cfg(feature = "bindings")]
pub mod export;

pub type Result<T> = std::result::Result<T,Cow<'static,str>>;

/// Represents a JSON object
#[derive(Debug)]
pub enum Json {
    Array(Vec<Json>),
    Object(HashMap<String,Json>),
    String(String),
    Number(f64),
    True, False, Null,
}

/// Configures the JSON parser
pub struct JsonConfig {
    /// Max depth for nested objects
    pub max_depth: u32,
    /// Recover from errors.
    /// For example, trailing commas on objects
    /// are not allowed, but this flag makes
    /// the parser skip them.
    pub recover_from_errors: bool,
}

const DEFAULT_CONFIG: JsonConfig = JsonConfig {
    max_depth: 500,
    recover_from_errors: false,
};

impl Default for JsonConfig {
    fn default() -> Self { DEFAULT_CONFIG }
}

macro_rules! deserialize {
    ($text:ident, $conf:ident) => {
        {
            let mut tokens = lexer::tokenize($text .as_ref())?;
            parser::parse(&mut tokens, $conf)
        }
    };
}

impl Json {
    /// Deserializes the given string into a [Json] object
    pub fn deserialize(text: impl AsRef<str>) -> Result<Json> {
        deserialize!(text, DEFAULT_CONFIG)
    }
    /// Deserializes the given string into a [Json] object
    /// using the given [JsonConfig]
    pub fn deserialize_with_config(text: impl AsRef<str>, conf: JsonConfig) -> Result<Json> {
        deserialize!(text, conf)
    }
    /// Serializes the JSON object into a string
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
                    write!(out, "\"{k}\":")?;
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
    /// Attempts to get a value of the given json object.
    /// If the json enum is not an Object variant, or if
    /// it doesn't contain the key, returns None
    pub fn get(&self, key: impl AsRef<str>) -> Option<&Json> {
        if let Json::Object(o) = self {
            o.get(key.as_ref())
        } else {
            None
        }
    }
    /// Same as [get], but with a mutable reference
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut Json> {
        if let Json::Object(o) = self {
            o.get_mut(key.as_ref())
        } else {
            None
        }
    }
    /// Attempts to get a value of the given json array.
    /// If the json enum is not an Array variant, or if
    /// it doesn't contain the key, returns None
    pub fn nth(&self, i: usize) -> Option<&Json> {
        if let Json::Array(arr) = self {
            arr.get(i)
        } else {
            None
        }
    }
    /// Same as [nth], but with a mutable reference
    pub fn nth_mut(&mut self, i: usize) -> Option<&mut Json> {
        if let Json::Array(arr) = self {
            arr.get_mut(i)
        } else {
            None
        }
    }
    /// Attempts to get the inner f64 of the json object, if
    /// it is a Number variant
    pub fn number(&self) -> Option<f64> {
        if let Json::Number(n) = self {
            Some(*n)
        } else { None }
    }
    /// Attempts to get the inner String of the json object, if
    /// it is a String variant
    pub fn string(&self) -> Option<&str> {
        if let Json::String(s) = self {
            Some(s)
        } else { None }
    }
    /// Attempts to get the inner Object of the json object, if
    /// it is an Object variant
    pub fn object(&self) -> Option<&HashMap<String,Json>> {
        if let Json::Object(o) = self {
            Some(o)
        } else { None }
    }
    /// Attempts to get the inner Array of the json object, if
    /// it is an Array variant
    pub fn array(&self) -> Option<&[Json]> {
        if let Json::Array(o) = self {
            Some(o)
        } else { None }
    }
    /// Attempts to get the inner boolean value of the json object, if
    /// it is a True or False variant
    pub fn boolean(&self) -> Option<bool> {
        if let Json::True = self {
            Some(true)
        } else if let Json::False = self {
            Some(false)
        } else { None }
    }
    /// Returns true if the json is a Nil variant
    pub fn is_null(&self) -> bool {
        matches!(self,Json::Null)
    }
}

impl Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        self.serialize(&mut string).unwrap();
        write!(f, "{}", string)
    }
}

impl From<f64> for Json {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Json {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
