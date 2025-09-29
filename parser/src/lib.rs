use ast::{Module, Stmt};
use diagnostics::{AggregateError, ErrorComponent};
use lexer::{SToken, Token};
use source::SourceFile;
use std::collections::VecDeque;
use utils::Spanned;

use crate::expr::BindingPower;

mod basic_ops;
mod expr;

pub struct Parser<'s, Tokens: Iterator> {
    tokens: Tokens,
    source: SourceFile,
    peeked: VecDeque<SToken<'s>>,
    lexer_errors: AggregateError,
    errors: AggregateError,
}

// If a parsing function returns None, an error occurred and we must synchronize to try to
// continue parsing
impl<'s, Tokens: Iterator<Item = Result<SToken<'s>, ErrorComponent>>> Parser<'s, Tokens> {
    pub(crate) fn parse_module_header(&mut self) -> Option<&'s str> {
        if !self.consume_if(|t| matches!(t, Token::At("module"))) {
            let (next, span) = self.peek_next_split();
            let msg = format!("Expected @module declaration, found {next:?}");
            self.new_parse_error(span, msg);
            return None;
        }
        let (Some(Token::Ident(name)), _) = self.advance_if_split(|t| matches!(t, Token::Ident(_)))
        else {
            let (next, span) = self.peek_next_split();
            let msg = format!(
                "Expected an identifier module name after @module declaration, found {next:?}"
            );
            self.new_parse_error(span, msg);
            return None;
        };
        Some(name)
    }
    pub(crate) fn parse_use(&mut self) -> Option<Stmt<'s>> {
        self.expect(&Token::At("use"))?;
        let (t, span) = self.advance_if_split(|t| matches!(t, Token::Ident(_)));
        let Some(Token::Ident(module)) = t else {
            self.new_parse_error(
                span,
                format!("Expected an identifier name after @use, found {t:?}"),
            );
            return None;
        };
        let mut alias = None;
        if self.consume_if(|t| matches!(t, Token::As)) {
            let (t, span) = self.advance_if_split(|t| matches!(t, Token::Ident(_)));
            let Some(Token::Ident(val)) = t else {
                self.new_parse_error(
                    span,
                    format!("Expected an identifier alias after @use _ as, found {t:?}"),
                );
                return None;
            };
            alias = Some(val);
        }
        Some(Stmt::Use { module, alias })
    }
    pub(crate) fn parse_return(&mut self) -> Option<Stmt<'s>> {
        self.expect(&Token::Return)?;

        let val = if !self.consume_if(|t| matches!(t, Token::Semicolon)) {
            let val = Some(self.parse_expr(BindingPower::Lowest)?);
            self.expect(&Token::Semicolon)?;
            val
        } else {
            None
        };
        Some(Stmt::Return(val))
    }
    pub(crate) fn parse_stmt(&mut self) -> Option<Stmt<'s>> {
        use Token::*;
        let tok = self.peek_next()?;
        match tok.inner {
            At("use") => self.parse_use(),
            Return => self.parse_return(),
            _ => {
                let Spanned { inner, span } = self.advance()?;
                self.new_parse_error(span, "TODO")
                    .set_long_message(format!(
                        "Parsing {inner} as a statement is not yet implemented"
                    ))
                    .set_level(diagnostics::ErrorLevel::Warning);
                None
            }
        }
    }
    pub fn parse(&mut self) -> (Module<'s>, AggregateError) {
        let name = self.parse_module_header().unwrap_or("UNSPECIFIED");
        let mut body = Vec::new();
        while self.peek_next().is_some() {
            let Some(stmt) = self.parse_stmt() else {
                continue;
            };
            body.push(stmt);
        }
        let components: Vec<ErrorComponent> = self
            .lexer_errors
            .components
            .drain(..)
            .chain(self.errors.components.drain(..))
            .collect();
        (
            Module {
                body: ast::Block(body),
                name,
            },
            AggregateError { components },
        )
    }
}
