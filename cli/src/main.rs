use diagnostics::{
    ErrorWithSource,
    render::{RenderContext, RenderableError},
};
use lexer::{Logos, SToken};

fn main() {
    let source = source::SourceFile::new(
        "examples/test.lx".into(),
        String::from(&include_str!("../../examples/test.lx")[1..]),
    );
    let lexer = lexer::Token::lexer(source.text())
        .spanned()
        .map(|r| match r {
            (Ok(token), span) => Ok(SToken::new(token, span)),
            (Err(()), span) => {
                Err(
                    ErrorWithSource::new(source.clone(), String::from("Failed to lex token"), span)
                        .into(),
                )
            }
        });
    let mut parser = parser::Parser::new(source.clone(), "test", lexer);
    let render_context = RenderContext::default();
    match parser.parse() {
        Ok(p) => {
            dbg!(p);
        }
        Err(err) => eprint!("{}", err.display(render_context)),
    }
}
