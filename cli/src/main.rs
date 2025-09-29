use diagnostics::{
    ErrorComponent,
    render::{RenderContext, RenderableError},
};
use lexer::{Logos, SToken};

fn main() {
    let source = source::SourceFile::new(
        "examples/test.lx".into(),
        String::from(include_str!("../../examples/test.lx")),
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
    dbg!(module);
}
