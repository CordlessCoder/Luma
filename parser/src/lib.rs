use ast::Module;
use diagnostics::{AggregateError, ErrorComponent};
use lexer::{SToken, Token};
use source::SourceFile;
use std::{borrow::Cow, collections::VecDeque};
mod basic_ops;

pub struct Parser<'s, Tokens: Iterator> {
    tokens: Tokens,
    source: SourceFile,
    filename: &'s str,
    peeked: VecDeque<SToken<'s>>,
    lexer_errors: AggregateError,
    errors: AggregateError,
}

// If a parsing function returns None, an error occurred and we must synchronize to try to
// continue parsing
impl<'s, Tokens: Iterator<Item = Result<SToken<'s>, ErrorComponent>>> Parser<'s, Tokens> {
    pub fn parse_module_header(&mut self) -> Option<&'s str> {
        if !self.consume_if(|t| matches!(t, Token::AtModule)) {
            let (next, span) = self.peek_next_split();
            let msg = format!("Expected @module declaration, found {next:?}");
            self.new_parse_error(span, msg);
            return None;
        }
        let Some((Token::StringLit(name), span)) =
            self.advance_if_split(|t| matches!(t, Token::StringLit(_)))
        else {
            let (next, span) = self.peek_next_split();
            let msg = format!("Expected the module name after @module declaration, found {next:?}");
            self.new_parse_error(span, msg);
            return None;
        };
        let Cow::Borrowed(name) = name else {
            self.new_parse_error(span, "Escape sequences in module names are not allowed.");
            return None;
        };
        Some(name)
    }
    pub fn parse(&mut self) -> Result<Module<'s>, AggregateError> {
        let name = self.parse_module_header().unwrap_or("UNSPECIFIED");
        let mut body = Vec::new();
        //     while self.peek_one().is_some() {
        //         let Some(stmt) = self.parse_statement() else {
        //             continue;
        //         };
        //         m.statements.push(stmt);
        //     }
        if !self.lexer_errors.is_empty() {
            return Err(std::mem::take(&mut self.lexer_errors));
        }
        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }
        Ok(Module {
            body: ast::Block(body),
            name,
        })
    }
}
