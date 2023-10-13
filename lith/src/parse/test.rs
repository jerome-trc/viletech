use doomfront::{
	rowan::ast::AstNode,
	testing::{assert_no_errors, prettyprint_maybe},
};

use crate::{ast, ParseTree};

/// Yes, seriously.
#[test]
fn smoke_nothing() {
	let ptree: ParseTree = doomfront::parse("", super::file, ());
	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}
}

#[test]
fn smoke_name() {
	const SAMPLES: &[&str] = &["'lorem_ipsum'", "lorem_ipsum"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				super::expr(p);
			},
			(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let token0 = ptree.cursor().first_token().unwrap();
		let name = ast::Name(token0);
		assert_eq!(name.text(), "lorem_ipsum");
	}
}

#[test]
fn smoke_imports() {
	const SAMPLES: &[&str] = &[
		"import \"/lorem/ipsum\" : * => dolor;",
		"import \"lorem/ipsum\" : dolor;",
		"import \"./lorem/ipsum\" : dolor => sit_amet;",
		"import \"lorem/../ipsum\" : 'dolor' => sit_amet, consectetur => adipiscing;",
	];

	const TESTS: &[fn(ast::Import)] = &[
		|ast| {
			assert_eq!(ast.path().unwrap().string().unwrap(), "/lorem/ipsum");

			let ast::Import::All { inner, .. } = ast else {
				panic!()
			};

			assert_eq!(inner.rename().unwrap().text(), "dolor");
		},
		|ast| {
			assert_eq!(ast.path().unwrap().string().unwrap(), "lorem/ipsum");

			let ast::Import::List { list, .. } = ast else {
				panic!()
			};

			let mut entries = list.entries();

			{
				let entry = entries.next().unwrap();
				assert_eq!(entry.name().unwrap().text(), "dolor");
				assert!(entry.rename().is_none());
			}
		},
		|ast| {
			assert_eq!(ast.path().unwrap().string().unwrap(), "./lorem/ipsum");

			let ast::Import::List { list, .. } = ast else {
				panic!()
			};

			let mut entries = list.entries();

			{
				let entry = entries.next().unwrap();
				assert_eq!(entry.name().unwrap().text(), "dolor");
				assert_eq!(entry.rename().unwrap().text(), "sit_amet");
			}
		},
		|ast| {
			assert_eq!(ast.path().unwrap().string().unwrap(), "lorem/../ipsum");

			let ast::Import::List { list, .. } = ast else {
				panic!()
			};

			let mut entries = list.entries();

			{
				let entry = entries.next().unwrap();
				assert_eq!(entry.name().unwrap().text(), "dolor");
				assert_eq!(entry.rename().unwrap().text(), "sit_amet");
			}

			{
				let entry = entries.next().unwrap();
				assert_eq!(entry.name().unwrap().text(), "consectetur");
				assert_eq!(entry.rename().unwrap().text(), "adipiscing");
			}
		},
	];

	for (i, sample) in SAMPLES.iter().copied().enumerate() {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				let mark = p.open();
				super::import(p, mark);
			},
			(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		TESTS[i](ast::Import::cast(ptree.cursor()).unwrap());
	}
}

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn smoke_literal_float() {
	const SAMPLES: &[&str] = &["0.", "0.1", "0_.", "0_.1", "0_.1_", "0_.1_f32"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				super::expr(p);
			},
			(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let ast = ast::ExprLit::cast(ptree.cursor()).unwrap();
		let lit = ast.token();

		match lit.float() {
			Some(Ok(_)) => {}
			Some(Err(err)) => {
				panic!("failed to parse float literal sample `{sample}`: {err}");
			}
			_ => panic!("failed to lex float literal"),
		};
	}
}

#[test]
fn smoke_literal_decimal() {
	const SAMPLES: &[&str] = &["0", "0_", "0_u8", "0_i128"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				super::expr(p);
			},
			(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let ast = ast::ExprLit::cast(ptree.cursor()).unwrap();
		let lit = ast.token();

		match lit.int() {
			Some(Ok(_)) => {}
			Some(Err(err)) => {
				panic!("failed to parse decimal literal sample `{sample}`: {err}");
			}
			_ => panic!("failed to lex decimal literal"),
		};
	}
}

