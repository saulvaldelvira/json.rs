use core::num::ParseFloatError;
use core::{error, fmt};

use crate::lexer::span::FilePosition;
use crate::prelude::Cow;

#[derive(Debug)]
pub struct Error {
    msg: Cow<'static,str>,
    fpos: Option<FilePosition>,
}

impl Error {
   pub fn new(msg: impl Into<Cow<'static,str>>) -> Self {
       Self {
           msg: msg.into(),
           fpos: None
       }
   }
   pub fn set_file_pos(&mut self, fpos: FilePosition) {
       self.fpos = Some(fpos);
   }

   pub fn get_message(&self) -> &Cow<'static,str> { &self.msg }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(fpos) = self.fpos {
            let FilePosition { start_line, start_col, .. } = fpos;
            write!(f, "[{start_line}:{start_col}]: ")?;
        }
        write!(f, "{}", self.msg)
    }
}

impl error::Error for Error { }

impl From<Cow<'static,str>> for Error {
    fn from(value: Cow<'static,str>) -> Self {
        Error::new(value)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::new(value.to_string())
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Error::new(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::new(value)
    }
}
