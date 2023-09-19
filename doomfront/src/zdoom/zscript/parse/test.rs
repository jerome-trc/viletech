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

#[test]
fn smoke_empty() {
	let ptree: ParseTree = crate::parse("", file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
}

#[test]
fn smoke_stray_glyph() {
	const SOURCE: &str = r#"/

/// A mixin class that does something.
mixin class df_Pickup {
Default {
	+FLAGSET
}
}
"#;

	let _ = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
}

#[test]
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
		Syn::IncludeDirective,
		Syn::StringLit,
	);

	assert!(inctree.missing.is_empty());

	for ptree in inctree.files {
		eprintln!("Checking `{}`...", ptree.path().display());
		assert_no_errors(&ptree);
	}
}

// Common //////////////////////////////////////////////////////////////////////

#[test]
fn smoke_identlist() {
	const SOURCE: &str = r#"property temple: of, the, ancient, techlords;"#;

	let ptree: ParseTree = crate::parse(SOURCE, property_def, zdoom::lex::Context::ZSCRIPT_LATEST);

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_types() {
	const SOURCES: &[&str] = &[
		"TeenyLittleBase",
		"Dead.On.Arrival",
		"readonly<Corruption2Factory>",
		"class",
		"class<Forge>",
		"array<Unwelcome>",
		"array<class<TheOssuary> >",
		"map<Corruption[1], Mortem[2]>",
		"mapiterator<FishInABarrel, Neoplasm>",
	];

	for source in SOURCES {
		let ptree: ParseTree = crate::parse(source, type_ref, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}

#[test]
fn smoke_version_qual() {
	const SOURCE: &str = r#"version("3.7.1")"#;

	let ptree: ParseTree = crate::parse(SOURCE, version_qual, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	let qual = ast::VersionQual::cast(ptree.cursor()).unwrap();
	assert_eq!(qual.version().unwrap().string().unwrap(), "3.7.1");
}

#[test]
fn smoke_deprecation_qual() {
	const SOURCE: &str = r#"deprecated("2.4.0", "Don't use this please")"#;

	let ptree: ParseTree = crate::parse(
		SOURCE,
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
fn smoke_1() {
	const SOURCE: &str = "(a[1]() + --b.c) * ++d && (e << f) ~== ((((g /= h ? i : j))))";

	let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_2() {
	const SOURCE: &str = r#"!multiplayer && (GetPlayerInput(INPUT_BUTTONS))"#;

	let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_vector_bin() {
	const SOURCE: &str = "(1.0, 2.0, 3.0) + (4.0, 5.0) - (6.0, 7.0, 8.0)";

	let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_string_lit_concat() {
	const SOURCE: &str = r#"n + "interstellar" "domine""nuclear waste processing facility""#;

	let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_unary_with_wsp() {
	const SOURCE: &str = r#"lastenemy && ! lastenemy.tracer"#;

	let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

// Statements //////////////////////////////////////////////////////////////////

#[test]
fn smoke_assign() {
	const SOURCE: &str = r#"[x, y, z] = w;"#;
	let ptree: ParseTree = crate::parse(SOURCE, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
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
fn smoke_for_loop() {
	const SOURCES: &[&str] = &[
		r#"for (;;) {}"#,
		r#"for (int i = 0;;) {}"#,
		r#"for (;i < arr.len();) {}"#,
		r#"for (;;++i) {}"#,
		r#"for ( int i = 0 ; i < arr.len() ; ++i) {}"#,
	];

	for source in SOURCES {
		let ptree: ParseTree = crate::parse(source, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}

#[test]
fn smoke_if() {
	const SOURCE: &str = r"if(player_data ) {
		uint press =	  GetPlayerInput(INPUT_BUTTONS) &
						(~GetPlayerInput(INPUT_OLDBUTTONS));

				if(press & BT_USER1)	player_data.Binds.Use(0);
		else	if(press & BT_USER2)	player_data.Binds.Use(1);
		else	if(press & BT_USER3)	player_data.Binds.Use(2);
		else	if(press & BT_USER4)	player_data.Binds.Use(3);
	}";

	let ptree: ParseTree = crate::parse(SOURCE, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_local_dynarray() {
	const SOURCE: &str = "Array<Demoniacal> Overrun;";

	let ptree: ParseTree = crate::parse(SOURCE, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_local_dynarray_of_classes() {
	const SOURCE: &str = r#"{ Array<Class<Vampire> > Castle; }"#;

	let ptree: ParseTree = crate::parse(SOURCE, compound_stat, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_static_const() {
	const SOURCE: &str = "static const float[] SOME_FLOATS = {
		-0.05, -0.2, -0.4, 0.3, 0.15, 0.1, 0.07, 0.03
	};";

	let ptree: ParseTree = crate::parse(
		SOURCE,
		static_const_stat,
		zdoom::lex::Context::ZSCRIPT_LATEST,
	);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

// Non-structural top-level ////////////////////////////////////////////////////

#[test]
fn smoke_constdef() {
	const SOURCE: &str = r#"const GOLDEN_ANARCHY = BUSHFIRE >>> NONSPECIFIC_TECH_BASE;"#;

	let ptree: ParseTree = crate::parse(SOURCE, const_def, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
}

#[test]
fn smoke_enumdef() {
	const SOURCE: &str = r#"
enum SepticTank {};

enum BeyondTimesGate {
ELEMENTAL,
}

enum BrickAndRoot {
CELL_BLOCK_HELL,
FORGOTTEN_DATA_PROCESSING_CENTER = 1,
UAC_WELCOME = (9 << 9),
COLOURS_OF_DOOM = "Ascent",
}
"#;

	let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
}

#[test]
fn enum_error_recovery() {
	const SOURCE: &str = r"
enum MyEnum {
ENUMVAL_0,
ENUMVAL_1,
ENUMVAL_2,
ENUM
}
";

	let ptree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_directives() {
	const SOURCE: &str = r##"

version "3.7.1"
#include "/summoning/hazard.zs"
#include
"the/pain/maze.zsc"

"##;

	let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
}

// Class/structure/etc. ////////////////////////////////////////////////////////

#[test]
fn smoke_class() {
	const SOURCE: &str = r#####"

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

	let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
}

#[test]
fn class_error_recovery() {
	const SOURCE: &str = r#####"class df_SomeClass : Actor abstract
{
protected action void A_DF_Action()
}"#####;

	let ptree = crate::parse(SOURCE, class_def, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn after_class_error_recovery() {
	const SOURCE: &str = r#####"
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

	let ptree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn mixin_class_error_recovery() {
	const SOURCE: &str = r#"/// A mixin class that does something.
mixin class df_Pickup {
Default {
	+FLAGSET
}

meta Actor a
}"#;

	let ptree = crate::parse(SOURCE, mixin_class_def, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert!(ptree.any_errors());
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_doc() {
	const SOURCE: &str = r#"

/// UAC Mines
/// Sector 14-3
class DevastationFixed {}

"#;

	let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	let class = ast::ClassDef::cast(ptree.cursor().first_child().unwrap()).unwrap();
	assert_eq!(class.name().unwrap().text(), "DevastationFixed");
	let mut docs = class.docs();
	assert_eq!(docs.next().unwrap().text_trimmed(), "UAC Mines");
	assert_eq!(docs.next().unwrap().text_trimmed(), "Sector 14-3");
	assert!(docs.next().is_none());
}

#[test]
fn smoke_field() {
	const SOURCE: &str = r#"int corruption, three[], nexus[][1];"#;

	let ptree: ParseTree = crate::parse(SOURCE, member_decl, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let field = ast::FieldDecl::cast(ptree.cursor()).unwrap();

	let ast::CoreType::Primitive(_) = field.type_spec().unwrap().core() else {
		panic!("Expected primitive type specifier `int`.");
	};

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
fn smoke_method() {
	const SOURCE: &str = r#"int, int uac_genesis() const;"#;

	let ptree: ParseTree = crate::parse(SOURCE, member_decl, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let fndecl = ast::FunctionDecl::cast(ptree.cursor()).unwrap();
	assert!(fndecl.is_const());
}

#[test]
fn smoke_varargs() {
	const SOURCE: &str = r#"void bashibozuk(int a, float b, ...) const {}"#;

	let ptree: ParseTree = crate::parse(SOURCE, member_decl, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let fndecl = ast::FunctionDecl::cast(ptree.cursor()).unwrap();
	let params = fndecl.param_list().unwrap();
	assert!(params.varargs());
}

// Actor ///////////////////////////////////////////////////////////////////////

#[test]
fn smoke_states_block() {
	const SOURCE: &str = "States { Spawn: XZW1 A 33; XZW1 B 2; Loop; }";

	let ptree: ParseTree = crate::parse(SOURCE, states_block, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_goto() {
	const SOURCE: &str = r#####"States {
goto Super::FrameSetup;
}"#####;

	let ptree: ParseTree = crate::parse(SOURCE, states_block, zdoom::lex::Context::ZSCRIPT_LATEST);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}
