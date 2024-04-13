use std::borrow::Cow;

use rowan::ast::AstNode;

use super::*;

use crate::{
	testing::*,
	zdoom::{
		self,
		zscript::{ast, IncludeTree, ParseTree},
	},
};

/// Yes, seriously.
#[test]
fn empty() {
	let ptree: ParseTree = crate::parse("", file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
}

#[test]
fn stray_glyph() {
	const SAMPLE: &str = r#"/

/// A mixin class that does something.
mixin class df_Pickup {
Default {
	+FLAGSET
}
}
"#;

	let _ = crate::parse(SAMPLE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
}

#[test]
#[ignore]
fn with_sample_data() {
	let (_, sample) = match read_sample_data("DOOMFRONT_ZSCRIPT_SAMPLE") {
		Ok(s) => s,
		Err(err) => {
			eprintln!("Skipping ZScript sample data-based unit test. Reason: {err}");
			return;
		}
	};

	let ptree: ParseTree = crate::parse(&sample, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
#[ignore]
fn with_sample_dir() {
	let dir = match check_sample_dir("DOOMFRONT_ZSCRIPT_SAMPLE_DIR") {
		Ok(p) => p,
		Err(err) => {
			eprintln!("Skipping ZScript sample data-based unit test. Reason: {err}");
			return;
		}
	};

	let walker = walkdir::WalkDir::new(&dir)
		.follow_links(false)
		.max_depth(8)
		.same_file_system(true)
		.into_iter()
		.filter_map(|res| res.ok());

	for (i, dir_entry) in walker.enumerate() {
		if dir_entry.file_type().is_dir() {
			continue;
		}

		let bytes = match std::fs::read(dir_entry.path()) {
			Ok(b) => b,
			Err(err) => {
				eprintln!("Skipping `{}` ({err})", dir_entry.path().display());
				continue;
			}
		};

		let sample = String::from_utf8_lossy(&bytes).to_string();
		eprintln!("Parsing #{i}: `{}`...", dir_entry.path().display());
		let ptree: ParseTree = crate::parse(&sample, file, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}

#[test]
#[ignore]
fn inctree() {
	let (root_path, _) = match read_sample_data("DOOMFRONT_ZSCRIPT_SAMPLE_INCTREE") {
		Ok(s) => s,
		Err(err) => {
			eprintln!("Skipping ZScript include tree unit test. Reason: {err}");
			return;
		}
	};

	let Some(root_parent_path) = root_path.parent() else {
		eprintln!(
			"Skipping ZScript include tree unit test. Reason: `{}` has no parent.",
			root_path.display()
		);
		return;
	};

	let inctree = IncludeTree::new(
		&root_path,
		|path| {
			let p = root_parent_path.join(path);

			if !p.exists() {
				return None;
			}

			let bytes = std::fs::read(p)
				.map_err(|err| panic!("file I/O failure: {err}"))
				.unwrap();
			let source = String::from_utf8_lossy(&bytes);
			Some(Cow::Owned(source.as_ref().to_owned()))
		},
		file,
		zdoom::lex::Context::ZSCRIPT_LATEST,
		Syntax::IncludeDirective,
		Syntax::StringLit,
	);

	assert!(inctree.missing.is_empty());

	for ptree in inctree.files {
		eprintln!("Checking `{}`...", ptree.path().display());
		assert_no_errors(&ptree);
	}
}

// Common //////////////////////////////////////////////////////////////////////

#[test]
fn identlist_smoke() {
	const SAMPLE: &str = r#"property temple: of, the, ancient, techlords;"#;

	let ptree: ParseTree = crate::parse(SAMPLE, property_def, zdoom::lex::Context::ZSCRIPT_LATEST);

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn types_smoke() {
	const SAMPLES: &[&str] = &[
		"TeenyLittleBase",
		"Dead.On.Arrival",
		"readonly<Corruption2Factory>",
		"class",
		"class<Forge>",
		"array<Unwelcome>",
		"array<class<TheOssuary> >",
		"map<Corruption[1], Mortem[2]>",
		"mapiterator<FishInABarrel, Neoplasm>",
		"@LoremIpsum",
		"readonly<@LoremIpsum>",
		"@ LoremIpsum",
		"readonly < @ LoremIpsum >",
	];

	for sample in SAMPLES {
		let ptree: ParseTree = crate::parse(sample, type_ref, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}

#[test]
fn version_qual_smoke() {
	const SAMPLE: &str = r#"version("3.7.1")"#;

	let ptree: ParseTree = crate::parse(SAMPLE, version_qual, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	let qual = ast::VersionQual::cast(ptree.cursor()).unwrap();
	assert_eq!(qual.string().unwrap().string().unwrap(), "3.7.1");
}

#[test]
fn deprecation_qual_smoke() {
	const SAMPLE: &str = r#"deprecated("2.4.0", "Don't use this please")"#;

	let ptree: ParseTree = crate::parse(
		SAMPLE,
		deprecation_qual,
		zdoom::lex::Context::ZSCRIPT_LATEST,
	);

	assert_no_errors(&ptree);
	let qual = ast::DeprecationQual::cast(ptree.cursor()).unwrap();
	assert_eq!(qual.version().unwrap().string().unwrap(), "2.4.0");
	assert_eq!(qual.message().unwrap().text(), "\"Don't use this please\"");
}

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn expr_smoke() {
	const SAMPLE: &str = r#"!multiplayer && (GetPlayerInput(INPUT_BUTTONS))"#;

	let ptree: ParseTree = crate::parse(SAMPLE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn expr_complex() {
	const SAMPLE: &str = "(a[1]() + --b.c) * ++d && (e << f) ~== ((((g /= h ? i : j))))";

	let ptree: ParseTree = crate::parse(SAMPLE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn sizeof_alignof() {
	const SAMPLES: &[&str] = &[
		r#"sizeof a"#,
		r#"sizeof(a)"#,
		r#"alignof 0"#,
		r#"alignof (0)"#,
		r#"sizeof x + alignof y"#,
	];

	for sample in SAMPLES {
		let ptree: ParseTree = crate::parse(sample, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
		eprintln!();
	}
}

#[test]
fn expr_vector_bin() {
	const SAMPLE: &str = "(1.0, 2.0, 3.0) + (4.0, 5.0) - (6.0, 7.0, 8.0)";

	let ptree: ParseTree = crate::parse(SAMPLE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn expr_lit_stringconcat() {
	const SAMPLE: &str = r#"n + "interstellar" "domine""nuclear waste processing facility""#;

	let ptree: ParseTree = crate::parse(SAMPLE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn expr_unary_with_wsp() {
	const SAMPLE: &str = r#"lastenemy && ! lastenemy.tracer"#;

	let ptree: ParseTree = crate::parse(SAMPLE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

// Statements //////////////////////////////////////////////////////////////////

#[test]
fn stat_assign_smoke() {
	const SAMPLE: &str = r#"[x, y, z] = w;"#;
	let ptree: ParseTree = crate::parse(SAMPLE, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
	let stat = ast::AssignStat::cast(ptree.cursor()).unwrap();
	let ast::Expr::Ident(e_id) = stat.assignee().unwrap() else {
		panic!()
	};
	assert_eq!(e_id.token().text(), "w");
	let mut assigned = stat.assigned();
	let ast::Expr::Ident(e_id) = assigned.next().unwrap() else {
		panic!()
	};
	assert_eq!(e_id.token().text(), "x");
	let ast::Expr::Ident(e_id) = assigned.next().unwrap() else {
		panic!()
	};
	assert_eq!(e_id.token().text(), "y");
	let ast::Expr::Ident(e_id) = assigned.next().unwrap() else {
		panic!()
	};
	assert_eq!(e_id.token().text(), "z");
}

#[test]
fn stat_for_smoke() {
	const SAMPLES: &[&str] = &[
		r#"for (;;) {}"#,
		r#"for (int i = 0;;) {}"#,
		r#"for (;i < arr.len();) {}"#,
		r#"for (;;++i) {}"#,
		r#"for ( int i = 0 ; i < arr.len() ; ++i) {}"#,
	];

	for sample in SAMPLES {
		let ptree: ParseTree = crate::parse(sample, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}

#[test]
fn stat_if_smoke() {
	const SAMPLE: &str = r"if(player_data ) {
		uint press =	  GetPlayerInput(INPUT_BUTTONS) &
						(~GetPlayerInput(INPUT_OLDBUTTONS));

				if(press & BT_USER1)	player_data.Binds.Use(0);
		else	if(press & BT_USER2)	player_data.Binds.Use(1);
		else	if(press & BT_USER3)	player_data.Binds.Use(2);
		else	if(press & BT_USER4)	player_data.Binds.Use(3);
	}";

	let ptree: ParseTree = crate::parse(SAMPLE, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn stat_local_dynarray_smoke() {
	const SAMPLE: &str = "Array<Demoniacal> Overrun;";

	let ptree: ParseTree = crate::parse(SAMPLE, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn stat_local_dynarray_of_classes() {
	const SAMPLE: &str = r#"{ Array<Class<Vampire> > Castle; }"#;

	let ptree: ParseTree = crate::parse(SAMPLE, compound_stat, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn stat_static_const_smoke() {
	const SAMPLE: &str = "static const float[] SOME_FLOATS = {
		-0.05, -0.2, -0.4, 0.3, 0.15, 0.1, 0.07, 0.03
	};";

	let ptree: ParseTree = crate::parse(
		SAMPLE,
		static_const_stat,
		zdoom::lex::Context::ZSCRIPT_LATEST,
	);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let ast = ast::StaticConstStat::cast(ptree.cursor()).unwrap();
	let name_tok = ast.name().unwrap();
	assert_eq!(name_tok.text(), "SOME_FLOATS");
}

// Non-structural top-level ////////////////////////////////////////////////////

#[test]
fn constdef_smoke() {
	const SAMPLE: &str = r#"const GOLDEN_ANARCHY = BUSHFIRE >>> NONSPECIFIC_TECH_BASE;"#;

	let ptree: ParseTree = crate::parse(SAMPLE, const_def, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
}

#[test]
fn enumdef_smoke() {
	const SAMPLE: &str = r#"
enum SepticTank {};

enum BeyondTimesGate {
	/// Lorem ipsum dolor sit amet
	ELEMENTAL,
}

enum BrickAndRoot {
CELL_BLOCK_HELL,
FORGOTTEN_DATA_PROCESSING_CENTER = 1,
UAC_WELCOME = (9 << 9),
COLOURS_OF_DOOM = "Ascent",
}
"#;

	let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let mut ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	{
		let ast::TopLevel::EnumDef(_) = ast.next().unwrap() else {
			panic!()
		};
	}

	{
		let ast::TopLevel::EnumDef(enumdef) = ast.next().unwrap() else {
			panic!()
		};

		let mut variants = enumdef.variants();

		let var0 = variants.next().unwrap();
		let doc = var0.docs().next().unwrap();

		assert_eq!(doc.text_trimmed(), "Lorem ipsum dolor sit amet");
	}
}

#[test]
fn enum_error_recovery() {
	const SAMPLE: &str = r"
enum MyEnum {
ENUMVAL_0,
ENUMVAL_1,
ENUMVAL_2,
ENUM
}
";

	let ptree = crate::parse(SAMPLE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn directives_smoke() {
	const SAMPLE: &str = r##"

version "3.7.1"
#include "/summoning/hazard.zs"
#include
"the/pain/maze.zsc"

"##;

	let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let mut tops = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	{
		let ast::TopLevel::Version(vers_directive) = tops.next().unwrap() else {
			panic!()
		};

		assert_eq!(vers_directive.string().unwrap().string().unwrap(), "3.7.1");

		assert_eq!(
			vers_directive.version().unwrap(),
			zdoom::Version {
				major: 3,
				minor: 7,
				rev: 1
			}
		);
	}
}

// Class/structure/etc. ////////////////////////////////////////////////////////

#[test]
fn classdef_smoke() {
	const SAMPLE: &str = r#####"

class Rocketpack_Flare : Actor
{
Default
{
	RenderStyle "Add";
	Scale 0.25;
	Alpha 0.95;
	+NOGRAVITY
	+NOINTERACTION
	+THRUGHOST
	+DONTSPLASH
	+NOTIMEFREEZE
}

States
{
	Spawn:
		FLER A 1 Bright NoDelay {
			A_FadeOut(0.3);
			A_SetScale(Scale.X - FRandom(0.005, 0.0075));
			Return A_JumpIf(Scale.X <= 0.0, "Null");
		}
		Loop;
}
}

"#####;

	let ptree: ParseTree = crate::parse(
		SAMPLE.trim(),
		|p| {
			let _ = class_def(p);
		},
		zdoom::lex::Context::ZSCRIPT_LATEST,
	);

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn class_file_smoke() {
	const SAMPLE: &str = r"
// ???

class Lorem : Ipsum abstract;

int dolor;

void sitAmet() {}

class Consectetur; // This is a field, not a class head.

// !!!
";

	let ptree: ParseTree = crate::parse(
		SAMPLE.trim(),
		|p| {
			let _ = file(p);
		},
		zdoom::lex::Context::ZSCRIPT_LATEST,
	);

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let ast::TopLevel::ClassDef(_) =
		ast::TopLevel::cast(ptree.cursor().first_child().unwrap()).unwrap()
	else {
		panic!()
	};
}

#[test]
fn class_named_void() {
	const SAMPLE: &str = "class void {}";
	let ptree: ParseTree = crate::parse(
		SAMPLE,
		|p| {
			let _ = class_def(p);
		},
		zdoom::lex::Context::ZSCRIPT_LATEST,
	);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn class_error_recovery() {
	const SAMPLE: &str = r#####"class df_SomeClass : Actor abstract
{
protected action void A_DF_Action()
}"#####;

	let ptree = crate::parse(
		SAMPLE,
		|p| {
			let _ = class_def(p);
		},
		zdoom::lex::Context::ZSCRIPT_LATEST,
	);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn after_class_error_recovery() {
	const SAMPLE: &str = r#####"
class df_SomeClass : Actor abstract
{
protected action void A_DF_Action();
}gl

class df_AnotherClass : Actor
{
Default
{

}
}
"#####;

	let ptree = crate::parse(SAMPLE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn mixin_class_error_recovery() {
	const SAMPLE: &str = r#"/// A mixin class that does something.
mixin class df_Pickup {
Default {
	+FLAGSET
}

meta Actor a
}"#;

	let ptree = crate::parse(SAMPLE, mixin_class_def, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn doc_comment_smoke() {
	const SAMPLE: &str = r#"

/// UAC Mines
/// Sector 14-3
class DevastationFixed {}

"#;

	let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	let class = ast::ClassDef::cast(ptree.cursor().first_child().unwrap()).unwrap();
	let head = class.head();
	assert_eq!(head.name().unwrap().text(), "DevastationFixed");
	let mut docs = class.docs();
	assert_eq!(docs.next().unwrap().text_trimmed(), "UAC Mines");
	assert_eq!(docs.next().unwrap().text_trimmed(), "Sector 14-3");
	assert!(docs.next().is_none());
}

#[test]
fn field_smoke() {
	const SAMPLE: &str = r#"int[1][] corruption, three[], nexus[][1];"#;

	let ptree: ParseTree = crate::parse(SAMPLE, member_decl, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let field = ast::FieldDecl::cast(ptree.cursor()).unwrap();
	let type_spec = field.type_spec().unwrap();

	let ast::CoreType::Primitive(_) = type_spec.core() else {
		panic!("Expected primitive type specifier `int`.");
	};

	{
		let mut arr_lens = type_spec.array_lengths();
		let len0 = arr_lens.next().unwrap();
		assert!(len0.expr().is_some());
		let len1 = arr_lens.next().unwrap();
		assert!(len1.expr().is_none());
	}

	let mut names = field.names();

	{
		let name0 = names.next().unwrap();
		assert_eq!(name0.ident().text(), "corruption");
	}

	{
		let name1 = names.next().unwrap();
		assert_eq!(name1.ident().text(), "three");
		let mut lengths = name1.array_lengths();
		assert!(lengths.next().unwrap().expr().is_none());
	}

	{
		let name2 = names.next().unwrap();
		assert_eq!(name2.ident().text(), "nexus");
		let mut lengths = name2.array_lengths();
		assert!(lengths.next().unwrap().expr().is_none());
		assert!(lengths.next().unwrap().expr().is_some());
	}
}

#[test]
fn method_smoke() {
	const SAMPLE: &str = r#"int, int uac_genesis() const;"#;

	let ptree: ParseTree = crate::parse(SAMPLE, member_decl, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let fndecl = ast::FunctionDecl::cast(ptree.cursor()).unwrap();
	assert!(fndecl.is_const());
}

#[test]
fn varargs_smoke() {
	const SAMPLE: &str = r#"void bashibozuk(int a, float b, ...) const {}"#;

	let ptree: ParseTree = crate::parse(SAMPLE, member_decl, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let fndecl = ast::FunctionDecl::cast(ptree.cursor()).unwrap();
	let params = fndecl.param_list().unwrap();
	assert!(params.varargs());
}

// Actor ///////////////////////////////////////////////////////////////////////

#[test]
fn states_block_smoke() {
	const SAMPLE: &str = "States { Spawn: XZW1 A 33; XZW1 B 2; Loop; }";

	let ptree: ParseTree = crate::parse(SAMPLE, states_block, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn goto_smoke() {
	const SAMPLE: &str = r#####"States {
goto Super::LoremIpsum + 12345;
goto Dolor::SitAmet;
goto Consectetur;
}"#####;

	let ptree: ParseTree = crate::parse(SAMPLE, states_block, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let ast = ast::StatesBlock::cast(ptree.cursor()).unwrap();
	let mut innards = ast.innards();

	{
		let innard = innards.next().unwrap();

		let ast::StatesInnard::Flow(flow) = innard else {
			panic!()
		};

		let ast::StateFlowKind::Goto {
			scope,
			name,
			offset,
		} = flow.kind()
		else {
			panic!()
		};

		assert_eq!(scope.unwrap().kind(), Syntax::KwSuper);
		assert_eq!(format!("{}", name.syntax().text()), "LoremIpsum");
		assert_eq!(offset.unwrap().int().unwrap().unwrap().0, 12345);
	}

	{
		let innard = innards.next().unwrap();

		let ast::StatesInnard::Flow(flow) = innard else {
			panic!()
		};

		let ast::StateFlowKind::Goto { scope, .. } = flow.kind() else {
			panic!()
		};

		let s = scope.unwrap();
		assert_eq!(s.kind(), Syntax::Ident);
		assert_eq!(s.text(), "Dolor");
	}

	{
		let innard = innards.next().unwrap();

		let ast::StatesInnard::Flow(flow) = innard else {
			panic!()
		};

		let ast::StateFlowKind::Goto { name, .. } = flow.kind() else {
			panic!()
		};

		assert_eq!(format!("{}", name.syntax().text()), "Consectetur");
	}
}
