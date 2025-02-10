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
    pub use core::fmt;
    pub use alloc::vec::Vec;
    pub use alloc::borrow::Cow;
    pub use alloc::boxed::Box;

    #[cfg(feature = "std")]
    pub type Map<K,V> = std::collections::HashMap<K,V>;

    #[cfg(not(feature = "std"))]
    pub type Map<K,V> = alloc::collections::BTreeMap<K,V>;
}

use core::ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign, Div, DivAssign, Mul, MulAssign};

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
    /// Serializes the JSON object into a [`fmt::Write`]
    pub fn serialize(&self, out: &mut dyn fmt::Write) -> fmt::Result {
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
    #[inline]
    pub fn get(&self, key: impl AsRef<str>) -> Option<&Json> {
        self.object().and_then(|obj| obj.get(key.as_ref()))
    }
    /// Same as [get](Self::get), but with a mutable reference
    #[inline]
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut Json> {
        self.object_mut().and_then(|obj| obj.get_mut(key.as_ref()))
    }
    /// Attempts to get a value of the given json array.
    /// If the json enum is not an Array variant, or if
    /// it doesn't contain the key, returns None
    #[inline]
    pub fn nth(&self, i: usize) -> Option<&Json> {
        self.array().and_then(|arr| arr.get(i))
    }
    /// Same as [nth](Self::nth), but with a mutable reference
    #[inline]
    pub fn nth_mut(&mut self, i: usize) -> Option<&mut Json> {
        self.array_mut().and_then(|arr| arr.get_mut(i))
    }

    /// Attempts to get the inner [`Number`] of the json
    /// object, if it is a [`Number`] variant
    ///
    /// [`Number`]: Json::Number
    #[inline]
    pub fn number(&self) -> Option<f64> {
        if let Json::Number(n) = self {
            Some(*n)
        } else { None }
    }
    /// Expects the json object to be a [`Number`] variant
    ///
    /// # Panics
    /// If the json object is not a [`Number`] variant
    ///
    /// [`Number`]: Json::Number
    #[inline]
    pub fn expect_number(&self) -> f64 {
        self.number().unwrap()
    }
    /// Attempts to get a mutable reference to the inner [`Number`]
    /// of the json object, if it is a [`Number`] variant
    ///
    /// [`Number`]: Json::Number
    #[inline]
    pub fn number_mut(&mut self) -> Option<&mut f64> {
        if let Json::Number(n) = self {
            Some(n)
        } else { None }
    }
    /// Expects the json object to be a [`Number`] variant
    /// and gets a mutable reference to the inner number.
    ///
    /// # Panics
    /// If the json object is not a [`Number`] variant
    ///
    /// [`Number`]: Json::Number
    #[inline]
    pub fn expect_number_mut(&mut self) -> &mut f64 {
        self.number_mut().unwrap()
    }

    /// Attempts to get the inner [`String`] of the json
    /// object, if it is a [`String`] variant
    ///
    /// [`String`]: Json::String
    #[inline]
    pub fn string(&self) -> Option<&str> {
        if let Json::String(s) = self {
            Some(s)
        } else { None }
    }
    /// Expects the json object to be a [`String`] variant
    ///
    /// # Panics
    /// If the json object is not a [`String`] variant
    ///
    /// [`String`]: Json::String
    #[inline]
    pub fn expect_string(&self) -> &str {
        self.string().unwrap()
    }
    /// Attempts to get a mutable reference to the inner
    /// [`String`] of the json object, if it is a [`String`] variant
    ///
    /// [`String`]: Json::String
    #[inline]
    pub fn string_mut(&mut self) -> Option<&mut str> {
        if let Json::String(s) = self {
            Some(s)
        } else { None }
    }
    /// Expects the json object to be a [`String`] variant
    /// and gets a reference to the inner string
    ///
    /// # Panics
    /// If the json object is not a [`String`] variant
    ///
    /// [`String`]: Json::String
    #[inline]
    pub fn expect_string_mut(&mut self) -> &mut str {
        self.string_mut().unwrap()
    }

    /// Attempts to get the inner Object of the json object, if
    /// it is an Object variant
    #[inline]
    pub fn object(&self) -> Option<&Map<Box<str>,Json>> {
        if let Json::Object(o) = self {
            Some(o)
        } else { None }
    }
    /// Expects the json object to be a [`Object`] variant
    /// and gets a reference to the inner object
    ///
    /// # Panics
    /// If the json object is not a [`Object`] variant
    ///
    /// [`Object`]: Json::Object
    #[inline]
    pub fn expect_object(&self) -> &Map<Box<str>,Json> {
        self.object().unwrap()
    }
    /// Attempts to get a mutable reference to the inner [`Object`] of
    /// the json element, if it is an [`Object`] variant
    ///
    /// [`Object`]: Json::Object
    #[inline]
    pub fn object_mut(&mut self) -> Option<&mut Map<Box<str>,Json>> {
        if let Json::Object(o) = self {
            Some(o)
        } else { None }
    }
    /// Expects the json object to be a [`Object`] variant
    /// and gets a mutable reference to the inner object
    ///
    /// # Panics
    /// If the json object is not a [`Object`] variant
    ///
    /// [`Object`]: Json::Object
    #[inline]
    pub fn expect_object_mut(&mut self) -> &mut Map<Box<str>,Json> {
        self.object_mut().unwrap()
    }

    /// Attempts to get the inner Array of the json object, if
    /// it is an Array variant
    #[inline]
    pub fn array(&self) -> Option<&[Json]> {
        if let Json::Array(o) = self {
            Some(o)
        } else { None }
    }
    /// Expects the json object to be a [`Array`] variant
    ///
    /// # Panics
    /// If the json object is not a [`Array`] variant
    ///
    /// [`Array`]: Json::Array
    #[inline]
    pub fn expect_array(&self) -> &[Json] {
        self.array().unwrap()
    }
    /// Attempts to get the inner Array of the json object, if
    /// it is an Array variant
    #[inline]
    pub fn array_mut(&mut self) -> Option<&mut [Json]> {
        if let Json::Array(o) = self {
            Some(o)
        } else { None }
    }
    /// Expects the json object to be a [`Array`] variant
    ///
    /// # Panics
    /// If the json object is not a [`Array`] variant
    ///
    /// [`Array`]: Json::Array
    #[inline]
    pub fn expect_array_mut(&mut self) -> &mut [Json] {
        self.array_mut().unwrap()
    }

    /// Attempts to get the inner boolean value of the json object, if
    /// it is a True or False variant
    #[inline]
    pub fn boolean(&self) -> Option<bool> {
        if let Json::True = self {
            Some(true)
        } else if let Json::False = self {
            Some(false)
        } else { None }
    }
    /// Expects the json object to be a [`True`] or [`False`] variant
    ///
    /// # Panics
    /// If the json object is not a [`True`] or [`False`] variant
    ///
    /// [`True`]: Json::True
    /// [`False`]: Json::False
    #[inline]
    pub fn expect_boolean(&self) -> bool {
        self.boolean().unwrap()
    }

    /// Returns true if the json is a Nil variant
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self,Json::Null)
    }
}

