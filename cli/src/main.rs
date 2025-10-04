use ast::tree::{TreeCtx, TreeDisplay};
use diagnostics::{
    ErrorComponent,
    render::{RenderContext, RenderableError},
};
use lexer::{Logos, SToken};

use std::io::{self, stdout};
use std::{fmt, path::Path};

struct FmtToIoWrite<W: io::Write>(pub W);
impl<W: io::Write> fmt::Write for FmtToIoWrite<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

fn parse_and_ast_print(path: impl AsRef<Path>) {
    let path = path.as_ref();
    let source = source::SourceFile::new(
        path.to_string_lossy().to_string(),
        std::fs::read_to_string(path).unwrap(),
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

fn main() {
    for arg in std::env::args_os().skip(1) {
        parse_and_ast_print(arg);
    }
}
