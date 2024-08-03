use std::fmt::Display;

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

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}]", self.start_line, self.start_col)
    }
}
