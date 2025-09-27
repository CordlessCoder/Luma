use std::{iter::FusedIterator, ops::Range};

use crate::Span;

#[derive(Debug, Clone)]
pub struct LineIterator<'s> {
    pub(crate) text: &'s str,
    pub(crate) line_starts: &'s [usize],
    pub(crate) remaining: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct TextLine<'s> {
    pub text: &'s str,
    pub span: Span,
    pub line: usize,
}

impl<'s> LineIterator<'s> {
    pub fn get_line(&self, line: usize) -> Option<TextLine<'s>> {
        let start = *self.line_starts.get(line)?;
        let end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text.len());
        let span = start..end;
        let text = &self.text[span.clone()];
        Some(TextLine { text, span, line })
    }
}

impl<'s> ExactSizeIterator for LineIterator<'s> {
    fn len(&self) -> usize {
        self.remaining.len()
    }
}

impl<'s> FusedIterator for LineIterator<'s> {}

impl<'s> Iterator for LineIterator<'s> {
    type Item = TextLine<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.remaining.next()?;
        self.get_line(line)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.remaining.start = self.remaining.end;
            return None;
        }
        self.remaining.start += n;
        self.next()
    }
}

impl<'s> DoubleEndedIterator for LineIterator<'s> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let line = self.remaining.next_back()?;
        self.get_line(line)
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.remaining.start = self.remaining.end;
            return None;
        }
        self.remaining.end -= n;
        self.next_back()
    }
}
