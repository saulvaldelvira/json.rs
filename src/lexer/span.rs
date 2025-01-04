//! Utilities to represent spans inside a file

use std::{fmt, str};

/// Represents a span in a buffer, bounded by an offset and a len
#[derive(Clone,Copy,Debug)]
pub struct Span {
    /// Offset of the span inside the buffer
    pub offset: usize,
    /// Length of the span
    pub len: usize,
}

/// Represents a [`Span`] in a file, bounded by
/// it's start line and col, plus it's end line and col
#[derive(Debug,Clone,Copy,Default)]
pub struct FilePosition {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl fmt::Display for FilePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FilePosition { start_line, start_col, end_line, end_col } = self;
        write!(f, "[{start_line}:{start_col},{end_line}:{end_col}]")
    }
}

impl Span {
    /// Slices the given string with this span
    #[must_use]
    #[inline(always)]
    pub fn slice<'a>(&self, src: &'a str) -> &'a str {
        &src[self.offset..self.offset + self.len]
    }
    /// Gets the [file position] of this span in the given string slice
    ///
    /// [file position]: FilePosition
    #[must_use]
    pub fn file_position(&self, src: &str) -> FilePosition {
        let mut fpos = FilePosition {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        };

        for c in src[..self.offset].chars() {
            if c == '\n' {
                fpos.start_col = 0;
                fpos.start_line += 1;
            }
            fpos.start_col += 1;
        }

        fpos.end_line = fpos.start_line;
        fpos.end_col = fpos.start_col;

        for c in src[self.offset..self.offset + self.len].chars() {
            if c == '\n' {
                fpos.end_col = 0;
                fpos.end_line += 1;
            }
            fpos.end_col += 1;
        }

        fpos
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]", self.offset, self.offset + self.len)
    }
}
