//! String inspection and manipulation tools.

use std::{
	borrow::Borrow,
	hash::{Hash, Hasher},
};

use arrayvec::ArrayString;

use crate::lazy_regex;

/// A "ZDoom string", which compares and hashes case-insensitively.
#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct ZString<T: Borrow<str>>(pub T);

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
								.map_err(|_| UnescapeError::Unrecognized)? as char
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

/// Extracts a version string from what will almost always be a file stem,
/// using a search pattern based off the most common versioning conventions used
/// in ZDoom modding. If the returned option is `None`, the given string is unmodified.
#[must_use]
pub fn version_from_string(string: &mut String) -> Option<String> {
	match lazy_regex!(
		r"(?x)
		(?:[VR]|[\s\-_][VvRr]|[\s\-_\.])\d{1,}
		(?:[\._\-]\d{1,})*
		(?:[\._\-]\d{1,})*
		[A-Za-z]*[\._\-]*
		[A-Za-z0-9]*
		$"
	)
	.find(string)
	{
		Some(m) => {
			const TO_TRIM: [char; 3] = [' ', '_', '-'];
			let ret = m.as_str().trim_matches(&TO_TRIM[..]).to_string();
			string.replace_range(m.range(), "");
			Some(ret)
		}
		None => None,
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn version_from_string() {
		let mut input = [
			"DOOM".to_string(),
			"DOOM2".to_string(),
			"weighmedown_1.2.0".to_string(),
			"none an island V1.0".to_string(),
			"bitter_arcsV0.9.3".to_string(),
			"stuck-in-the-system_v3.1".to_string(),
			"555-5555 v5a".to_string(),
			"yesterdays_pain_6-19-2022".to_string(),
			"i-am_a dagger_1.3".to_string(),
			"BROKEN_MANTRA_R3.1c".to_string(),
			"There Is Still Time 0.3tf1".to_string(),
			"setmefree-4.7.1c".to_string(),
			"Outoftheframe_1_7_0b".to_string(),
			"a c i d r a i n_716".to_string(),
		];

		let expected = [
			("DOOM", ""),
			("DOOM2", ""),
			("weighmedown", "1.2.0"),
			("none an island", "V1.0"),
			("bitter_arcs", "V0.9.3"),
			("stuck-in-the-system", "v3.1"),
			("555-5555", "v5a"),
			("yesterdays_pain", "6-19-2022"),
			("i-am_a dagger", "1.3"),
			("BROKEN_MANTRA", "R3.1c"),
			("There Is Still Time", "0.3tf1"),
			("setmefree", "4.7.1c"),
			("Outoftheframe", "1_7_0b"),
			("a c i d r a i n", "716"),
		];

		for (i, string) in input.iter_mut().enumerate() {
			let res = super::version_from_string(string);

			if expected[i].1.is_empty() {
				assert!(
					res.is_none(),
					"[{i}] expected nothing; returned: {}",
					res.unwrap()
				);
			} else {
				assert!(
					string == expected[i].0,
					"[{i}] expected modified string: {} - actual output: {}",
					expected[i].0,
					string
				);

				let vers = res.unwrap();

				assert!(
					vers == expected[i].1,
					"[{i}] expected return value: {} - actual return value: {}",
					expected[i].1,
					vers
				);
			}
		}
	}
}