impl fmt::Display for Json {
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

            impl AddAssign<$nty> for Json {
                fn add_assign(&mut self, rhs: $nty) {
                    *self.expect_number_mut() += f64::from(rhs);
                }
            }

            impl Add<$nty> for Json {
                type Output = Json;

                fn add(self, rhs: $nty) -> Self::Output {
                    Json::Number(self.expect_number() + f64::from(rhs))
                }
            }

            impl SubAssign<$nty> for Json {
                fn sub_assign(&mut self, rhs: $nty) {
                    *self.expect_number_mut() -= f64::from(rhs);
                }
            }

            impl Sub<$nty> for Json {
                type Output = Json;

                fn sub(self, rhs: $nty) -> Self::Output {
                    Json::Number(self.expect_number() - f64::from(rhs))
                }
            }

            impl MulAssign<$nty> for Json {
                fn mul_assign(&mut self, rhs: $nty) {
                    *self.expect_number_mut() *= f64::from(rhs);
                }
            }

            impl Mul<$nty> for Json {
                type Output = Json;

                fn mul(self, rhs: $nty) -> Self::Output {
                    Json::Number(self.expect_number() * f64::from(rhs))
                }
            }

            impl DivAssign<$nty> for Json {
                fn div_assign(&mut self, rhs: $nty) {
                    *self.expect_number_mut() /= f64::from(rhs);
                }
            }

            impl Div<$nty> for Json {
                type Output = Json;

                fn div(self, rhs: $nty) -> Self::Output {
                    Json::Number(self.expect_number() / f64::from(rhs))
                }
            }
        )*
    };
}

from_num!(f64,f32,i32,i16,u16,u8);

impl From<String> for Json {
    fn from(value: String) -> Self {
        Self::String(value.into_boxed_str())
    }
}

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

impl Index<&str> for Json {
    type Output = Json;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!("Attemp to index a json element that doesn't contain the given key: '{index}'")
        })
    }
}

impl IndexMut<&str> for Json {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.get_mut(index).unwrap_or_else(|| {
            panic!("Attemp to index a json element that doesn't contain the given key: '{index}'")
        })
    }
}

impl Index<usize> for Json {
    type Output = Json;

    fn index(&self, index: usize) -> &Self::Output {
        self.nth(index).unwrap_or_else(|| {
            panic!("Attemp to index a json element that can't be indexed by {index}")
        })
    }
}

impl IndexMut<usize> for Json {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.nth_mut(index).unwrap_or_else(|| {
            panic!("Attemp to index a json element that can't be indexed by {index}")
        })
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
