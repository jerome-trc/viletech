use rowan::ast::AstNode;

use crate::{
	testing::*,
	zdoom::{
		self,
		decorate::{ast, parse, ParseTree, Syn},
	},
};

#[test]
#[ignore]
fn with_sample_dir() {
	let dir = match check_sample_dir("DOOMFRONT_DECORATE_SAMPLE_DIR") {
		Ok(p) => p,
		Err(err) => {
			eprintln!("Skipping DECORATE sample data-based unit test. Reason: {err}");
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
		let ptree: ParseTree = crate::parse(&sample, super::file, zdoom::lex::Context::NON_ZSCRIPT);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn smoke_expr() {
	const SOURCE: &str = "x ^ ((a * b) + (c / d)) | y & z && foo";

	let ptree: ParseTree =
		crate::parse(SOURCE, parse::expr::expr, zdoom::lex::Context::NON_ZSCRIPT);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn color_expr() {
	const SOURCE: &str = "72e0f1";

	let ptree: ParseTree =
		crate::parse(SOURCE, parse::expr::expr, zdoom::lex::Context::NON_ZSCRIPT);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn call_no_args() {
	const SOURCE: &str = "nobody_told_me_about_decorate()";

	let ptree: ParseTree =
		crate::parse(SOURCE, parse::expr::expr, zdoom::lex::Context::NON_ZSCRIPT);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn call_with_rng() {
	const SOURCE: &str = "set_random_seed[rngtbl](1234567890)";

	let ptree: ParseTree =
		crate::parse(SOURCE, parse::expr::expr, zdoom::lex::Context::NON_ZSCRIPT);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

// Statements //////////////////////////////////////////////////////////////////

#[test]
fn smoke_for() {
	const SOURCE: &str = "for (user_int; --user_int; user_int++) {}";

	let ptree: ParseTree = crate::parse(SOURCE, parse::statement, zdoom::lex::Context::NON_ZSCRIPT);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

// Non-actor top level /////////////////////////////////////////////////////////

#[test]
fn smoke_damagetype() {
	const SOURCE: &str = r#"DamageType NullDamage
	{
	   Factor 0
	   NoArmor
	   ReplaceFactor
	}"#;

	let ptree: ParseTree = crate::parse(
		SOURCE.trim(),
		parse::damage_type,
		zdoom::lex::Context::NON_ZSCRIPT,
	);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_enum_def() {
	const SOURCE: &str = r#"enum {
	LIMBO,
	DIS = LIMBO,
	WARRENS = 0,
	MYST_FORT = 9.9,
	MT_EREBUS = false,
	CATHEDRAL = "Yes, string literals are valid enum initializers in DECORATE!",
};"#;

	let ptree: ParseTree = crate::parse(
		SOURCE.trim(),
		parse::enum_def,
		zdoom::lex::Context::NON_ZSCRIPT,
	);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let cursor = ptree.cursor();
	let enumdef = ast::TopLevel::cast(cursor.children().next().unwrap())
		.unwrap()
		.into_enumdef()
		.unwrap();
	let mut variants = enumdef.variants();

	let var1 = variants.next().unwrap();
	assert_eq!(var1.name().text(), "LIMBO");
	assert!(var1.initializer().is_none());

	let var2 = variants.next().unwrap();
	assert_eq!(var2.name().text(), "DIS");
	assert_eq!(
		var2.initializer()
			.unwrap()
			.into_ident()
			.unwrap()
			.token()
			.text(),
		"LIMBO"
	);

	let var7 = variants.last().unwrap();

	assert_eq!(var7.name().text(), "CATHEDRAL");
	assert_eq!(
		var7.initializer()
			.unwrap()
			.into_literal()
			.unwrap()
			.token()
			.string()
			.unwrap(),
		"Yes, string literals are valid enum initializers in DECORATE!"
	);
}

#[test]
fn smoke_include_directive() {
	const SOURCE: &str = "#InClUdE \"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\"";

	let ptree: ParseTree = crate::parse(
		SOURCE,
		parse::include_directive,
		zdoom::lex::Context::NON_ZSCRIPT,
	);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let cursor = ptree.cursor();

	assert_sequence(
		&[
			(Syn::IncludeDirective, None),
			(Syn::KwInclude, Some("#InClUdE")),
			(Syn::Whitespace, Some(" ")),
			(
				Syn::StringLit,
				Some("\"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\""),
			),
		],
		cursor.clone(),
	);

	let incdir = ast::IncludeDirective::cast(cursor).unwrap();

	assert_eq!(
		incdir.path().text(),
		"\"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\""
	);
}

#[test]
fn smoke_constants() {
	const SOURCE: &str = r##"

const /* bools are */ int KNEE_DEEP = 1234567890;
const float SHORES_INFERNO /* forbidden */ = 0.9999999;

"##;

	let ptree: ParseTree = crate::parse(SOURCE, parse::file, zdoom::lex::Context::NON_ZSCRIPT);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
	let cursor = ptree.cursor();

	let mut constdefs = cursor
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap().into_constdef().unwrap());

	{
		let constant = constdefs.next().unwrap();

		assert_eq!(constant.name().text(), "KNEE_DEEP");
		assert_eq!(constant.type_spec(), ast::ConstType::Int);
		assert_eq!(
			constant
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.int()
				.unwrap()
				.unwrap()
				.0,
			1234567890
		);
	}

	{
		let constant = constdefs.next().unwrap();

		assert_eq!(constant.name().text(), "SHORES_INFERNO");
		assert_eq!(constant.type_spec(), ast::ConstType::Float);
		assert_eq!(
			constant
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap()
				.unwrap(),
			0.9999999
		);
	}
}

// Actors //////////////////////////////////////////////////////////////////////

#[test]
fn smoke_actor() {
	const SOURCE: &str = r#####"
aCtOr hangar : nuclearplant replaces toxinrefinery 10239 {
enum {
	CMDCTRL,
	PHOBOSLAB = CMDCTRL,
	CENTPROC = 0,
	COMPSTAT = 9.9,
	PHOBOSANOMALY = false,
	MILBASE = "Yes, string literals are valid enum initializers in DECORATE!",
}; // Floats and booleans too.

CONST int DEIMOSANOMALY = 1234567890;

var int dark_halls;
var float hidingTheSecrets;

ResetAllFlagsOrSomething
Containment.Area "unruly",0123
+REFINERY
DropItem "CMDCENTER" 255 1 PainSound "spawning/vats"

States(Actor, overlay, ITEM, WeapoN) {
	Spawn: TNT1 A Random(1, 6)
	Wickedly:
		____ "#" 0
		goto super::Spawn.Something + 0
	Repent:
		3HA_ A 1 bright light('perfect')
		"####" B 6 canraise fast nodelay slow A_SpawnItemEx(1, "??")
		"----" "]" -1 offset(-1, 1) light("sever")
		Loop
}
}
	"#####;

	let ptree: ParseTree = crate::parse(
		SOURCE.trim(),
		parse::actor_def,
		zdoom::lex::Context::NON_ZSCRIPT,
	);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let cursor = ptree.cursor();
	let toplevel = ast::TopLevel::cast(cursor.first_child().unwrap()).unwrap();

	let actordef = match toplevel {
		ast::TopLevel::ActorDef(inner) => inner,
		other => panic!("expected `ActorDef`, found: {other:#?}"),
	};

	assert_eq!(actordef.name().text(), "hangar");

	assert_eq!(
		actordef
			.base_class()
			.expect("actor definition has no base class")
			.text(),
		"nuclearplant"
	);

	assert_eq!(
		actordef
			.replaced_class()
			.expect("actor definition has no replacement clause")
			.text(),
		"toxinrefinery"
	);

	assert_eq!(
		actordef
			.editor_number()
			.expect("actor definition has no editor number")
			.text()
			.parse::<u16>()
			.expect("actor editor number is not a valid u16"),
		10239
	);
}

#[test]
fn actor_identifiers() {
	const SOURCE: &str = "ACTOR SpiderBullets:Inventory{Inventory.MaxAmount 30}";

	let ptree: ParseTree = crate::parse(SOURCE, parse::actor_def, zdoom::lex::Context::NON_ZSCRIPT);
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_states() {
	const SOURCE: &str = r#####"
	States
	{
	ClearTarget:
		TNT1 A 0 A_ClearTarget
		TNT1 A 0 A_LookEx (LOF_NOSEESOUND, 0, 0, 0, 360, "SeeLoop")
		Goto SeeLoop
	See:
		ARAC A 20
	SeeLoop:
	}
	"#####;

	let ptree: ParseTree = crate::parse(
		SOURCE.trim(),
		parse::actor::states_block,
		zdoom::lex::Context::NON_ZSCRIPT,
	);

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}
