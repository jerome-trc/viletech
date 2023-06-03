//! Function and structure declarations, enums, unions, symbolic constants, et cetera.

use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	comb,
	util::builder::GreenCache,
};

use crate::{Syn, TokenStream};

use super::{Extra, ParserBuilder};

impl<C: GreenCache> ParserBuilder<C> {
	/// Builds a [`Syn::FuncDecl`] node.
	pub fn func_decl<'i>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		let body = comb::node(
			Syn::Block.into(),
			primitive::group((comb::just(Syn::BraceL), comb::just(Syn::BraceR))),
		);

		let ret_ty = comb::node(
			Syn::ReturnType.into(),
			primitive::group((
				comb::just(Syn::Colon),
				self.trivia_0plus(),
				self.type_expr(self.expr()),
				self.trivia_0plus(),
			)),
		);

		comb::node(
			Syn::FuncDecl.into(),
			primitive::group((
				self.annotations(),
				// Q: Qualifiers?
				comb::just(Syn::KwFunc),
				self.trivia_1plus(),
				self.ident(),
				self.trivia_0plus(),
				self.param_list(),
				self.trivia_0plus(),
				ret_ty.or_not(),
				primitive::choice((body, comb::just(Syn::Semicolon))),
			)),
		)
	}

	/// Builds a [`Syn::ParamList`] node.
	pub fn param_list<'i>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		let param = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just(Syn::Colon),
			self.trivia_0plus(),
			self.type_expr(self.expr()),
		));

		comb::node(
			Syn::ParamList.into(),
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
				.collect::<()>(),
				self.trivia_0plus(),
				comb::just(Syn::ParenR),
			)),
		)
	}
}

#[cfg(test)]
mod test {
	use doomfront::{
		rowan::ast::AstNode,
		util::{builder::GreenCacheNoop, testing::*},
	};

	use crate::{ast, Version};

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
		let stream = Syn::stream(SOURCE, vers);
		let ptree = doomfront::parse(parser, None, Syn::FileRoot.into(), SOURCE, stream);

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
