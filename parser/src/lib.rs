use diagnostics::{AggregateError, ErrorComponent};
use lexer::SToken;
use source::SourceFile;
use std::collections::VecDeque;
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
    // pub fn parse(&mut self) -> Result<Program<'s>, AggregateError {
    //     todo!()
    //     let mut m = Program {
    //         filename: self.filename,
    //         statements: Vec::new(),
    //     };
    //     while self.peek_one().is_some() {
    //         let Some(stmt) = self.parse_statement() else {
    //             continue;
    //         };
    //         m.statements.push(stmt);
    //     }
    //     if !self.lexer_errors.is_empty() {
    //         return Err(std::mem::take(&mut self.lexer_errors));
    //     }
    //     if !self.errors.is_empty() {
    //         return Err(std::mem::take(&mut self.errors));
    //     }
    //     Ok(m)
    // }
}
