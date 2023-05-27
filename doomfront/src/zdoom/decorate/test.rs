use rowan::{ast::AstNode, SyntaxNode};

use crate::{
	util::{builder::GreenCacheNoop, testing::*},
	zdoom::decorate::{
		ast::{self},
		parse, Syn,
	},
};

#[test]
fn actor_def() {
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

	ResetAllFlagsOrSomething
	Containment.Area
	+REFINERY
	DropItem "CMDCENTER" 255 1 PainSound "spawning/vats"

	States(Actor) {
		Spawn: TNT1 A Random(1, 6)
		Wickedly:
			____ "#" 0
			goto super::Spawn.Something + 0
		Repent:
			3HA_ A 1 bright light('perfect')
			"####" B 6 canraise fast nodelay slow A_SpawnItemEx [rngtbl] (1, "??")
			"----" "]" -1 offset(-1, 1) light("sever")
			Loop
	}

	-TOWER.OF.BABEL
	Decal mysteryfortress
	ClassReference 'Pandemonium'

}
"#####;

	let ptree = parse::parse::<GreenCacheNoop>(SOURCE, None).unwrap();
	let cursor = SyntaxNode::<Syn>::new_root(ptree.root);
	let toplevel = ast::TopLevel::cast(cursor.first_child().unwrap()).unwrap();

	let actordef = match toplevel {
		ast::TopLevel::ActorDef(inner) => inner,
		other => panic!("Expected `ActorDef`, found: {other:#?}"),
	};

	assert_eq!(actordef.name().text(), "hangar");

	assert_eq!(
		actordef
			.base_class()
			.expect("Actor definition has no base class.")
			.text(),
		"nuclearplant"
	);

	assert_eq!(
		actordef
			.replaced_class()
			.expect("Actor definition has no replacement clause.")
			.text(),
		"toxinrefinery"
	);

	assert_eq!(
		actordef
			.editor_number()
			.expect("Actor definition has no editor number.")
			.text()
			.parse::<u16>()
			.expect("Actor editor number is not a valid u16."),
		10239
	);

	let mut innards = actordef.innards();

	let _ = innards.next().unwrap().into_enumdef().unwrap();
	let constdef = innards.next().unwrap().into_constdef().unwrap();
	assert_eq!(constdef.name().text(), "DEIMOSANOMALY");
	assert_eq!(constdef.type_spec(), ast::ConstType::Int);

	let _ = innards.next().unwrap().into_propsettings().unwrap();

	let flag1 = innards.next().unwrap().into_flagsetting().unwrap();
	assert!(flag1.is_adding());
	assert_eq!(flag1.name().syntax().text(), "REFINERY");

	let _ = innards.next().unwrap().into_propsettings().unwrap();

	let statesdef = innards.next().unwrap().into_statesdef().unwrap();
	assert_eq!(statesdef.usage_qual().unwrap(), ast::StateUsage::Actor);

	let mut state_items = statesdef.items();

	let label1 = state_items.next().unwrap().into_label().unwrap();
	assert_eq!(label1.token().text(), "Spawn");

	let state1 = state_items.next().unwrap().into_state().unwrap();

	assert_eq!(state1.sprite().text(), "TNT1");
	assert_eq!(state1.frames().text(), "A");

	let state1_dur = state1.duration().into_node().unwrap();
	assert_eq!(
		ast::ExprCall::cast(state1_dur).unwrap().name().text(),
		"Random"
	);

	let label2 = state_items.next().unwrap().into_label().unwrap();
	assert_eq!(label2.token().text(), "Wickedly");

	let _state2 = state_items.next().unwrap().into_state().unwrap();
	let change1 = state_items.next().unwrap().into_change().unwrap();
	match change1 {
		ast::StateChange::Goto {
			target,
			offset,
			super_target,
		} => {
			assert_eq!(target.syntax().text(), "Spawn.Something");
			assert_eq!(offset.unwrap(), 0);
			assert!(super_target);
		}
		other => panic!("Expected `StateChange::Goto`, found: {other:#?}"),
	}

	let _ = state_items.next().unwrap().into_label().unwrap();
	let state3 = state_items.next().unwrap().into_state().unwrap();
	assert_eq!(state3.light().unwrap().text(), "'perfect'");

	let state4 = state_items.next().unwrap().into_state().unwrap();
	let mut state4_quals = state4.qualifiers();

	assert!(matches!(
		state4_quals.next().unwrap(),
		ast::StateQual::CanRaise
	));
	assert!(matches!(state4_quals.next().unwrap(), ast::StateQual::Fast));
	assert!(matches!(
		state4_quals.next().unwrap(),
		ast::StateQual::NoDelay
	));
	assert!(matches!(state4_quals.next().unwrap(), ast::StateQual::Slow));
}

