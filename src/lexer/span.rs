use core::fmt;

#[derive(Debug,Clone,Copy)]
pub struct Span {
    pub (super) start_line: usize,
    pub (super) start_col: usize,
    pub (super) end_line: usize,
    pub (super) end_col: usize,
}

impl Default for Span {
    fn default() -> Self {
        Span { start_line: 1, start_col: 1, end_line: 1, end_col: 1 }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]", self.start_line, self.start_col)
    }
}
