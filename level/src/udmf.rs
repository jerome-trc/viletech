//! A parser for the [Universal Doom Map Format](https://doomwiki.org/wiki/UDMF).

// NOTE: Block field readers use const slices of strings and function pointers to
// run comparisons and mutate level parts; Compiler Explorer says these get
// optimized to inline tests at `opt-level=3` as of 1.69.0. If you're reading this
// a year or two from now, test again, and see if the GCC backend does the same.

mod linedef;
mod sectordef;
mod sidedef;
mod thingdef;

use std::num::{ParseFloatError, ParseIntError};

use chumsky::{self, primitive, span::SimpleSpan, text, util::MaybeRef, IterParser, Parser};
use util::lazy_regex;

use crate::repr::{
	Vertex,
	{Level, LevelFormat, LineDef, LineFlags, Sector, SideDef, Thing, ThingFlags, UdmfNamespace},
};

pub fn parse_textmap(source: &str) -> Result<Level, Vec<Error>> {
	if source.len() < 128 {
		return Err(vec![Error::TextmapTooShort]);
	}

	let ns_slice = &source[..128];

	let (namespace, ns_end) = if let Some(captures) = lazy_regex!(
		"(?i)namespace = \"(doom|heretic|hexen|strife|zdoom|eternity|vavoom|zdoomtranslated)\";"
	)
	.captures(ns_slice)
	{
		let capture = captures.get(1).unwrap();
		let ns_str = capture.as_str();

		let ns = if ns_str.eq_ignore_ascii_case("doom") {
			UdmfNamespace::Doom
		} else if ns_str.eq_ignore_ascii_case("heretic") {
			UdmfNamespace::Heretic
		} else if ns_str.eq_ignore_ascii_case("hexen") {
			UdmfNamespace::Hexen
		} else if ns_str.eq_ignore_ascii_case("strife") {
			UdmfNamespace::Strife
		} else if ns_str.eq_ignore_ascii_case("zdoom") {
			UdmfNamespace::ZDoom
		} else if ns_str.eq_ignore_ascii_case("eternity") {
			UdmfNamespace::Eternity
		} else if ns_str.eq_ignore_ascii_case("vavoom") {
			UdmfNamespace::Vavoom
		} else if ns_str.eq_ignore_ascii_case("zdoomtranslated") {
			UdmfNamespace::ZDoomTranslated
		} else {
			return Err(vec![Error::InvalidNamespace(ns_str.to_string())]);
		};

		(ns, capture.end() + 2)
	} else {
		return Err(vec![Error::NoNamespace]);
	};

	let source = &source[ns_end..];

	let mut level = Level::new(LevelFormat::Udmf(namespace));

	let result = parser().parse_with_state(source, &mut level);
	let (output, errors) = result.into_output_errors();

	if errors.is_empty() && output.is_some() {
		level.bounds = Level::bounds(&level.geom.vertices);
		Ok(level)
	} else {
		Err(errors)
	}
}

#[derive(Debug)]
pub enum Error {
	InvalidNamespace(String),
	Lex {
		span: SimpleSpan,
		token: Option<MaybeRef<'static, char>>,
	},
	NoNamespace,
	ParseFloat {
		inner: ParseFloatError,
		input: String,
	},
	ParseInt {
		inner: ParseIntError,
		input: String,
	},
	TextmapEmpty,
	TextmapTooShort,
	UnknownVertDefField(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidNamespace(namespace) => {
				write!(f, "`{namespace}` is not a valid UDMF namespace.")
			}
			Self::Lex { span, token } => {
				if let Some(tok) = token {
					let c = tok.into_inner();
					write!(f, "Unexpected token `{c}` at {span}.")
				} else {
					write!(f, "Unexpected end of input.")
				}
			}
			Self::NoNamespace => {
				write!(f, "TEXTMAP is missing a UDMF namespace statement.")
			}
			Self::ParseFloat { inner, input } => {
				write!(
					f,
					"Failed to parse float from: `{input}`\r\n\tReason: {inner}"
				)
			}
			Self::ParseInt { inner, input } => {
				write!(
					f,
					"Failed to parse integer from: `{input}`\r\n\tReason: {inner}"
				)
			}
			Self::TextmapEmpty => {
				write!(f, "TEXTMAP is empty.")
			}
			Self::TextmapTooShort => {
				write!(f, "TEXTMAP is too short for any meaningful content.")
			}
			Self::UnknownVertDefField(name) => {
				write!(f, "TEXTMAP contains vertex with unknown field: `{name}`")
			}
		}
	}
}

