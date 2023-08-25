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
func conductor();
/// Here, perhaps, is some useful information.
func faultline():void{}
#[annotation]
func atmospheric_pressure():lith.int32;
#[annotation()]
/// Well, it can't be that useful. It's unit test placeholder nonsense.
func untilted(): type(1) {}
#[annotation(deadly_town)]
func liquid_luck(within: 'reach???') {}
#[annotation(wasteland: pyrarinth)]
/// But if you're reading this, maybe it's at least
/// the slightest bit entertaining.
func quicksilver(heliotrope: escape.velocity) { }
#[annotation(battle_strategy, astral: 'dreadnought')]
func atomic(thecry: coal.yaw);
"#;

	let ptree: ParseTree = doomfront::parse(SOURCE, file, crate::Version::new(0, 0, 0));

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let mut ast = ptree
		.cursor()
		.children()
		.map(|root| ast::TopLevel::cast(root).unwrap());

	let fndecl0 = ast.next().unwrap().into_func_decl().unwrap();
	assert_eq!(fndecl0.name().text(), "conductor");

	let fndecl1 = ast.next().unwrap().into_func_decl().unwrap();
	assert_eq!(fndecl1.name().text(), "faultline");
}

#[test]
fn smoke_import() {
	const SOURCE: &str = r#"
import "/digital/nomad.lith": * => crawler;
import "pressure/cooker.lith": urchin;
import "inhabitants.lith": {dream, dweller};
import	"/in/search/of/an/answer.lith" : { necrocosmic , alchemical=>apparatus };
"#;

	let ptree: ParseTree = doomfront::parse(SOURCE, file, crate::Version::new(0, 0, 0));

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());

	let mut ast = ptree
		.cursor()
		.children()
		.map(|root| ast::TopLevel::cast(root).unwrap());

	{
		let import = ast.next().unwrap().into_import().unwrap();
		assert_eq!(
			import.file_path().unwrap().text(),
			r#""/digital/nomad.lith""#
		);
		assert_eq!(import.all_alias().unwrap().text(), "crawler");
	}

	{
		let import = ast.next().unwrap().into_import().unwrap();
		assert!(import.group().is_none());
		assert_eq!(import.single().unwrap().name().text(), "urchin");
	}

	{
		let import = ast.next().unwrap().into_import().unwrap();
		let mut igrp = import.group().unwrap().entries();
		assert_eq!(igrp.next().unwrap().name().text(), "dream");
		assert_eq!(igrp.next().unwrap().name().text(), "dweller");
	}

	{
		let import = ast.next().unwrap().into_import().unwrap();
		let mut igrp = import.group().unwrap().entries();
		let e0 = igrp.next().unwrap();
		assert_eq!(e0.name().text(), "necrocosmic");
		assert!(e0.rename().is_none());
		let e1 = igrp.next().unwrap();
		assert_eq!(e1.name().text(), "alchemical");
		assert_eq!(e1.rename().unwrap().text(), "apparatus");
	}
}
