use std::collections::VecDeque;

use crate::Parser;
use diagnostics::{AggregateError, ErrorComponent, ErrorWithSource};
use lexer::{SToken, Token};
use source::{SourceFile, Span};

impl<'s, Tokens: Iterator<Item = Result<SToken<'s>, ErrorComponent>>> Parser<'s, Tokens> {
    pub fn new(source: SourceFile, filename: &'s str, tokens: Tokens) -> Self {
        Self {
            tokens,
            peeked: VecDeque::new(),
            errors: AggregateError::new(),
            lexer_errors: AggregateError::new(),
            filename,
            source,
        }
    }
    fn end_span(&self) -> Span {
        let len = self.source.text().len();
        len..len
    }
    fn add_lexer_error(&mut self, err: ErrorComponent) {
        self.lexer_errors.add_error(err);
    }
    pub(crate) fn new_parse_error(
        &mut self,
        span: Span,
        message: impl ToString,
    ) -> &mut ErrorWithSource {
        let err = self
            .errors
            .add_error(ErrorComponent::WithSource(ErrorWithSource::new(
                self.source.clone(),
                message.to_string(),
                span,
            )));
        #[expect(irrefutable_let_patterns)]
        let ErrorComponent::WithSource(s) = err else {
            unreachable!()
        };
        s
    }
    #[inline(always)]
    pub(crate) fn advance(&mut self) -> Option<SToken<'s>> {
        if let t @ Some(_) = self.peeked.pop_front() {
            return t;
        }
        loop {
            match self.tokens.next()? {
                Ok(t) => break Some(t),
                Err(e) => {
                    self.add_lexer_error(e);
                }
            }
        }
    }
    pub(crate) fn put_back(&mut self, tok: SToken<'s>) {
        self.peeked.push_back(tok);
    }
    pub(crate) fn advance_split(&mut self) -> Option<(Token<'s>, Span)> {
        self.advance().map(SToken::split)
    }
    #[inline(always)]
    pub(crate) fn peek(&mut self, idx: usize) -> Option<&SToken<'s>> {
        while self.peeked.len() <= idx {
            match self.tokens.next()? {
                Ok(t) => {
                    self.peeked.push_back(t);
                }
                Err(e) => {
                    self.add_lexer_error(e);
                }
            }
        }
        self.peeked.get(idx)
    }
    #[inline]
    pub(crate) fn peek_next(&mut self) -> Option<&SToken<'s>> {
        self.peek(0)
    }
    #[inline]
    pub(crate) fn peek_next_span(&mut self) -> Option<Span> {
        self.peek_next().map(|s| s.as_span())
    }
    #[inline]
    pub(crate) fn peek_next_split(&mut self) -> (Option<&Token<'s>>, Span) {
        let end_span = self.end_span();
        let next = self.peek_next();
        let span = next.map(|t| t.as_span()).unwrap_or(end_span);
        let next = next.map(|n| &n.inner);
        (next, span)
    }
    #[inline(always)]
    fn check(&mut self, predicate: impl FnOnce(&Token) -> bool) -> bool {
        self.peek_next().is_some_and(|t| predicate(&t.inner))
    }
    pub(crate) fn advance_if(
        &mut self,
        predicate: impl FnOnce(&Token) -> bool,
    ) -> Option<SToken<'s>> {
        let advance = self.check(predicate);
        if advance {
            return self.advance();
        }
        None
    }
    pub(crate) fn advance_if_split(
        &mut self,
        predicate: impl FnOnce(&Token) -> bool,
    ) -> Option<(Token<'s>, Span)> {
        self.advance_if(predicate).map(SToken::split)
    }
    pub(crate) fn consume_if(&mut self, predicate: impl FnOnce(&Token) -> bool) -> bool {
        self.advance_if(predicate).is_some()
    }
}