impl<'a> chumsky::error::Error<'a, &'a str> for Error {
	fn expected_found<E: IntoIterator<Item = Option<chumsky::util::MaybeRef<'a, char>>>>(
		_: E,
		found: Option<chumsky::util::MaybeRef<'a, char>>,
		span: SimpleSpan,
	) -> Self {
		Self::Lex {
			span,
			token: found.map(|mref| mref.into_owned()),
		}
	}
}

// Details /////////////////////////////////////////////////////////////////////

type Extra<'i> = chumsky::extra::Full<Error, Level, ()>;

fn parser<'i>() -> impl Parser<'i, &'i str, (), Extra<'i>> + Clone {
	let dec_digit = primitive::one_of("0123456789");
	let hex_digit = primitive::one_of("0123456789abcdefABCDEF");
	let wsp = primitive::one_of([' ', '\r', '\n', '\t'])
		.repeated()
		.at_least(1)
		.slice();
	let c_comment = primitive::just("/*")
		.then(
			primitive::any()
				.and_is(primitive::just("*/").not())
				.repeated(),
		)
		.then(primitive::just("*/"))
		.slice();
	let cpp_comment = primitive::just("//")
		.then(primitive::any().and_is(text::newline().not()).repeated())
		.slice();

	// (RAT) The spec prescribes the following grammar for integer literals:
	// `integer := [+-]?[1-9]+[0-9]* | 0[0-9]+ | 0x[0-9A-Fa-f]+`
	// But this can never match the literal `0`, so I assume it's incorrect.
	let dec_with_sign = primitive::group((
		primitive::one_of(['+', '-']).or_not(),
		primitive::just('0').repeated(),
		primitive::one_of("123456789"),
		dec_digit.repeated(),
	))
	.slice();

	let hex = primitive::group((primitive::just("0x"), hex_digit.repeated().at_least(1))).slice();

	let int = primitive::choice((dec_with_sign, primitive::just('0').slice(), hex)).slice();

	let float = primitive::group((
		primitive::one_of(['+', '-']).or_not(),
		dec_digit.repeated().at_least(1),
		primitive::just('.'),
		dec_digit.repeated(),
		primitive::group((
			primitive::one_of(['e', 'E']),
			primitive::one_of(['+', '-']).or_not(),
			dec_digit.repeated().at_least(1),
		))
		.or_not(),
	))
	.slice();

	let string = primitive::group((
		primitive::just('"'),
		primitive::none_of(['"', '\\']).repeated(),
		primitive::group((
			primitive::just('\\'),
			primitive::any(),
			primitive::none_of(['"', '\\']).repeated(),
		))
		.repeated(),
		primitive::just('"'),
	))
	.slice();

	let value = primitive::choice((
		primitive::just("true").map_slice(|s| (s, Literal::True)),
		primitive::just("false").map_slice(|s| (s, Literal::False)),
		string.map_slice(|s| (s, Literal::String(s))),
		float.map_slice(|s| (s, Literal::Float(s))),
		int.map_slice(|s| (s, Literal::Int(s))),
	));

	let field = primitive::group((
		text::ident(),
		primitive::just('=').padded().ignored(),
		value,
		primitive::just(';').padded().ignored(),
	))
	.map(|f| KeyValPair {
		key: f.0,
		val: f.2 .0,
		kind: f.2 .1,
	});

	let linedef = primitive::group((
		primitive::just("linedef")
			.map_with_state(|_, _, level: &mut Level| {
				level.geom.linedefs.push(LineDef {
					udmf_id: -1,
					vert_start: usize::MAX,
					vert_end: usize::MAX,
					flags: LineFlags::empty(),
					special: 0,
					trigger: 0,
					args: [0; 5],
					side_right: usize::MAX,
					side_left: None,
				});
			})
			.padded(),
		primitive::just('{').padded(),
		field
			.try_map_with_state(|kvp: KeyValPair, _, level: &mut Level| {
				linedef::read_linedef_field(kvp, level)
			})
			.padded()
			.repeated(),
		primitive::just('}').padded(),
	))
	.try_map_with_state(|_, _, _| {
		// TODO: Sanity checks.
		Ok(())
	});

	let thingdef = primitive::group((
		primitive::just("thing")
			.map_with_state(|_, _, level: &mut Level| {
				level.things.push(Thing {
					tid: 0,
					ed_num: 0,
					pos: glam::vec3(0.0, 0.0, 0.0),
					angle: 0,
					flags: ThingFlags::empty(),
					args: [0; 5],
				});
			})
			.padded(),
		primitive::just('{').padded(),
		field
			.try_map_with_state(|kvp: KeyValPair, _, level: &mut Level| {
				thingdef::read_thingdef_field(kvp, level)
			})
			.padded()
			.repeated(),
		primitive::just('}').padded(),
	))
	.try_map_with_state(|_, _, _| {
		// TODO: Sanity checks.
		Ok(())
	});

	let sectordef = primitive::group((
		primitive::just("sector")
			.map_with_state(|_, _, level: &mut Level| {
				level.geom.sectors.push(Sector {
					udmf_id: i32::MAX,
					height_floor: 0.0,
					height_ceil: 0.0,
					tex_floor: None,
					tex_ceil: None,
					light_level: 0,
					special: 0,
					trigger: 0,
				});
			})
			.padded(),
		primitive::just('{').padded(),
		field
			.try_map_with_state(|kvp: KeyValPair, _, level: &mut Level| {
				sectordef::read_sectordef_field(kvp, level)
			})
			.padded()
			.repeated(),
		primitive::just('}').padded(),
	))
	.try_map_with_state(|_, _, _| {
		// TODO: Sanity checks.
		Ok(())
	});

	let sidedef = primitive::group((
		primitive::just("sidedef")
			.map_with_state(|_, _, level: &mut Level| {
				level.geom.sidedefs.push(SideDef {
					offset: glam::IVec2::default(),
					tex_top: None,
					tex_bottom: None,
					tex_mid: None,
					sector: usize::MAX,
				});
			})
			.padded(),
		primitive::just('{').padded(),
		field
			.try_map_with_state(|kvp: KeyValPair, _, level: &mut Level| {
				sidedef::read_sidedef_field(kvp, level)
			})
			.padded()
			.repeated(),
		primitive::just('}').padded(),
	))
	.try_map_with_state(|_, _, _| {
		// TODO: Sanity checks.
		Ok(())
	});

	let vertdef = primitive::group((
		primitive::just("vertex")
			.map_with_state(|_, _, level: &mut Level| {
				level.geom.vertices.push(Vertex(glam::Vec4::default()));
			})
			.padded(),
		primitive::just('{').padded(),
		field
			.try_map_with_state(|kvp: KeyValPair, _, level: &mut Level| {
				let vertdef = level.geom.vertices.last_mut().unwrap();

				let val = kvp.val.parse::<f64>().map_err(|err| Error::ParseFloat {
					inner: err,
					input: kvp.val.to_string(),
				})?;

				if kvp.key.eq_ignore_ascii_case("x") {
					vertdef.x = val as f32;
				} else if kvp.key.eq_ignore_ascii_case("y") {
					vertdef.y = val as f32;
				} else if kvp.key.eq_ignore_ascii_case("zfloor") {
					*vertdef.bottom_mut() = val as f32;
				} else if kvp.key.eq_ignore_ascii_case("zceiling") {
					*vertdef.top_mut() = val as f32;
				} else {
					return Err(Error::UnknownVertDefField(kvp.key.to_string()));
				}

				Ok(())
			})
			.padded()
			.repeated(),
		primitive::just('}').padded(),
	))
	.map(|_| ());

	primitive::choice((
		vertdef,
		linedef,
		sectordef,
		sidedef,
		thingdef,
		wsp.ignored(),
		c_comment.ignored(),
		cpp_comment.ignored(),
	))
	.repeated()
	.collect::<()>()
	.recover_with(chumsky::recovery::via_parser(
		chumsky::recovery::nested_delimiters('{', '}', [], |_| ()),
	))
	.boxed()
}

#[derive(Debug)]
pub(self) struct KeyValPair<'i> {
	key: &'i str,
	val: &'i str,
	kind: Literal<'i>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Literal<'i> {
	True,
	False,
	String(&'i str),
	Float(&'i str),
	Int(&'i str),
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn block() {
		const SOURCE: &str = r#" thing {
			x = 1120.0;
			y = -1072.0;
			angle = 0;
			type = 28800;
			skill1 = true;
			skill2 = true;
			skill3 = true;
			skill4 = true;
			skill5 = true;
			single = true;
			coop = true;
			dm = true;
		} "#;

		let mut level = Level::new(LevelFormat::Udmf(UdmfNamespace::Doom));
		let result = parser().parse_with_state(SOURCE, &mut level);
		let (output, errors) = result.into_output_errors();
		assert!(errors.is_empty());
		assert!(output.is_some());
	}
}
