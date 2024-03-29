use doomfront::{
	rowan::ast::AstNode,
	testing::{assert_no_errors, prettyprint_maybe},
};

use crate::{ast, LexContext, ParseTree};

/// Yes, seriously.
#[test]
fn empty() {
	let ptree: ParseTree = doomfront::parse("", super::file, LexContext::default());
	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}
}

#[test]
fn name_smoke() {
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
fn arglist_smoke() {
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

#[test]
#[ignore]
fn with_sample_data() {
	let (_, sample) = match doomfront::testing::read_sample_data("LITHICA_PARSE_SAMPLE") {
		Ok(s) => s,
		Err(err) => {
			eprintln!("Skipping sample data-based unit test. Reason: {err}");
			return;
		}
	};

	let ptree: ParseTree = doomfront::parse(&sample, super::file, LexContext::default());
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn expr_lit_float_smoke() {
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
fn expr_lit_decimal_smoke() {
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
fn expr_lit_suffixed_string_smoke() {
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
fn expr_bin_userop_smoke() {
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

#[test]
fn expr_aggregate_smoke() {
	const SAMPLES: &[&str] = &[
		".{}",
		".{ }",
		".{ [0] = lorem }",
		".{ [0] = lorem, }",
		".{ [0] = lorem, .ipsum = dolor, sit_amet }",
		".{ [0] = lorem, .ipsum = dolor, sit_amet, }",
	];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample.trim(),
			|p| {
				super::expr(p, true);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let ast::Expr::Aggregate(_) = ast::Expr::cast(ptree.cursor()).unwrap() else {
			panic!()
		};
	}
}

#[test]
fn expr_block_smoke() {
	const SAMPLE: &str = "{ let i = 0; break i; }";

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

	let _ = ast::Expr::cast(ptree.cursor()).unwrap();
	let _ = ast::PrimaryExpr::cast(ptree.cursor()).unwrap();
}

#[test]
fn expr_block_as_primary() {
	const SAMPLE: &str = "{ break i; }.abs()";

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

	let _ = ast::Expr::cast(ptree.cursor()).unwrap();
}

#[test]
fn expr_construct_smoke() {
	const SAMPLES: &[&str] = &[
		"Black.Quartz {}",
		"Black.Quartz { }",
		"Black.Quartz { [0] = lorem }",
		"Black.Quartz { [0] = lorem, }",
		"Black.Quartz { [0] = lorem, .ipsum = dolor, sit_amet }",
		"Black.Quartz { [0] = lorem, .ipsum = dolor, sit_amet, }",
	];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample.trim(),
			|p| {
				super::expr(p, true);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let ast::Expr::Construct(_) = ast::Expr::cast(ptree.cursor()).unwrap() else {
			panic!()
		};
	}
}

#[test]
fn expr_struct_smoke() {
	const SAMPLE: &str = r#"struct {
		const LOREM_IPSUM: dolor = sit_amet;

		lorem: ipsum;

		function lorem_ipsum();
	}"#;

	let ptree: ParseTree = doomfront::parse(
		SAMPLE.trim(),
		|p| {
			super::expr(p, true);
		},
		LexContext::default(),
	);

	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}

	let ast::Expr::Struct(e_struct) = ast::Expr::cast(ptree.cursor()).unwrap() else {
		panic!()
	};

	let mut innards = e_struct.innards();

	let ast::StructInnard::Item(_) = innards.next().unwrap() else {
		panic!()
	};

	let ast::StructInnard::Field(_) = innards.next().unwrap() else {
		panic!()
	};

	let ast::StructInnard::Item(_) = innards.next().unwrap() else {
		panic!()
	};
}

// Patterns ////////////////////////////////////////////////////////////////////

#[test]
fn pat_smoke() {
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
fn pat_slice_smoke() {
	const SAMPLES: &[&str] = &["[]", "[ ]", "[ lorem, -2,_, ipsum ]"];

	for sample in SAMPLES {
		let ptree: ParseTree =
			doomfront::parse(sample.trim(), super::pattern, LexContext::default());

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}
	}
}

// Statements //////////////////////////////////////////////////////////////////

#[test]
fn stmt_continue_smoke() {
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
fn stmt_bind_smoke() {
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
fn stmt_break_smoke() {
	const SAMPLES: &[&str] = &[
		"break ;",
		"break ::lorem::;",
		"break ::lorem:: ipsum ;",
		"break ipsum ;",
	];

	const TESTS: &[fn(ast::StmtBreak)] = &[
		|ast| {
			assert!(ast.block_label().is_none());
			assert!(ast.expr().is_none());
		},
		|ast| {
			let label = ast.block_label().unwrap();
			assert_eq!(label.ident().unwrap().text(), "lorem");
			assert!(ast.expr().is_none());
		},
		|ast| {
			let label = ast.block_label().unwrap();
			assert_eq!(label.ident().unwrap().text(), "lorem");
			let expr = ast.expr().unwrap();
			assert_eq!(format!("{}", expr.syntax().text()), "ipsum");
		},
		|ast| {
			assert!(ast.block_label().is_none());
			let expr = ast.expr().unwrap();
			assert_eq!(format!("{}", expr.syntax().text()), "ipsum");
		},
	];

	for (i, sample) in SAMPLES.iter().copied().enumerate() {
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

		TESTS[i](ast::StmtBreak::cast(ptree.cursor()).unwrap());
	}
}

#[test]
fn stmt_expr_smoke() {
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

#[test]
fn stmt_return_smoke() {
	const SAMPLES: &[&str] = &["return;", "return ;", "return 0;", "return 0 ;"];

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
fn func_decl_smoke() {
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
fn sym_const_smoke() {
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
