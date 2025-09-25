pub use logos::Lexer;
use memchr::memchr2;
use std::borrow::Cow;

use crate::Token;

pub fn unescape(c: char) -> Option<char> {
    Some(match c {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '0' => '\0',
        '\'' => '\'',
        '\"' => '\"',
        _ => return None,
    })
}

pub fn unescape_string<'s>(lex: &mut Lexer<'s, Token<'s>>) -> Option<Cow<'s, str>> {
    let remainder = lex.remainder();
    let first_important = memchr2(b'\\', b'"', remainder.as_bytes())?;
    // Early return on string without escapes
    if remainder.as_bytes()[first_important] == b'"' {
        lex.bump(first_important + 1);
        return Some(Cow::Borrowed(&remainder[..first_important]));
    }
    let mut out = String::from(&remainder[..first_important]);
    let mut i = first_important + 1;
    let escaped = remainder[i..].chars().next()?;
    out.push(unescape(escaped)?);
    i += escaped.len_utf8();
    loop {
        let haystack = &remainder[i..];
        let important = memchr2(b'\\', b'"', haystack.as_bytes())?;
        i += important;
        // Capture string component before escape/quote
        out.push_str(&haystack[..important]);
        if haystack.as_bytes()[important] == b'"' {
            lex.bump(i + 1);
            return Some(Cow::Owned(out));
        }
        let escaped = haystack[important + 1..].chars().next()?;
        i += 1;
        out.push(unescape(escaped)?);
        i += escaped.len_utf8();
    }
}
