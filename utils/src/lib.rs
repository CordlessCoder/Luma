use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Range<usize>,
}

impl<T> From<(T, Range<usize>)> for Spanned<T> {
    fn from((inner, span): (T, Range<usize>)) -> Self {
        Self { inner, span }
    }
}

impl<T> Spanned<T> {
    pub const fn new(inner: T, span: Range<usize>) -> Self {
        Self { inner, span }
    }
    pub fn as_span(&self) -> Range<usize> {
        self.span.clone()
    }
    pub fn split(self) -> (T, Range<usize>) {
        (self.inner, self.span)
    }
}
