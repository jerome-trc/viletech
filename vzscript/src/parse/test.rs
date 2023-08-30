use doomfront::{rowan::ast::AstNode, testing::*};

use crate::{ast, ParseTree};

use super::{expr::*, file};

#[test]
fn smoke_expr_for() {
	const SOURCE: &str = r#"for helio : trope {}"#;

	let ptree: ParseTree = doomfront::parse(
		SOURCE,
		|p| {
			Expression::parse(p);
		},
		crate::Version::new(0, 0, 0),
	);

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

#[test]
fn smoke_func_decl() {
	const SOURCE: &str = r#"
function conductor();
/// Here, perhaps, is some useful information.
function faultline():void{}
#[annotation]
function atmospheric_pressure():vzs.int32;
#[annotation()]
/// Well, it can't be that useful. It's unit test placeholder nonsense.
function untilted(): type(1) {}
#[annotation(deadly_town)]
function liquid_luck(within: 'reach???') {}
#[annotation(wasteland: pyrarinth)]
/// But if you're reading this, maybe it's at least
/// the slightest bit entertaining.
function quicksilver(heliotrope: escape.velocity) { }
#[annotation(battle_strategy, astral: 'dreadnought')]
function atomic(thecry: coal.yaw);
"#;

	let ptree: ParseTree = doomfront::parse(SOURCE, file, crate::Version::new(0, 0, 0));

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let mut ast = ptree
		.cursor()
		.children()
		.map(|root| ast::TopLevel::cast(root).unwrap());

	{
		let ast::TopLevel::FuncDecl(fndecl) = ast.next().unwrap() else { panic!() };
		assert_eq!(fndecl.name().unwrap().text(), "conductor");
	}

	{
		let ast::TopLevel::FuncDecl(fndecl) = ast.next().unwrap() else { panic!() };
		assert_eq!(fndecl.name().unwrap().text(), "faultline");
	}
}
