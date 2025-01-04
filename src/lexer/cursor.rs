use core::str::CharIndices;

use crate::lexer::Span;

use super::span::FilePosition;

pub struct Cursor<'a> {
    chars: CharIndices<'a>,
    start: usize,
    start_chars: CharIndices<'a>,
    file_pos: FilePosition,
    current: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            chars: text.char_indices(),
            start: 0,
            start_chars: text.char_indices(),
            file_pos: FilePosition::default(),
            current: 0,
        }
    }
    pub fn step(&mut self) {
        self.start = self.current;
        self.file_pos.start_line = self.file_pos.end_line;
        self.file_pos.start_col = self.file_pos.end_col;
        self.start_chars = self.chars.clone();
    }
    pub fn is_finished(&self) -> bool {
        self.chars.as_str().is_empty()
    }
    pub fn current_lexem(&self) -> &str {
        let n = self.current - self.start;
        let n = self.start_chars.clone().take(n).map(|(_,c)| c.len_utf8()).sum();
        &self.start_chars.as_str()[0..n]
    }
    pub fn get_span(&self) -> Span {
        let offset = self.start_chars.offset();
        Span {
            offset,
            len: self.chars.offset() - offset
        }
    }
    pub fn file_pos(&self) -> FilePosition { self.file_pos }
    pub fn advance(&mut self) -> char {
        let c = self.chars.next().map_or('\0', |(_,c)| c);
        self.current += 1;
        self.file_pos.end_col += 1;
        if c == '\n' {
            self.file_pos.end_line += 1;
            self.file_pos.end_col = 1;
        }
        c
    }
    pub fn advance_while<F>(&mut self, f: F) -> bool
    where
        F: Fn(&char) -> bool
    {
        while f(&self.peek()) {
            self.advance();
            if self.is_finished() { return false; }
        }
        true
    }
    pub fn peek(&self) -> char {
        self.chars
            .clone()
            .next()
            .map_or('\0', |(_,c)| c)
    }
    pub fn peek_next(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next()
            .map_or('\0', |(_,c)| c)
    }
    pub fn match_next(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance();
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::Cursor;

    #[test]
    fn test() {
        let text = "Hello world!";
        let mut cursor = Cursor::new(text);
        for c in text.chars() {
            assert!(!cursor.is_finished());
            let next = cursor.advance();
            assert_eq!(next, c);
        }
        assert!(cursor.is_finished());
    }
}
