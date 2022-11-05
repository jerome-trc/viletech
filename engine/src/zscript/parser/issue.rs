/*

Copyright (C) 2021-2022 Jessica "Gutawer" Russell

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

use std::{
	collections::{BTreeMap, HashMap},
	ops::Range,
};

use serde::Serialize;
use unicode_width::UnicodeWidthChar;
use vec1::Vec1;

use super::{
	fs::{FileIndex, Files},
	Span,
};

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum Level {
	Warning,
	Error,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Issue {
	pub level: Level,
	pub msg: String,
	pub main_spans: Vec1<Span>,
	pub info_spans: Vec<Span>,
}

impl Issue {
	fn get_lines(&self, files: &Files) -> ParsingErrorLines {
		let mut ret = ParsingErrorLines {
			file_lines: HashMap::new(),
		};

		for s in self.main_spans.iter() {
			let file = &files[s.file];
			let span_lines = get_span_lines(&file.lines, *s);
			for l in span_lines.lines {
				let ranges = ret
					.file_lines
					.entry(s.file)
					.or_insert_with(BTreeMap::new)
					.entry(l.line)
					.or_insert_with(ErrorLineRanges::default);

				ranges.main.push(l.range);
			}
		}

		for s in self.info_spans.iter() {
			let file = &files[s.file];
			let span_lines = get_span_lines(&file.lines, *s);
			for l in span_lines.lines {
				let ranges = ret
					.file_lines
					.entry(s.file)
					.or_insert_with(BTreeMap::new)
					.entry(l.line)
					.or_insert_with(ErrorLineRanges::default);

				ranges.info.push(l.range);
			}
		}
		ret
	}

	pub fn repr(&self, files: &Files) -> String {
		let s = match self.level {
			Level::Warning => "warning",
			Level::Error => "error",
		};
		let mut ret = format!("{}: {}\n", s, self.msg);
		let err_lines = self.get_lines(files);
		ret += &err_lines.repr(files);
		ret
	}
}

pub trait ToDisplayedErrors {
	fn to_displayed_errors(&self, files: &Files) -> DisplayedParsingErrors;
}

impl ToDisplayedErrors for Issue {
	fn to_displayed_errors(&self, files: &Files) -> DisplayedParsingErrors {
		DisplayedParsingErrors(self.repr(files))
	}
}

impl ToDisplayedErrors for Vec<Issue> {
	fn to_displayed_errors(&self, files: &Files) -> DisplayedParsingErrors {
		let mut sorted = self.clone();
		sort_errs(&mut sorted);
		DisplayedParsingErrors(repr_errs(files, &sorted))
	}
}

#[derive(Debug)]
pub struct DisplayedParsingErrors(String);

impl std::fmt::Display for DisplayedParsingErrors {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::error::Error for DisplayedParsingErrors {}

pub fn sort_errs(errs: &mut [Issue]) {
	errs.sort_unstable_by_key(|err| *err.main_spans.iter().min().unwrap());
}

pub fn repr_errs(files: &Files, errs: &[Issue]) -> String {
	let mut ret = "".to_string();
	for e in errs.iter() {
		ret += &format!("{}\n", e.repr(files));
	}
	ret
}

#[derive(Default)]
struct ErrorLineRanges {
	main: Vec<Range<usize>>,
	info: Vec<Range<usize>>,
}

struct ParsingErrorLines {
	file_lines: HashMap<FileIndex, BTreeMap<usize, ErrorLineRanges>>,
}

impl ParsingErrorLines {
	fn repr(&self, files: &Files) -> String {
		let mut ret = "".to_string();
		for (file, lines_ranges) in self.file_lines.iter() {
			let file = &files[*file];
			let text = &*file.text;
			let max_line = *lines_ranges.keys().max().unwrap();
			let line_indicator_length = format!("{}", max_line + 1).len();
			let mut last_line = None;
			ret += &format!(
				"{}--> {}\n",
				" ".repeat(line_indicator_length),
				file.filename
			);
			ret += &format!("{} |\n", " ".repeat(line_indicator_length));
			for (l, v) in lines_ranges.iter() {
				if let Some(last) = last_line {
					match l.cmp(&(last + 2)) {
						std::cmp::Ordering::Equal => {
							let last_line_range = &file.lines[l - 1];
							let last_line_text = &text[last_line_range.start..last_line_range.end];
							ret += &format!(
								"{:>width$} |{}\n",
								l - 1 + 1,
								last_line_text.replace('\t', "    "),
								width = line_indicator_length
							);
						}
						std::cmp::Ordering::Greater => {
							ret += "...\n";
						}
						_ => {}
					}
				}
				last_line = Some(l);
				let line = &file.lines[*l];
				let line_text = &text[line.start..line.end];
				let mut p = vec![0u8; line_text.len() + 1];
				for r in v.info.iter() {
					let r = if r.start == r.end {
						r.start..(r.start + 1)
					} else {
						r.clone()
					};
					for x in r.clone() {
						p[x] = 1;
					}
				}
				for r in v.main.iter() {
					let r = if r.start == r.end {
						r.start..(r.start + 1)
					} else {
						r.clone()
					};
					for x in r.clone() {
						p[x] = 2;
					}
				}
				ret += &format!(
					"{:>width$} |{}\n",
					l + 1,
					line_text.replace('\t', "    "),
					width = line_indicator_length
				);
				ret += &format!("{} |", " ".repeat(line_indicator_length));
				for (i, c) in line_text.char_indices() {
					ret += &match p[i] {
						2 => "^",
						1 => "-",
						_ => " ",
					}
					.repeat(if c == '\t' {
						4
					} else {
						UnicodeWidthChar::width(c).unwrap_or(1)
					});
				}
				ret += match p[p.len() - 1] {
					2 => "^",
					1 => "-",
					_ => " ",
				};
				ret += "\n";
			}
		}
		ret
	}
}

fn get_line(lines: &[Range<usize>], start: usize) -> usize {
	match lines.binary_search_by_key(&start, |Range { start, .. }| *start) {
		Ok(l) => l,
		Err(l) => l - 1,
	}
}

struct LineInfo {
	line: usize,
	range: Range<usize>,
}

struct SpanLines {
	lines: Vec<LineInfo>,
}

fn get_span_lines(lines: &[Range<usize>], span: Span) -> SpanLines {
	let mut line_index = get_line(lines, span.start);
	let mut ret = SpanLines { lines: vec![] };
	loop {
		let line = &lines[line_index];
		let col = if line.start > span.start {
			0
		} else {
			span.start - line.start
		};
		if span.end > line.end {
			let end_col = line.len();
			ret.lines.push(LineInfo {
				line: line_index,
				range: col..end_col,
			});
			line_index += 1;
			continue;
		}
		let end_col = span.end - line.start;
		ret.lines.push(LineInfo {
			line: line_index,
			range: col..end_col,
		});
		break;
	}
	ret
}
