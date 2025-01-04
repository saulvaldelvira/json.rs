use core::num::ParseFloatError;
use core::{error, fmt};

use crate::prelude::Cow;

#[derive(Debug)]
pub struct Error(Cow<'static,str>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error { }

impl From<Cow<'static,str>> for Error {
    fn from(value: Cow<'static,str>) -> Self {
        Error(value)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error(value.to_string().into())
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Error(value.into())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error(value.into())
    }
}
