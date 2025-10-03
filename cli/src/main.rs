use ast::tree::{TreeCtx, TreeDisplay};
use diagnostics::{
    ErrorComponent,
    render::{RenderContext, RenderableError},
};
use lexer::{Logos, SToken};

use std::fmt;
use std::io::{self, stdout};

struct FmtToIoWrite<W: io::Write>(pub W);
impl<W: io::Write> fmt::Write for FmtToIoWrite<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

fn main() {
    let source = source::SourceFile::new(
        "examples/math.lx".into(),
        String::from(include_str!("../../examples/math.lx")),
    );
    let lexer = lexer::Token::lexer(source.text())
        .spanned()
        .map(|r| match r {
            (Ok(token), span) => Ok(SToken::new(token, span)),
            (Err(()), span) => Err(ErrorComponent::new(
                source.clone(),
                String::from("Failed to lex token"),
                span,
            )),
        });
    let mut parser = parser::Parser::new(source.clone(), lexer);
    let render_context = RenderContext::default();
    let (module, errors) = parser.parse();
    eprint!("{}", errors.display(render_context));

    let writer = stdout();
    let mut writer = FmtToIoWrite(writer);
    let mut ctx = TreeCtx::new();
    module.fmt_tree(&mut ctx, &mut writer).unwrap();
}
