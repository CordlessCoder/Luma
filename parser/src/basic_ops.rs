use std::{collections::VecDeque, fmt::Debug};

use crate::Parser;
use diagnostics::{AggregateError, ErrorComponent};
use lexer::{SToken, Token};
use source::{SourceFile, Span};

impl<'s, Tokens: Iterator<Item = Result<SToken<'s>, ErrorComponent>>> Parser<'s, Tokens> {
    pub fn new(source: SourceFile, tokens: Tokens) -> Self {
        Self {
            tokens,
            peeked: VecDeque::new(),
            errors: AggregateError::new(),
            lexer_errors: AggregateError::new(),
            source,
        }
    }
    pub(crate) fn end_span(&self) -> Span {
        let len = self.source.text().len();
        len..len
    }
    pub(crate) fn add_lexer_error(&mut self, err: ErrorComponent) {
        self.lexer_errors.add_error(err);
    }
    pub(crate) fn new_parse_error(
        &mut self,
        span: Span,
        message: impl ToString,
    ) -> &mut ErrorComponent {
        self.errors.add_error(ErrorComponent::new(
            self.source.clone(),
            message.to_string(),
            span,
        ))
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
        self.peeked.push_front(tok);
    }
    pub(crate) fn advance_split(&mut self) -> (Option<Token<'s>>, Span) {
        let span = self.peek_next_span().unwrap_or_else(|| self.end_span());
        (self.advance().map(|s| s.inner), span)
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
    ) -> (Option<Token<'s>>, Span) {
        let span = self.peek_next_span().unwrap_or_else(|| self.end_span());
        let t = self.advance_if(predicate).map(|s| s.inner);
        (t, span)
    }
    pub(crate) fn consume_if(&mut self, predicate: impl FnOnce(&Token) -> bool) -> bool {
        self.advance_if(predicate).is_some()
    }
    /// Advance if the token matches, log an error otherwise.
    pub(crate) fn expect(&mut self, expected: &Token<'s>) -> Option<SToken<'s>> {
        let tok = self.advance_if(|t| t == expected);
        if tok.is_none() {
            let (found, span) = self.peek_next_split();
            let msg = format!("Expected {expected}, found {found:?}");
            self.new_parse_error(span, msg);
        }
        tok
    }
}
impl<T: Iterator> Debug for Parser<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format_error_warning_count(components: &[ErrorComponent]) -> String {
            let mut errors: usize = 0;
            let mut warnings: usize = 0;
            for component in components {
                match component.level {
                    diagnostics::ErrorLevel::Error => errors += 1,
                    diagnostics::ErrorLevel::Warning => warnings += 1,
                }
            }
            match (errors, warnings) {
                (0, 0) => "Empty".to_string(),
                (err, 0) => format!("{err} errors"),
                (0, warn) => format!("{warn} warnings"),
                (err, warn) => format!("{err} errors, {warn} warnings"),
            }
        }
        let lexer_errors = format_error_warning_count(&self.lexer_errors.components);
        let parser_errors = format_error_warning_count(&self.lexer_errors.components);
        f.debug_struct("Parser")
            .field("tokens", &"Iterator")
            .field("source", &self.source)
            .field("peeked", &self.peeked)
            .field("lexer_errors", &lexer_errors)
            .field("errors", &parser_errors)
            .finish()
    }
}
