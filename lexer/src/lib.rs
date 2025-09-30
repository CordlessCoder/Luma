use crate::escapes::{unescape, unescape_string};
use crate::int::parse_int;
pub use logos::{Lexer, Logos};
use std::borrow::Cow;
use utils::Spanned;

mod display;
mod escapes;
mod int;

pub type SToken<'s> = Spanned<Token<'s>>;

#[derive(Debug, Clone, Logos, PartialEq)]
#[logos(skip "[ \r\n\t]*")]
#[logos(skip "//[^\n]*")]
pub enum Token<'s> {
    // preprocessor directives
    #[regex(r"@[A-Za-z0-9_]+", |lex| &lex.slice()[1..])]
    /// @... - preprocessor directive
    At(&'s str),

    // keywords
    #[token("elif")]
    /// elif
    Elif,
    #[token("return")]
    /// return
    Return,
    #[token("break")]
    /// break
    Break,
    #[token("continue")]
    /// continue
    Continue,
    #[token("struct")]
    /// struct
    Struct,
    #[token("enum")]
    /// enum
    Enum,
    #[token("pub")]
    /// pub
    Pub,
    #[token("void")]
    /// void
    Void,
    #[token("char")]
    /// char
    Char,
    #[token("str")]
    /// str
    Str,
    #[token("uint")]
    /// uint
    UInt,
    #[token("int")]
    /// int
    Int,
    #[token("float")]
    /// float
    Float,
    #[token("double")]
    /// double
    Double,
    #[token("bool")]
    /// bool
    Bool,
    #[token("output")]
    /// output
    Output,
    #[token("outputln")]
    /// outputln
    Outputln,
    #[token("const")]
    /// const
    Const,
    #[token("alloc")]
    /// alloc
    Alloc,
    #[token("free")]
    /// free
    Free,
    #[token("sizeof")]
    /// sizeof
    Sizeof,
    #[token("as")]
    /// as
    As,
    #[token("defer")]
    /// defer
    Defer,
    #[token("in")]
    /// in
    In,
    #[token("switch")]
    /// switch
    Switch,
    #[token("fn")]
    /// fn
    Fn,
    #[token("let")]
    /// let
    Let,
    #[token("if")]
    /// if
    If,
    #[token("else")]
    /// else
    Else,
    #[token("loop")]
    /// loop
    Loop,
    #[token("cast")]
    /// cast
    Cast,

    // literals
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    BoolLit(bool),
    #[regex("\"", unescape_string)]
    StringLit(Cow<'s, str>),
    #[regex(r"'\\?[^']'", (|lex: &mut Lexer<'_>| -> Option<char> {
        let text = lex.slice();
        let text = &text[1..text.len() - 1];
        let mut chars = text.chars();
        let c = chars.next().unwrap();
        if c != '\\' {
            return Some(c);
        };
        let c = chars.next().unwrap();
        unescape(c)
    }))]
    CharLiteral(char),
    #[regex(r"0x[0-9a-fA-F][0-9a-fA-F_]*", |lex| parse_int(16, &lex.slice()[2..]))]
    #[regex(r"0o[0-7_]+", |lex| parse_int(8, &lex.slice()[2..]))]
    #[regex(r"0b[0-1_]+", |lex| parse_int(2, &lex.slice()[2..]))]
    #[regex(r"[0-9][0-9_]*", |lex| parse_int(10, lex.slice()))]
    IntLit(u64),
    #[regex(r"\.\d+", |lex| lex.slice().parse().ok())]
    #[regex(r"\d+\.\d+", |lex| lex.slice().parse().ok())]
    FloatLit(f64),
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident(&'s str),

    // operators
    #[token("~")]
    /// ~
    Tilde,
    #[token("?")]
    /// ?
    Question,
    #[token("<<")]
    /// <<
    LShift,
    #[token(">>")]
    /// >>
    RShift,
    #[token("=>")]
    /// =>
    RArrow,
    #[token("==")]
    /// ==
    EqEq,
    #[token("!=")]
    /// ==
    Ne,
    #[token("<=")]
    /// ==
    LtEq,
    #[token(">=")]
    /// ==
    GtEq,
    // #[token("+=")]
    // /// +=
    // PlusEq,
    // #[token("-=")]
    // /// -=
    // MinusEq,
    // #[token("*=")]
    // /// *=
    // StarEq,
    // #[token("/=")]
    // /// /=
    // SlashEq,
    #[token("&&")]
    /// &&
    And,
    #[token("||")]
    /// ||
    Or,
    #[token("&")]
    /// &
    Ampersand,
    #[token("^")]
    /// ^
    BitXor,
    #[token("|")]
    /// |
    BitOr,
    #[token("<")]
    /// <
    Lt,
    #[token(">")]
    /// >
    Gt,
    #[token("%")]
    /// %
    Percent,
    #[token("++")]
    /// ++
    PlusPlus,
    #[token("+")]
    /// +
    Plus,
    #[token("--")]
    /// --
    MinusMinus,
    #[token("-")]
    /// -
    Minus,
    #[token("*")]
    /// *
    Star,
    #[token("/")]
    /// /
    Slash,
    #[token("::")]
    /// ::
    ColonColon,
    #[token(":")]
    /// :
    Colon,
    #[token("(")]
    /// (
    LParen,
    #[token(")")]
    /// )
    RParen,
    #[token("{")]
    /// {
    LBrace,
    #[token("}")]
    /// }
    RBrace,
    #[token("[")]
    /// [
    LBracket,
    #[token("]")]
    /// ]
    RBracket,
    #[token(",")]
    /// ,
    Comma,
    #[token(";")]
    /// ;
    Semicolon,
    #[token("..")]
    /// ..
    Range,
    #[token(".")]
    /// .
    Dot,
    #[token("=")]
    /// =
    Eq,
    #[token("!")]
    /// !
    Not,
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use pretty_assertions::assert_eq;
    use std::borrow::Cow;

    use crate::Token;

    fn string<'s>(text: impl Into<Cow<'s, str>>) -> Token<'s> {
        Token::StringLit(text.into())
    }

    #[test]
    fn lex_example() {
        use Token::*;
        let example = include_str!("../../examples/test.lx");
        let tokens: Vec<_> = Token::lexer(example).collect::<Result<_, _>>().unwrap();
        assert_eq!(
            &tokens,
            [
                // @module "main"
                At("module"),
                Ident("main"),
                // @use "math" as math
                At("use"),
                Ident("math"),
                As,
                Ident("math"),
                // pub const main = fn () int {
                Pub,
                Const,
                Ident("main"),
                Eq,
                Fn,
                LParen,
                RParen,
                Int,
                LBrace,
                //     output("Addition: ", math.add(5, 3), "\n");
                Output,
                LParen,
                string("Addition: "),
                Comma,
                Ident("math"),
                Dot,
                Ident("add"),
                LParen,
                IntLit(5),
                Comma,
                IntLit(3),
                RParen,
                Comma,
                string("\n"),
                RParen,
                Semicolon,
                //     output("Subtraction: ", math.subtract(5, 3), "\n");
                Output,
                LParen,
                string("Subtraction: "),
                Comma,
                Ident("math"),
                Dot,
                Ident("subtract"),
                LParen,
                IntLit(5),
                Comma,
                IntLit(3),
                RParen,
                Comma,
                string("\n"),
                RParen,
                Semicolon,
                //     output("Multiplication: ", math.multiply(5, 3), "\n");
                Output,
                LParen,
                string("Multiplication: "),
                Comma,
                Ident("math"),
                Dot,
                Ident("multiply"),
                LParen,
                IntLit(5),
                Comma,
                IntLit(3),
                RParen,
                Comma,
                string("\n"),
                RParen,
                Semicolon,
                //     output("Division: ", math.divide(5, 0), "\n");
                Output,
                LParen,
                string("Division: "),
                Comma,
                Ident("math"),
                Dot,
                Ident("divide"),
                LParen,
                IntLit(5),
                Comma,
                IntLit(0),
                RParen,
                Comma,
                string("\n"),
                RParen,
                Semicolon,
                //     return 0;
                Return,
                IntLit(0),
                Semicolon,
                // }
                RBrace,
            ]
            .as_slice()
        );
    }
}
