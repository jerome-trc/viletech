//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
	/// The returned parser emits a [`Syn::ArrayLen`] node.
	pub(super) fn array_len<'i>(&self) -> parser_t!(Vec<GreenNode>) {
		primitive::group((
			comb::just_ts(Token::BracketL, Syn::BracketL),
			self.trivia_0plus(),
			self.expr().or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR),
		))
		.map(|group| coalesce_node(group, Syn::ArrayLen))
		.repeated()
		.at_least(1)
		.collect()
	}

	/// The returned parser emits a [`Syn::DeprecationQual`] node.
	pub fn deprecation_qual<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwDeprecated, Syn::KwDeprecated),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenR),
			self.trivia_0plus(),
			comb::just_ts(Token::StringLit, Syn::StringLit),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				comb::just_ts(Token::StringLit, Syn::StringLit),
			))
			.or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::DeprecationQual))
	}

	/// The returned parser emits a [`Syn::Ident`] token.
	pub(super) fn ident<'i>(&self) -> parser_t!(GreenToken) {
		primitive::any()
			.filter(|token: &Token| {
				matches!(
					token,
					Token::Ident
						| Token::KwBright | Token::KwFast
						| Token::KwSlow | Token::KwNoDelay
						| Token::KwCanRaise | Token::KwOffset
						| Token::KwLight
				)
			})
			.map_with_state(comb::green_token(Syn::Ident))
	}

	/// The returned parser emits a [`Syn::IdentChain`] node.
	pub fn ident_chain<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.ident(),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Dot, Syn::Dot),
				self.ident(),
			))
			.repeated()
			.collect::<Vec<_>>(),
		))
		.map(|group| coalesce_node(group, Syn::IdentChain))
	}

	/// The returned parser emits a series of [`Syn::Ident`] tokens (comma-separated).
	pub fn ident_list<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.ident().map(|gtok| vec![gtok.into()]).foldl(
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				self.ident(),
			))
			.repeated(),
			|mut lhs, (mut t0, comma, mut t1, ident)| {
				lhs.append(&mut t0);
				lhs.push(comma.into());
				lhs.append(&mut t1);
				lhs.push(ident.into());
				lhs
			},
		)
	}

	/// The returned parser emits a [`Syn::Whitespace`] or [`Syn::Comment`] token.
	pub(super) fn trivia<'i>(&self) -> parser_t!(GreenElement) {
		primitive::choice((
			comb::just_ts(Token::Whitespace, Syn::Whitespace),
			comb::just_ts(Token::Comment, Syn::Comment),
			comb::just_ts(Token::RegionStart, Syn::RegionStart),
			comb::just_ts(Token::RegionEnd, Syn::RegionEnd),
		))
		.map(|token| token.into())
	}

	/// Shorthand for `self.trivia().repeated().collect()`.
	pub(super) fn trivia_0plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().collect()
	}

	/// Shorthand for `self.trivia().repeated().at_least(1).collect()`.
	pub(super) fn trivia_1plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().at_least(1).collect()
	}

	/// The returned parser emits a [`Syn::TypeRef`] node.
	pub fn type_ref<'i>(&self) -> parser_t!(GreenNode) {
		chumsky::recursive::recursive(|tref| {
			let at_ident = primitive::group((comb::just_ts(Token::At, Syn::At), self.ident()))
				.map(|group| coalesce_node(group, Syn::TypeRef));

			let ident = self
				.ident()
				.map(|gtok| GreenNode::new(Syn::IdentChain.into(), [gtok.into()]));

			let readonly = primitive::group((
				comb::just_ts(Token::KwReadonly, Syn::KwReadonly),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				primitive::choice((ident, at_ident.clone())),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_identchain = self
				.ident_chain()
				.map(|gnode| GreenNode::new(Syn::TypeRef.into(), [gnode.into()]));

			let tref_let = comb::just_ts(Token::KwLet, Syn::KwLet)
				.map(|gtok| GreenNode::new(Syn::TypeRef.into(), [gtok.into()]));

			let simple = primitive::choice((readonly, at_ident, tref_identchain, tref_let));

			let tref_or_fixedlen_array =
				primitive::group((tref.clone(), self.array_len().or_not())).map(coalesce_vec);

			let class_restrictor = primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				self.ident_chain(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(coalesce_vec);

			let tref_class = primitive::group((
				comb::just_ts(Token::KwClass, Syn::KwClass),
				class_restrictor.or_not(),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_array_dyn = primitive::group((
				comb::just_ts(Token::KwArray, Syn::KwArray),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_map = primitive::group((
				comb::just_ts(Token::KwMap, Syn::KwMap),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_mapiter = primitive::group((
				comb::just_ts(Token::KwMapIterator, Syn::KwMapIterator),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			primitive::choice((tref_class, tref_array_dyn, tref_map, tref_mapiter, simple)).boxed()
		})
	}

	/// The returned parser emits a [`Syn::VarName`] node.
	pub fn var_name<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.ident(),
			primitive::group((self.trivia_0plus(), self.array_len())).or_not(),
		))
		.map(|group| coalesce_node(group, Syn::VarName))
	}

	/// The returned parser emits a [`Syn::VersionQual`] node.
	pub fn version_qual<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwVersion, Syn::KwVersion),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenL),
			self.trivia_0plus(),
			comb::just_ts(Token::StringLit, Syn::StringLit),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::VersionQual))
	}
}

pub(super) fn ident(p: &mut crate::parser::Parser<Syn>) {
	p.expect_any(
		&[
			(Token::Ident, Syn::Ident),
			(Token::KwBright, Syn::Ident),
			(Token::KwCanRaise, Syn::Ident),
			(Token::KwFast, Syn::Ident),
			(Token::KwLight, Syn::Ident),
			(Token::KwOffset, Syn::Ident),
			(Token::KwSlow, Syn::Ident),
		],
		&["an identifier"],
	)
}

#[must_use]
pub(super) fn eat_ident(p: &mut crate::parser::Parser<Syn>) -> bool {
	p.eat_any(&[
		(Token::Ident, Syn::Ident),
		(Token::KwBright, Syn::Ident),
		(Token::KwCanRaise, Syn::Ident),
		(Token::KwFast, Syn::Ident),
		(Token::KwLight, Syn::Ident),
		(Token::KwOffset, Syn::Ident),
		(Token::KwSlow, Syn::Ident),
	])
}

pub(super) fn trivia(p: &mut crate::parser::Parser<Syn>) -> bool {
	p.eat(Token::Whitespace, Syn::Whitespace) || p.eat(Token::Comment, Syn::Comment)
}

pub(super) fn trivia_0plus(p: &mut crate::parser::Parser<Syn>) {
	while trivia(p) {}
}

pub(super) fn _trivia_1plus(p: &mut crate::parser::Parser<Syn>) {
	p.expect_any(
		&[
			(Token::Whitespace, Syn::Whitespace),
			(Token::Comment, Syn::Comment),
		],
		&["whitespace or a comment (one or more)"],
	);

	trivia_0plus(p);
}

#[cfg(test)]
#[cfg(any())]
mod test {
	use crate::{
		testing::*,
		zdoom::{zscript::ParseTree, Version},
	};

	use super::*;

	#[test]
	fn smoke_identlist() {
		const SOURCE: &str = r#"temple, of, the, ancient, techlords"#;

		let tbuf = crate::_scan(SOURCE, Version::default());
		let parser = ParserBuilder::new(Version::default())
			.ident_list()
			.map(|elems| GreenNode::new(Syn::Root.into(), elems));
		let result = crate::_parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_typeref() {
		const SOURCES: &[&str] = &[
			"TeenyLittleBase",
			"Dead.On.Arrival",
			"readonly<Corruption2Factory>",
			"class",
			"class<Forge>",
			"array<Unwelcome>",
			"array<class<TheOssuary> >",
			"map<Corruption[1], Mortem[2][3]>",
			"mapiterator<FishInABarrel, Neoplasm>",
		];

		for source in SOURCES {
			let tbuf = crate::_scan(source, Version::default());
			let parser = ParserBuilder::new(Version::default()).type_ref();
			let result = crate::_parse(parser, source, &tbuf);
			let ptree: ParseTree = unwrap_parse_tree(result);
			assert_no_errors(&ptree);
		}
	}

	#[test]
	fn smoke_version_qual() {
		const SOURCE: &str = r#"version("3.7.1")"#;

		let tbuf = crate::_scan(SOURCE, Version::default());
		let parser = ParserBuilder::new(Version::default()).version_qual();
		let result = crate::_parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_deprecation_qual() {
		const SOURCE: &str = r#"deprecated("2.4.0", "Don't use this please")"#;

		let tbuf = crate::_scan(SOURCE, Version::default());
		let parser = ParserBuilder::new(Version::default()).deprecation_qual();
		let result = crate::_parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
	}
}
