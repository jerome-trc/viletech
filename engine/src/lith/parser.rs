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

use pest::{iterators::Pairs, Parser as PestParser};

#[derive(pest_derive::Parser)]
#[grammar = "lith/lex.pest"]
struct Lexer;

#[allow(unused)]
fn lex(input: &str) -> Result<Pairs<Rule>, Box<pest::error::Error<Rule>>> {
	Ok(Lexer::parse(Rule::TokenStream, input)?
		.next()
		.unwrap()
		.into_inner())
}
