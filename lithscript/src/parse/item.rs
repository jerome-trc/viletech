//! Function and structure declarations, enums, unions, symbolic constants, et cetera.

#[cfg(test)]
mod test {
	use doomfront::{rowan::ast::AstNode, testing::*};

	use crate::{ast, ParseTree};

	#[test]
	fn smoke_func_decl() {
		const SOURCE: &str = r#"
func conductor();
func r#faultline():void{}
func atmospheric_pressure():lith.int32;
func untilted(): type(1) {}
func liquid_luck(within: reach) {}
func quicksilver(heliotrope: escape.velocity) {}
func atomic(r#123: coal.r#yaw);
"#;

		let root = crate::parse::file(SOURCE).unwrap();
		let ptree = ParseTree::new(root, vec![]);

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

		let root = crate::parse::file(SOURCE).unwrap();
		let ptree = ParseTree::new(root, vec![]);

		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());

		let mut ast = ptree
			.cursor()
			.children()
			.map(|root| ast::TopLevel::cast(root).unwrap());

		let import0 = ast.next().unwrap().into_import().unwrap();
		assert_eq!(import0.module().unwrap().text(), "\"/digital/nomad.lith\"");
		assert_eq!(import0.all_alias().unwrap().text(), "crawler");

		let import1 = ast.next().unwrap().into_import().unwrap();
		assert!(import1.list().is_none());
		assert_eq!(import1.single().unwrap().name().text(), "urchin");

		let import2 = ast.next().unwrap().into_import().unwrap();
		let mut ilist2 = import2.list().unwrap().entries();
		assert_eq!(ilist2.next().unwrap().name().text(), "dream");
		assert_eq!(ilist2.next().unwrap().name().text(), "dweller");

		let import3 = ast.next().unwrap().into_import().unwrap();
		let mut ilist3 = import3.list().unwrap().entries();
		let import3_e0 = ilist3.next().unwrap();
		assert_eq!(import3_e0.name().text(), "necrocosmic");
		assert!(import3_e0.rename().is_none());
		let import3_e1 = ilist3.next().unwrap();
		assert_eq!(import3_e1.name().text(), "alchemical");
		assert_eq!(import3_e1.rename().unwrap().text(), "apparatus");
	}
}