#[test]
fn enum_def() {
	const SOURCE: &str = r#"

enum {
	LIMBO,
	DIS = LIMBO,
	WARRENS = 0,
	MYST_FORT = 9.9,
	MT_EREBUS = false,
	CATHEDRAL = "Yes, string literals are valid enum initializers in DECORATE!",
}; // Floats and booleans too.

"#;

	let ptree = parse::parse::<GreenCacheNoop>(SOURCE, None).unwrap();
	let cursor = SyntaxNode::<Syn>::new_root(ptree.root);
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
fn include_directive() {
	const SOURCE: &str = " #InClUdE \"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\"";

	let ptree = parse::parse::<GreenCacheNoop>(SOURCE, None).unwrap();
	let cursor = SyntaxNode::<Syn>::new_root(ptree.root);

	assert_sequence(
		&[
			(Syn::Root, None),
			(Syn::Whitespace, Some(" ")),
			(Syn::IncludeDirective, None),
			(Syn::PreprocInclude, Some("#InClUdE")),
			(Syn::Whitespace, Some(" ")),
			(
				Syn::LitString,
				Some("\"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\""),
			),
		],
		cursor.clone(),
	);

	let incdirect = match ast::TopLevel::cast(cursor.first_child().unwrap()).unwrap() {
		ast::TopLevel::IncludeDirective(inner) => inner,
		other => panic!("Expected `IncludeDirective`, found: {other:#?}"),
	};

	assert_eq!(
		incdirect.path().text(),
		"\"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\""
	);
}

#[test]
fn symbolic_constants() {
	const SOURCE: &str = r##"

const /* bools */ int KNEE_DEEP = 1234567890;
const fixed /* are */ SHORES = 9.0000000;
const float INFERNO /* forbidden */ = 0.9999999;

"##;

	let ptree = parse::parse::<GreenCacheNoop>(SOURCE, None).unwrap();
	let cursor = SyntaxNode::<Syn>::new_root(ptree.root);
	let mut constdefs = cursor
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap().into_constdef().unwrap());

	let constdef1 = constdefs.next().unwrap();

	assert_eq!(constdef1.name().text(), "KNEE_DEEP");
	assert_eq!(constdef1.type_spec(), ast::ConstType::Int);
	assert_eq!(
		constdef1
			.expr()
			.into_literal()
			.unwrap()
			.token()
			.int()
			.unwrap()
			.unwrap(),
		1234567890
	);

	let constdef2 = constdefs.next().unwrap();

	assert_eq!(constdef2.name().text(), "SHORES");
	assert_eq!(constdef2.type_spec(), ast::ConstType::Fixed);
	assert_eq!(
		constdef2
			.expr()
			.into_literal()
			.unwrap()
			.token()
			.float()
			.unwrap(),
		9.0000000
	);

	let constdef3 = constdefs.next().unwrap();

	assert_eq!(constdef3.name().text(), "INFERNO");
	assert_eq!(constdef3.type_spec(), ast::ConstType::Float);
	assert_eq!(
		constdef3
			.expr()
			.into_literal()
			.unwrap()
			.token()
			.float()
			.unwrap(),
		0.9999999
	);
}
