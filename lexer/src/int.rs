use std::borrow::Cow;

fn remove_underscores<'s>(s: &'s str) -> Cow<'s, str> {
    let Some(first_underscore) = memchr::memchr(b'_', s.as_bytes()) else {
        return s.into();
    };
    let mut out = s[..first_underscore].to_string();
    let mut rest = &s[first_underscore + 1..];
    while let Some(first_underscore) = memchr::memchr(b'_', rest.as_bytes()) {
        out.push_str(&rest[..first_underscore]);
        rest = &rest[first_underscore + 1..];
    }
    out.push_str(rest);
    out.into()
}

pub fn parse_int(base: u32, text: &str) -> Option<u64> {
    u64::from_str_radix(&remove_underscores(text), base).ok()
}
