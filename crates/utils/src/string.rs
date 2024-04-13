//! String inspection and manipulation tools.

use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
};

use arrayvec::ArrayString;

/// A generic helper for providing a [`std::fmt::Display`] implementation to `T`
/// when `T` requires context (`C`) to be formatted.
pub struct AnyDisplay<'t, 'c, T, C>(
    pub &'t T,
    pub &'c C,
    pub fn(&T, &C, &mut std::fmt::Formatter) -> std::fmt::Result,
);

impl<'t, 'c, T, C> std::fmt::Display for AnyDisplay<'t, 'c, T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.2)(self.0, self.1, f)
    }
}

/// A "ZDoom string", which compares and hashes case-insensitively.
#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct ZString<T>(pub T);

impl<T: Borrow<str>> ZString<T> {
    /// This function does nothing special, but exists to allow succinct creation
    /// of these objects when using type aliases, which cannot be used in constructors.
    #[must_use]
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<A, B> PartialEq<ZString<B>> for ZString<A>
where
    A: Borrow<str>,
    B: Borrow<str>,
{
    fn eq(&self, other: &ZString<B>) -> bool {
        let a: &str = std::borrow::Borrow::borrow(&self.0);
        let b: &str = std::borrow::Borrow::borrow(&other.0);
        a.eq_ignore_ascii_case(b)
    }
}

impl<T: Borrow<str>> PartialEq<str> for ZString<T> {
    fn eq(&self, other: &str) -> bool {
        let a: &str = std::borrow::Borrow::borrow(&self.0);
        a.eq_ignore_ascii_case(other)
    }
}

impl<T: Borrow<str>> Eq for ZString<T> {}

impl<T: Borrow<str>> Hash for ZString<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s: &str = std::borrow::Borrow::borrow(&self.0);

        for c in s.chars() {
            c.to_ascii_lowercase().hash(state);
        }
    }
}

impl<T: Borrow<str>> Borrow<str> for ZString<T> {
    fn borrow(&self) -> &str {
        std::borrow::Borrow::borrow(&self.0)
    }
}

impl<T: Borrow<str>> std::ops::Deref for ZString<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Borrow<str>> std::fmt::Display for ZString<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::borrow::Borrow::borrow(&self.0))
    }
}

/// e.g. 999 bytes becomes `999 B`, while 1536 bytes becomes `1.5 KB`, and so on.
/// Will only subdivide into gigabytes and no further.
#[must_use]
pub fn subdivide_file_len(len: usize) -> String {
    if len == 0 {
        return "0 B".to_string();
    }

    let mut len = len as f32;
    let mut unit = "B";

    if len > 1024.0 {
        len /= 1024.0;
        unit = "KB";
    } else {
        return format!("{len:.2} {unit}");
    }

    if len > 1024.0 {
        len /= 1024.0;
        unit = "MB";
    }

    if len > 1024.0 {
        len /= 1024.0;
        unit = "GB";
    }

    format!("{len:.2} {unit}")
}

/// Shortcut for `string.get(..string.chars().count().min(chars)).unwrap()`.
#[must_use]
pub fn subslice(string: &str, chars: usize) -> &str {
    string.get(..string.chars().count().min(chars)).unwrap()
}

#[must_use]
pub fn is_only_whitespace(string: &str) -> bool {
    !string.chars().any(|c| !c.is_whitespace())
}

/// For use with a parser span (i.e. ZScript), so returns not only a line
/// but also which line in the text it is out of each line (starting at 0).
#[must_use]
pub fn line_from_char_index(string: &str, index: usize) -> Option<(&str, usize)> {
    debug_assert!(index < string.chars().count());

    let lines = string.split_inclusive(&['\n', '\r']);
    let mut c = index;
    let mut i = 0;

    for line in lines {
        let count = line.chars().count();

        if c < count {
            return Some((line, i));
        } else {
            c -= count;
            i += 1;
        }
    }

    None
}

/// Taken from the [`enquote`] crate courtesy of Christopher Knight
/// ([@reujab](https://github.com/reujab)) and used under [The Unlicense].
/// Modified for parsing code point literals; performs no allocations
/// internally and emits a single character.
///
/// This specialization means that reaching the end of the given string unexpectedly
/// causes a panic, as does supplying a `string` longer than 10 code points.
///
/// [`enquote`]: https://github.com/reujab/enquote/blob/master/src/lib.rs
/// [The Unlicense]: https://github.com/reujab/enquote/blob/master/unlicense
pub fn unescape_char(string: &str) -> Result<char, UnescapeError> {
    /// `Iterator::take` cannot be used because it consumes the iterator.
    fn take<I: Iterator<Item = char>>(iterator: &mut I, n: usize) -> ArrayString<10> {
        let mut s = ArrayString::<10>::default();

        for _ in 0..n {
            s.push(iterator.next().unwrap_or_default());
        }

        s
    }

    fn decode_unicode(code_point: &str) -> Result<char, UnescapeError> {
        match u32::from_str_radix(code_point, 16) {
            Err(_) => Err(UnescapeError::Unrecognized),
            Ok(n) => std::char::from_u32(n).ok_or(UnescapeError::InvalidUtf8),
        }
    }

    let mut chars = string.chars();
    let mut ret = ArrayString::<10>::new();

    loop {
        match chars.next() {
            None => break,
            Some(c) => ret.push(match c {
                '\\' => match chars.next() {
                    Some(c) => match c {
                        _ if c == '\\' || c == '"' || c == '\'' || c == '`' => c,
                        'a' => '\x07',
                        'b' => '\x08',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'v' => '\x0b',
                        // Octal
                        '0'..='9' => {
                            let mut octal = ArrayString::<10>::default();
                            octal.push(c);
                            octal.push_str(&take(&mut chars, 2));

                            u8::from_str_radix(&octal, 8)
                                .map_err(|_| UnescapeError::Unrecognized)?
                                as char
                        }
                        // Hexadecimal
                        'x' => {
                            let hex = take(&mut chars, 2);

                            u8::from_str_radix(&hex, 16).map_err(|_| UnescapeError::Unrecognized)?
                                as char
                        }
                        // Unicode
                        'u' => decode_unicode(&take(&mut chars, 4))?,
                        'U' => decode_unicode(&take(&mut chars, 8))?,
                        _ => return Err(UnescapeError::Unrecognized),
                    },
                    None => unreachable!(),
                },
                _ => c,
            }),
        }
    }

    Ok(ret.parse::<char>().unwrap())
}

/// See [`unescape_char`].
#[derive(Debug)]
pub enum UnescapeError {
    Unrecognized,
    InvalidUtf8,
}

impl std::error::Error for UnescapeError {}

impl std::fmt::Display for UnescapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unrecognized => write!(f, "encountered an unrecognized escape character"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 code point"),
        }
    }
}
