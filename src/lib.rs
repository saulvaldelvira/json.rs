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

#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

mod prelude {
    pub use alloc::string::ToString;
    pub use core::fmt::{self,Display,Write};
    pub use alloc::vec::Vec;
    pub use alloc::borrow::Cow;
    pub use alloc::boxed::Box;

    #[cfg(feature = "std")]
    pub type Map<K,V> = std::collections::HashMap<K,V>;

    #[cfg(not(feature = "std"))]
    pub type Map<K,V> = alloc::collections::BTreeMap<K,V>;
}

use prelude::*;

mod lexer;
mod parser;

#[cfg(feature = "bindings")]
pub mod export;

mod error;

type Result<T> = core::result::Result<T,error::Error>;

/// Represents a JSON object
#[derive(Debug,PartialEq)]
pub enum Json {
    Array(Box<[Json]>),
    Object(Map<Box<str>,Json>),
    String(Box<str>),
    Number(f64),
    True, False, Null,
}

/// Configures the JSON parser
#[repr(C)]
#[derive(Clone,Copy)]
pub struct JsonConfig {
    /// Max depth for nested objects
    pub max_depth: u32,
    /// Recover from errors.
    /// For example, trailing commas on objects
    /// are not allowed, but this flag makes
    /// the parser skip them.
    pub recover_from_errors: bool,
}

/// Default config used by [`Json::deserialize`]
const DEFAULT_CONFIG: JsonConfig = JsonConfig {
    max_depth: u32::MAX,
    recover_from_errors: false,
};

impl Default for JsonConfig {
    fn default() -> Self { DEFAULT_CONFIG }
}

impl Json {
    /// Deserializes the given string into a [Json] object
    ///
    /// ## Configuration used
    /// [`max_depth`](JsonConfig::max_depth) = [`u32::MAX`]
    ///
    /// [`recover_from_errors`](JsonConfig::recover_from_errors) = false
    #[inline]
    pub fn deserialize(text: impl AsRef<str>) -> Result<Json> {
        Json::deserialize_with_config(text, DEFAULT_CONFIG)
    }
    /// Deserializes the given string into a [Json] object
    /// using the given [`JsonConfig`]
    pub fn deserialize_with_config(text: impl AsRef<str>, conf: JsonConfig) -> Result<Json> {
        let text = text.as_ref();
        let tokens = lexer::tokenize(text)?;
        parser::parse(text, &tokens, conf)
    }
    /// Serializes the JSON object into a `fmt::Write`
    pub fn serialize(&self, out: &mut dyn Write) -> core::fmt::Result {
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
    /// Same as [get](Self::get), but with a mutable reference
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
    /// Same as [nth](Self::nth), but with a mutable reference
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
    pub fn object(&self) -> Option<&Map<Box<str>,Json>> {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

macro_rules! from_num {
    ( $( $nty:ty ),* ) => {
        $(
            impl From<$nty> for Json {
                fn from(value: $nty) -> Self {
                    Self::Number(value.into())
                }
            }
        )*
    };
}

from_num!(f64,f32,i32,i16,u16,u8);

impl From<Box<str>> for Json {
    fn from(value: Box<str>) -> Self {
        Self::String(value)
    }
}

impl<'a> From<&'a str> for Json {
    fn from(value: &'a str) -> Self {
        Self::String(value.into())
    }
}

impl From<Vec<Json>> for Json {
    fn from(value: Vec<Json>) -> Self {
        Self::Array(value.into())
    }
}

impl From<Map<Box<str>,Json>> for Json {
    fn from(value: Map<Box<str>,Json>) -> Self {
        Self::Object(value)
    }
}

impl From<bool> for Json {
    fn from(value: bool) -> Self {
        if value { Json::True } else { Json::False }
    }
}

#[doc(hidden)]
pub use prelude::Map;

/// Builds a [Json] object
///
/// # Example
/// ```
/// use json::json;
///
/// let j = json!({
///     "hello" : ["w", 0, "r", "ld"],
///     "array" : [
///         { "key" : "val" },
///         12.21,
///         null,
///         true,
///         false
///     ]
/// });
/// ```
#[macro_export]
macro_rules! json {
    ( $lit:literal ) => {
        $crate::Json::from( $lit )
    };
    ( [ $( $e:tt ),* ] ) => {
        $crate::Json::from(
            vec![
                $(
                    json!($e)
                ),*
            ]
        )
    };
    ( { $( $key:literal : $val:tt ),*  } ) => {
        {
            let mut map = $crate::Map::new();
            $( map.insert($key .into(), json!($val) );  )*
            $crate::Json::from ( map )
        }
    };
    ( null ) => {
        $crate::Json::Null
    }
}
