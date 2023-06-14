//! Function and structure declarations, enums, unions, symbolic constants, et cetera.

use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	comb,
	gcache::GreenCache,
	parser_t,
	parsing::*,
	rowan::GreenNode,
	GreenElement,
};

use crate::Syn;

use super::ParserBuilder;

impl<C: GreenCache> ParserBuilder<C> {
	/// Builds a [`Syn::FuncDecl`] node.
	pub fn func_decl<'i>(&self) -> parser_t!(Syn, GreenNode) {
		let body = primitive::group((comb::just(Syn::BraceL), comb::just(Syn::BraceR)))
			.map(|group| coalesce_node(group, Syn::Block));

		primitive::group((
			self.annotations(),
			// Q: Qualifiers?
			comb::just(Syn::KwFunc),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			self.param_list(),
			self.trivia_0plus(),
			self.type_spec().or_not(),
			primitive::choice((
				body.map(GreenElement::from),
				comb::just(Syn::Semicolon).map(GreenElement::from),
			)),
		))
		.map(|group| coalesce_node(group, Syn::FuncDecl))
	}

	/// Builds a [`Syn::ParamList`] node.
	pub fn param_list<'i>(&self) -> parser_t!(Syn, GreenNode) {
		let param = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just(Syn::Colon),
			self.trivia_0plus(),
			self.expr(),
		));

		primitive::group((
			comb::just(Syn::ParenL),
			self.trivia_0plus(),
			param.clone().or_not(),
			primitive::group((
				self.trivia_0plus(),
				comb::just(Syn::Comma),
				self.trivia_0plus(),
				param,
			))
			.repeated()
			.collect::<Vec<_>>(),
			self.trivia_0plus(),
			comb::just(Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::ParamList))
	}
}

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
func atmospheric_pressure():vzs.int32;
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