#[test]
fn smoke_expr_lit_suffixed_string() {
	const SAMPLE: &str = "\"lorem ipsum\"_dolor_sit_amet";

	let ptree: ParseTree = doomfront::parse(
		SAMPLE,
		|p| {
			super::expr(p);
		},
		(),
	);

	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}

	let ast = ast::ExprLit::cast(ptree.cursor()).unwrap();
	assert_eq!(ast.token().string().unwrap(), "lorem ipsum");
	assert_eq!(ast.string_suffix().unwrap().text(), "_dolor_sit_amet");
}

#[test]
fn smoke_expr_bin_userop() {
	const SAMPLE: &str = "a @dot b";

	let ptree: ParseTree = doomfront::parse(
		SAMPLE,
		|p| {
			super::expr(p);
		},
		(),
	);

	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}

	let ast = ast::ExprBin::cast(ptree.cursor()).unwrap();

	let ast::BinOp::User { ident, .. } = ast.operator().unwrap() else {
		panic!()
	};

	assert_eq!(ident.text(), "dot");
}

// Items ///////////////////////////////////////////////////////////////////////

#[test]
fn smoke_func_decl() {
	const SAMPLES: &[&str] = &[
		// Nothing extraneous.
		r#"function lorem_ipsum();"#,
		// Only a return type.
		r#"function lorem_ipsum(): dolor;"#,
		// One parameter.
		r#"function lorem_ipsum(dolor: sit_amet);"#,
		// One const parameter with a default.
		r#"function lorem_ipsum(const dolor: sit = amet);"#,
	];

	const TESTS: &[fn(ast::FunctionDecl)] = &[
		|ast| {
			assert_eq!(ast.name().unwrap().text(), "lorem_ipsum");
			assert!(ast.return_type().is_none());
		},
		|ast| {
			let ret_t = ast.return_type().unwrap();
			let ret_t_expr = ret_t.expr().unwrap();
			let ast::Expr::Ident(e) = ret_t_expr else {
				panic!();
			};
			assert_eq!(e.token().text(), "dolor");
		},
		|ast| {
			let param_list = ast.params().unwrap();
			let mut params = param_list.iter();
			let param = params.next().unwrap();
			assert_eq!(param.name().unwrap().text(), "dolor");

			assert!(matches!(
				param.type_spec().unwrap().expr().unwrap(),
				ast::Expr::Ident(_)
			));
		},
		|ast| {
			let param_list = ast.params().unwrap();
			let mut params = param_list.iter();
			let param = params.next().unwrap();
			assert_eq!(param.name().unwrap().text(), "dolor");
			assert!(param.is_const());
			assert!(matches!(param.default().unwrap(), ast::Expr::Ident(_)));
		},
	];

	for (i, sample) in SAMPLES.iter().copied().enumerate() {
		let ptree = doomfront::parse(sample, super::core_element::<true>, ());

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!()
		}

		TESTS[i](ast::FunctionDecl::cast(ptree.cursor()).unwrap());
	}
}

#[test]
fn smoke_sym_const() {
	const SAMPLES: &[&str] = &[r#"const LOREM_IPSUM: dolor = sit_amet;"#];

	const TESTS: &[fn(ast::SymConst)] = &[|ast| {
		let name = ast.name().unwrap();
		let tspec = ast.type_spec().unwrap();
		let expr = ast.expr().unwrap();

		let ast::Expr::Ident(t) = tspec.expr().unwrap() else {
			panic!()
		};

		let ast::Expr::Ident(e) = expr else { panic!() };

		debug_assert_eq!(name.text(), "LOREM_IPSUM");
		debug_assert_eq!(t.token().text(), "dolor");
		debug_assert_eq!(e.token().text(), "sit_amet");
	}];

	for (i, sample) in SAMPLES.iter().copied().enumerate() {
		let ptree = doomfront::parse(
			sample,
			|p| {
				let mark = p.open();
				super::symbolic_constant(p, mark);
			},
			(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!()
		}

		TESTS[i](ast::SymConst::cast(ptree.cursor()).unwrap());
	}
}
