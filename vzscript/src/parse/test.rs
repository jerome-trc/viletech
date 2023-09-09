use doomfront::{rowan::ast::AstNode, testing::prettyprint_maybe};

use crate::{ast, ParseTree, SyntaxNode};

use super::{common::*, expr::*, file, item::*, stat::*};

fn assert_no_errors(ptree: &ParseTree, case: usize) {
	assert!(
		!ptree.any_errors(),
		"test case {case} encountered errors: {}\r\n",
		{
			let mut output = String::new();

			for err in ptree.errors() {
				output.push_str(&format!("\r\n{err:#?}"));
			}

			output
		}
	);
}

/// Yes, seriously.
#[test]
fn smoke_nothing() {
	let ptree: ParseTree = doomfront::parse("", file, crate::Version::new(0, 0, 0));
	assert_no_errors(&ptree, 0);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!()
	}
}

#[test]
fn smoke_attribute() {
	const SOURCES: &[&str] = &[
		"#[the_thaumaturge]",
		"#[ untilted]",
		"#[the_cry ]",
		"#[ coal ]",
	];

	for (_, source) in SOURCES.iter().copied().enumerate() {
		let ptree: ParseTree =
			doomfront::parse(source, Attribute::parse, crate::Version::new(0, 0, 0));

		assert_no_errors(&ptree, 0);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!()
		}
	}
}

#[test]
fn smoke_annotation() {
	const SOURCE: &str = r##"#include("/liquid/luck.vzs")"##;

	let ptree: ParseTree =
		doomfront::parse(SOURCE, Annotation::parse, crate::Version::new(0, 0, 0));

	assert_no_errors(&ptree, 0);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!()
	}
}

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn smoke_expr_field() {
	const SOURCE: &str = r#"mist.at.dawn"#;

	let ptree: ParseTree = doomfront::parse(
		SOURCE,
		|p| {
			let _ = Expression::parse(p);
		},
		crate::Version::new(0, 0, 0),
	);

	assert_no_errors(&ptree, 0);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!()
	}
}

#[test]
fn smoke_expr_for() {
	const SOURCE: &str = r#"for conductor : faultline {}"#;

	let ptree: ParseTree = doomfront::parse(
		SOURCE,
		|p| {
			let _ = Expression::parse(p);
		},
		crate::Version::new(0, 0, 0),
	);

	assert_no_errors(&ptree, 0);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!()
	}
}

// Statements //////////////////////////////////////////////////////////////////

#[test]
fn smoke_stat_bind() {
	const SOURCES: &[&str] = &[
		"let petrichor;",
		"readonly twilit: jungle;",
		"#[allow(unused)] let vinefort = encased;",
		"readonly enigma: stormwater = 'the night guard';",
	];

	for (i, source) in SOURCES.iter().copied().enumerate() {
		let ptree = doomfront::parse(
			source,
			super::core_element::<false>,
			crate::Version::new(0, 0, 0),
		);
		assert_no_errors(&ptree, 0);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!()
		}
	}
}

// Non-structural items ////////////////////////////////////////////////////////

#[test]
fn smoke_func_decl() {
	const SOURCES: &[&str] = &[
		// Nothing extraneous.
		r#"function escape_velocity();"#,
		// Only a return type.
		r#"function atmospheric_pressure(): void;"#,
		// Everything.
		r#"/// lorem ipsum
		/// dolor sit amet
		#[within_reach(yaw)]
		function heliotrope(atomic: 'quicksilver' = 9_0_9): weather_warning { }"#,
	];

	const TESTS: &[fn(SyntaxNode)] = &[
		|node| {
			let fndecl = ast::FuncDecl::cast(node).unwrap();
			assert_eq!(fndecl.name().unwrap().text(), "escape_velocity");
			assert!(fndecl.return_type().is_none());
		},
		|node| {
			let fndecl = ast::FuncDecl::cast(node).unwrap();
			assert_eq!(fndecl.name().unwrap().text(), "atmospheric_pressure");
			let ret_t = fndecl.return_type().unwrap();
			let ret_t_expr = ret_t.expr().unwrap();
			let ast::Expr::Ident(e) = ret_t_expr else { panic!(); };
			assert_eq!(e.token().text(), "void");
		},
		|node| {
			let fndecl = ast::FuncDecl::cast(node).unwrap();
			assert_eq!(fndecl.name().unwrap().text(), "heliotrope");
			let mut docs = fndecl.docs();
			assert_eq!(docs.next().unwrap().text_trimmed(), "lorem ipsum");
			assert_eq!(docs.next().unwrap().text_trimmed(), "dolor sit amet");
			let attr = fndecl.attributes().next().unwrap();
			assert_eq!(attr.name().unwrap().text(), "within_reach");
			let mut params = fndecl.params().unwrap().iter();
			let param0 = params.next().unwrap();
			assert_eq!(param0.name().unwrap().text(), "atomic");
			let param0_t = param0.type_spec().unwrap();
			let ast::Expr::Literal(literal) = param0_t.expr().unwrap() else {
				panic!()
			};
			let lit_tok = literal.token();
			assert_eq!(lit_tok.name().unwrap(), "quicksilver");
			let default = param0.default().unwrap();
			let ast::Expr::Literal(literal) = default else {
				panic!()
			};
			let lit_tok = literal.token();
			assert_eq!(lit_tok.int().unwrap(), Ok((9_0_9, ast::IntSuffix::None)));
		},
	];

	for (i, source) in SOURCES.iter().copied().enumerate() {
		let ptree = doomfront::parse(
			source,
			super::core_element::<true>,
			crate::Version::new(0, 0, 0),
		);
		assert_no_errors(&ptree, 0);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!()
		}

		TESTS[i](ptree.cursor());
	}
}

// Structural items ////////////////////////////////////////////////////////////
