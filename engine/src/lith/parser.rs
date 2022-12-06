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

use std::collections::VecDeque;

use pest::{iterators::Pair, Parser as PestParser};
use serde::Serialize;
use vec1::Vec1;

use crate::utils::lang::FileSpan;

type LexError = Box<pest::error::Error<Rule>>;

#[derive(pest_derive::Parser)]
#[grammar = "lith/lex.pest"]
struct Lexer;

pub fn parse<'inp>(input: &'inp str) -> Result<ParseOutput<'inp>, LexError> {
	let token_stream = Lexer::parse(Rule::TokenStream, input)?
		.next()
		.unwrap()
		.into_inner();

	let _tq = VecDeque::<Pair<Rule>>::with_capacity(8);
	let issues = Vec::<Issue>::default();

	for token in token_stream {
		match token.as_rule() {
			_ => {}
		}
	}

	Ok(ParseOutput { issues })
}

#[derive(Debug)]
pub struct ParseOutput<'inp> {
	pub issues: Vec<Issue<'inp>>,
}

#[derive(Debug)]
pub struct Issue<'inp> {
	pub level: IssueLevel,
	pub msg: String,
	pub main_spans: Vec1<FileSpan<'inp>>,
	pub info_spans: Vec<FileSpan<'inp>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum IssueLevel {
	Warning,
	Error,
}
