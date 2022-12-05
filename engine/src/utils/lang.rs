//! General-purpose language parsing infrastructure.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::hash::Hash;

use serde::Serialize;

use crate::vfs;

/// Ties a [`pest::Span`] to an entry in the [virtual file system](vfs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FileSpan<'inp> {
	#[serde(skip)]
	file: vfs::Handle,
	#[serde(with = "PestSpanSerde")]
	span: pest::Span<'inp>,
}

#[derive(Serialize)]
#[serde(remote = "pest::Span")]
struct PestSpanSerde<'inp> {
	#[serde(skip)]
	input: &'inp str,
	#[serde(getter = "pest::Span::start")]
	start: usize,
	#[serde(getter = "pest::Span::end")]
	end: usize,
}

impl<'inp> From<PestSpanSerde<'inp>> for pest::Span<'inp> {
	fn from(f: PestSpanSerde<'inp>) -> Self {
		pest::Span::new(f.input, f.start, f.end).unwrap()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Identifier<'inp>(pub FileSpan<'inp>);

impl<'inp> std::ops::Deref for Identifier<'inp> {
	type Target = FileSpan<'inp>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Hash for Identifier<'_> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.span.start().hash(state);
		self.span.end().hash(state);
	}
}

impl std::fmt::Display for Identifier<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0.span.as_str())
	}
}
