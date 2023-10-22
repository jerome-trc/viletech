use doomfront::{
	rowan::ast::AstNode,
	testing::{assert_no_errors, prettyprint_maybe},
};

use crate::{ast, LexContext, ParseTree};

/// Yes, seriously.
#[test]
fn smoke_nothing() {
	let ptree: ParseTree = doomfront::parse("", super::file, LexContext::default());
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
				super::expr(p, true);
			},
			LexContext::default(),
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
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		TESTS[i](ast::Import::cast(ptree.cursor()).unwrap());
	}
}

#[test]
fn smoke_arglist() {
	const SAMPLES: &[&str] = &[
		"()",
		"( lorem)",
		"(lorem)",
		"(lorem )",
		"(lorem, ipsum)",
		"(lorem: ipsum)",
		"(lorem, ipsum: dolor)",
		"(...)",
		"(lorem, ...)",
		"(lorem, ipsum: dolor, ...)",
	];

	const TESTS: &[fn(ast::ArgList)] = &[
		|ast| {
			assert!(ast.iter().next().is_none());
			assert!(!ast.acceding());
		},
		|_| {},
		|_| {},
		|_| {},
		|_| {},
		|_| {},
		|ast| {
			let mut args = ast.iter();
			let _ = args.next().unwrap();
			let arg1 = args.next().unwrap();
			let arg1_name = arg1.name().unwrap();
			assert_eq!(arg1_name.text(), "ipsum");
		},
		|ast| {
			assert!(ast.iter().next().is_none());
			assert!(ast.acceding());
		},
		|ast| {
			assert!(ast.iter().next().is_some());
			assert!(ast.acceding());
		},
		|ast| {
			assert!(ast.iter().next().is_some());
			assert!(ast.acceding());
		},
	];

	for (i, sample) in SAMPLES.iter().copied().enumerate() {
		let ptree: ParseTree = doomfront::parse(sample, super::arg_list, LexContext::default());

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		TESTS[i](ast::ArgList::cast(ptree.cursor()).unwrap());
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
				super::expr(p, true);
			},
			LexContext::default(),
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
				super::expr(p, true);
			},
			LexContext::default(),
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
			super::expr(p, true);
		},
		LexContext::default(),
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
			super::expr(p, true);
		},
		LexContext::default(),
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

// Patterns ////////////////////////////////////////////////////////////////////

#[test]
fn smoke_pat_simple() {
	const SAMPLES: &[&str] = &[
		"_",
		"lorem_ipsum",
		"\"lorem ipsum\"",
		"'lorem ipsum'",
		"123",
		"0.",
		"-0_i128",
		"- 09876.54321",
		"(_)",
	];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(sample, super::pattern, LexContext::default());

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}
	}
}

#[test]
fn smoke_pat_slice() {
	const SAMPLES: &[&str] = &["[ lorem, -2,_, ipsum ]"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(sample, super::pattern, LexContext::default());

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}
	}
}

// Statements //////////////////////////////////////////////////////////////////

#[test]
fn smoke_stmt_continue() {
	const SAMPLES: &[&str] = &[
		"continue;",
		"continue ;",
		"continue::lorem::;",
		"continue ::ipsum::;",
		"continue::dolor:: ;",
		"continue ::sit_amet:: ;",
	];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				let mark = p.open();
				super::statement(p, mark);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}
	}
}

#[test]
fn smoke_stmt_bind() {
	const SAMPLES: &[&str] = &[
		"let lorem;",
		"var ipsum ;",
		"let const dolor;",
		"var const amet=\"\";",
		"let (consectetur) = 0;",
		"var const-1 = -1;",
		"let _= 'adipiscing';",
	];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				let mark = p.open();
				super::statement(p, mark);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}
	}
}

#[test]
fn smoke_stmt_expr() {
	const SAMPLES: &[&str] = &["lorem;", "ipsum();", "dolor = sit_amet;", "-1;", "(0);"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				let mark = p.open();
				super::statement(p, mark);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}
	}
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
		// With an annotation.
		r##"
		#[dolor.sit_amet()]
		function lorem_ipsum();"##,
		// Reference parameter.
		r#"function lorem_ipsum(& dolor: sit = amet);"#,
		// Mutable reference parameter.
		r#"function lorem_ipsum(& var dolor: sit = amet);"#,
		// Intrinsic.
		r#"function lorem_ipsum(...);"#,
	];

	const TESTS: &[fn(ast::FunctionDecl)] = &[
		|ast| {
			assert_eq!(ast.name().unwrap().text(), "lorem_ipsum");
			assert!(ast.return_type().is_none());
		},
		|ast| {
			let ret_t = ast.return_type().unwrap();
			let ret_t_expr = ret_t.into_expr().unwrap();
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
				param.type_spec().unwrap().into_expr().unwrap(),
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
		|ast| {
			let anno = ast.annotations().next().unwrap();

			let ast::AnnotationName::Scoped(id0, id1) = anno.name().unwrap() else {
				panic!()
			};

			assert_eq!(id0.text(), "dolor");
			assert_eq!(id1.text(), "sit_amet");
			assert!(anno.arg_list().unwrap().iter().next().is_none());
		},
		|ast| {
			let param_list = ast.params().unwrap();
			let mut params = param_list.iter();
			let param = params.next().unwrap();

			assert!(matches!(param.ref_spec(), ast::ParamRefSpec::Ref(_)),);
		},
		|ast| {
			let param_list = ast.params().unwrap();
			let mut params = param_list.iter();
			let param = params.next().unwrap();

			assert!(matches!(param.ref_spec(), ast::ParamRefSpec::RefVar(_, _)),);
		},
		|ast| {
			let param_list = ast.params().unwrap();
			assert!(param_list.intrinsic_params());
		},
	];

	for (i, sample) in SAMPLES.iter().copied().enumerate() {
		let ptree = doomfront::parse(
			sample.trim(),
			super::core_element::<true>,
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!()
		}

		TESTS[i](ast::FunctionDecl::cast(ptree.cursor()).unwrap());
	}
}

#[test]
fn smoke_sym_const() {
	const SAMPLES: &[&str] = &[
		r#"const LOREM_IPSUM: dolor = sit_amet;"#,
		r##"
		#[consectetur]
		const LOREM_IPSUM: dolor = sit + amet;"##,
	];

	const TESTS: &[fn(ast::SymConst)] = &[
		|ast| {
			let name = ast.name().unwrap();
			let tspec = ast.type_spec().unwrap();
			let expr = ast.expr().unwrap();

			let ast::Expr::Ident(t) = tspec.into_expr().unwrap() else {
				panic!()
			};

			let ast::Expr::Ident(e) = expr else { panic!() };

			debug_assert_eq!(name.text(), "LOREM_IPSUM");
			debug_assert_eq!(t.token().text(), "dolor");
			debug_assert_eq!(e.token().text(), "sit_amet");
		},
		|_| {},
	];

	for (i, sample) in SAMPLES.iter().copied().enumerate() {
		let ptree = doomfront::parse(
			sample.trim(),
			super::core_element::<true>,
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!()
		}

		TESTS[i](ast::SymConst::cast(ptree.cursor()).unwrap());
	}
}
