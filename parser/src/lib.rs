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
        let name = self.expect_ident(" module name after @module declaration")?;
        Some(name)
    }
    pub(crate) fn parse_use(&mut self) -> Option<Stmt<'s>> {
        self.expect(&Token::At("use"))?;
        let module = self.expect_ident(" name after @use")?;
        let mut alias = None;
        if self.consume_if(|t| matches!(t, Token::As)) {
            alias = Some(self.expect_ident(" alias after @use _ as")?);
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
    pub(crate) fn parse_var_decl(&mut self, public: bool) -> Option<Stmt<'s>> {
        let (kind, _) = self.advance_if_split(|t| matches!(t, Token::Let | Token::Const));
        // SAFETY: This function should only be called when the next token is var, const or public
        let kind = kind.unwrap();
        let mutable = kind == Token::Let;
        let name = self.expect_ident(" after {kind} token in variable declaration")?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        if self.consume_if(|t| t == &Token::Semicolon) {
            return Some(Stmt::VarDecl {
                name,
                ty,
                init: None,
                public,
                mutable,
            });
        }
        self.expect(&Token::Eq)?;
        let init = self.parse_expr(BindingPower::Lowest)?;
        self.expect(&Token::Semicolon)?;
        Some(Stmt::VarDecl {
            name,
            ty,
            init: Some(init),
            public,
            mutable,
        })
    }
    pub(crate) fn parse_block(&mut self) -> Option<Stmt<'s>> {
        self.expect(&Token::LBrace)?;
        let mut block = Vec::new();
        while self
            .peek_next_split()
            .0
            .is_some_and(|t| matches!(t, Token::RBrace))
        {
            let stmt = self.parse_stmt()?;
            block.push(stmt);
        }
        self.expect(&Token::RBrace)?;
        Some(Stmt::Block(ast::Block(block)))
    }
    pub(crate) fn parse_if(&mut self) -> Option<Stmt<'s>> {
        self.expect(&Token::If)?;
        self.expect(&Token::LParen)?;
        let cond = self.parse_expr(BindingPower::Lowest)?;
        self.expect(&Token::RParen)?;
    }
    pub(crate) fn parse_stmt(&mut self) -> Option<Stmt<'s>> {
        let public = self.consume_if(|t| matches!(t, Token::Pub));
        use Token::*;
        let tok = self.peek_next()?;
        match tok.inner {
            Const | Let => self.parse_var_decl(public),
            _ if public => {
                let msg = format!(
                    "public can only be followed by let or const, not {tok}",
                    tok = tok.inner
                );
                let span = tok.as_span();
                self.new_parse_error(span, "Invalid token after public")
                    .set_long_message(msg);
                None
            }
            At("use") => self.parse_use(),
            Return => self.parse_return(),
            LBrace => self.parse_block(),
            If => self.parse_if(),
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
