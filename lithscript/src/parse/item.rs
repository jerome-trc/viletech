//! Function and structure declarations, enums, unions, symbolic constants, et cetera.

#[cfg(test)]
mod test {
	use doomfront::{gcache::GreenCacheNoop, rowan::ast::AstNode, testing::*};

	use crate::{ast, ParseTree, Version};

	use super::*;

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

		let vers = Version::new(0, 0, 0);
		let builder = ParserBuilder::<GreenCacheNoop>::new(vers);
		let parser = builder.file();
		let tbuf = doomfront::scan(SOURCE, vers);
		let result = doomfront::parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);

		let mut ast = ptree
			.cursor()
			.children()
			.map(|root| ast::FileRoot::cast(root).unwrap());

		let fndecl1 = ast.next().unwrap().into_func_decl().unwrap();
		assert_eq!(fndecl1.name().text(), "conductor");

		let fndecl2 = ast.next().unwrap().into_func_decl().unwrap();
		assert_eq!(fndecl2.name().text(), "faultline");
	}
}
