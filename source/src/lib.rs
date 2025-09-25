use std::{cell::OnceCell, num::NonZero, ops::Range, rc::Rc};

mod lines;
pub use lines::*;

// A cheaply [Clone]able container for source code.
/// Allows for efficiently mapping from byte offsets to line and column values.
#[derive(Clone, Debug)]
pub struct SourceFile(Rc<SourceStorage>);

impl SourceFile {
    pub fn new(path: String, text: String) -> Self {
        let storage = SourceStorage::new(path, text);
        Self(Rc::new(storage))
    }
    pub fn path(&self) -> &str {
        &self.0.path
    }
    pub fn text(&self) -> &str {
        &self.0.text
    }
    pub fn span_to_pos(&self, span: &Span) -> [HumanFilePos; 2] {
        [span.start, span.end.saturating_sub(1)].map(|o| self.offset_to_pos(o))
    }
    /// Panics if the provided offset is out of bounds of the text.
    pub fn offset_to_pos(&self, offset: usize) -> HumanFilePos {
        if offset > self.text().len() {
            panic!("Cannot calculate an offset outside the file")
        }
        let starts = self.0.line_starts();
        let line = starts
            .binary_search(&offset)
            .unwrap_or_else(|l| l.saturating_sub(1));
        let line_start = starts.get(line).copied().unwrap_or(0);
        let column_bytes = offset.checked_sub(line_start).unwrap() as u32;
        let column = self
            .text()
            .get(line_start..offset)
            // Count UTF-8 chars for columns
            .map(|s| s.chars().count() as u32)
            // Fall back to byte columns if UTF-8 access fails
            .unwrap_or(column_bytes);
        // NOTE: Let's assume line and column counts > 2^32-1 won't be encountered *that* often.
        let line = NonZero::new(line as u32 + 1).unwrap();
        let column = NonZero::new(column + 1).unwrap();
        HumanFilePos {
            line,
            column_utf8: column,
            column_bytes,
        }
    }
    pub fn lines<'s>(&'s self) -> LineIterator<'s> {
        let starts = self.0.line_starts();
        LineIterator {
            text: self.text(),
            line_starts: starts,
            remaining: 0..starts.len() + 1,
        }
    }
}

pub type Span = Range<usize>;

/// A human-readable position in a file
/// line and column both start at 1
#[derive(Debug, Clone, Copy)]
pub struct HumanFilePos {
    line: NonZero<u32>,
    column_utf8: NonZero<u32>,
    column_bytes: u32,
}

impl HumanFilePos {
    pub fn line(&self) -> usize {
        self.line.get() as usize
    }
    pub fn line_0idx(&self) -> usize {
        self.line.get() as usize - 1
    }
    pub fn col(&self) -> usize {
        self.column_utf8.get() as usize
    }
    pub fn col_bytes(&self) -> usize {
        self.column_bytes as usize
    }
}

#[derive(Debug)]
struct SourceStorage {
    text: String,
    path: String,
    line_starts: OnceCell<Box<[usize]>>,
}

impl SourceStorage {
    pub fn new(path: String, text: String) -> Self {
        SourceStorage {
            text,
            path,
            line_starts: OnceCell::new(),
        }
    }
    fn line_starts(&self) -> &[usize] {
        self.line_starts.get_or_init(|| {
            let newlines = self
                .text
                .bytes()
                .enumerate()
                .filter(|&(_, b)| b == b'\n')
                .map(|(idx, _)| idx + 1);
            core::iter::once(0).chain(newlines).collect()
        })
    }
}
