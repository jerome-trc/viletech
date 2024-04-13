//! Abstract syntax tree nodes.

mod actor;
mod expr;
mod stat;
mod structure;
mod types;

use std::{
	num::IntErrorKind,
	path::{Path, PathBuf},
};

use rowan::{ast::AstNode, Language};

use crate::{
	simple_astnode,
	zdoom::{self, ast::LitToken},
	AstError, AstResult,
};

use super::{Syntax, SyntaxNode, SyntaxToken};

pub use self::{actor::*, expr::*, stat::*, structure::*, types::*};

/// A top-level element in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TopLevel {
	ClassDef(ClassDef),
	ClassExtend(ClassExtend),
	ConstDef(ConstDef),
	EnumDef(EnumDef),
	MixinClassDef(MixinClassDef),
	Include(IncludeDirective),
	StructDef(StructDef),
	StructExtend(StructExtend),
	Version(VersionDirective),
}

impl AstNode for TopLevel {
	type Language = Syntax;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::ClassDef
				| Syntax::ClassExtend
				| Syntax::ConstDef
				| Syntax::EnumDef
				| Syntax::MixinClassDef
				| Syntax::IncludeDirective
				| Syntax::StructDef
				| Syntax::StructExtend
				| Syntax::VersionDirective
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::ClassDef => Some(Self::ClassDef(ClassDef(node))),
			Syntax::ClassExtend => Some(Self::ClassExtend(ClassExtend(node))),
			Syntax::ConstDef => Some(Self::ConstDef(ConstDef(node))),
			Syntax::EnumDef => Some(Self::EnumDef(EnumDef(node))),
			Syntax::MixinClassDef => Some(Self::MixinClassDef(MixinClassDef(node))),
			Syntax::IncludeDirective => Some(Self::Include(IncludeDirective(node))),
			Syntax::StructDef => Some(Self::StructDef(StructDef(node))),
			Syntax::StructExtend => Some(Self::StructExtend(StructExtend(node))),
			Syntax::VersionDirective => Some(Self::Version(VersionDirective(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			TopLevel::ClassDef(inner) => inner.syntax(),
			TopLevel::ClassExtend(inner) => inner.syntax(),
			TopLevel::ConstDef(inner) => inner.syntax(),
			TopLevel::EnumDef(inner) => inner.syntax(),
			TopLevel::MixinClassDef(inner) => inner.syntax(),
			TopLevel::Include(inner) => inner.syntax(),
			TopLevel::StructDef(inner) => inner.syntax(),
			TopLevel::StructExtend(inner) => inner.syntax(),
			TopLevel::Version(inner) => inner.syntax(),
		}
	}
}

// ConstDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ConstDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConstDef(SyntaxNode);

simple_astnode!(Syntax, ConstDef, Syntax::ConstDef);

impl ConstDef {
	/// The returned token is always tagged [`Syntax::KwConst`].
	#[must_use]
	pub fn keyword(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwConst)
			})
			.unwrap()
	}

	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	pub fn initializer(&self) -> AstResult<Expr> {
		match self.0.last_child() {
			Some(node) => Expr::cast(node).ok_or(AstError::Incorrect),
			None => Err(AstError::Missing),
		}
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(&self.0)
	}
}

// EnumDef /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::EnumDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumDef(SyntaxNode);

simple_astnode!(Syntax, EnumDef, Syntax::EnumDef);

impl EnumDef {
	/// The returned token is always tagged [`Syntax::KwEnum`].
	#[must_use]
	pub fn keyword(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwEnum)
			})
			.unwrap()
	}

	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn type_spec(&self) -> Option<(SyntaxToken, EnumType)> {
		self.0.children_with_tokens().find_map(|elem| {
			let ret1 = match elem.kind() {
				Syntax::KwSByte => EnumType::KwSByte,
				Syntax::KwByte => EnumType::KwByte,
				Syntax::KwInt8 => EnumType::KwInt8,
				Syntax::KwUInt8 => EnumType::KwUInt8,
				Syntax::KwShort => EnumType::KwShort,
				Syntax::KwUShort => EnumType::KwUShort,
				Syntax::KwInt16 => EnumType::KwInt16,
				Syntax::KwUInt16 => EnumType::KwUInt16,
				Syntax::KwInt => EnumType::KwInt,
				Syntax::KwUInt => EnumType::KwUInt,
				_ => return None,
			};

			Some((elem.into_token().unwrap(), ret1))
		})
	}

	pub fn variants(&self) -> impl Iterator<Item = EnumVariant> {
		self.0.children().filter_map(EnumVariant::cast)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(&self.0)
	}
}

/// See [`EnumDef::type_spec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EnumType {
	KwSByte,
	KwByte,
	KwInt8,
	KwUInt8,
	KwShort,
	KwUShort,
	KwInt16,
	KwUInt16,
	KwInt,
	KwUInt,
}

impl std::fmt::Display for EnumType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::KwSByte => write!(f, "sbyte"),
			Self::KwByte => write!(f, "byte"),
			Self::KwInt8 => write!(f, "int8"),
			Self::KwUInt8 => write!(f, "uint8"),
			Self::KwShort => write!(f, "short"),
			Self::KwUShort => write!(f, "ushort"),
			Self::KwInt16 => write!(f, "int16"),
			Self::KwUInt16 => write!(f, "uint16"),
			Self::KwInt => write!(f, "int"),
			Self::KwUInt => write!(f, "uint"),
		}
	}
}

/// Wraps a node tagged [`Syntax::EnumVariant`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumVariant(SyntaxNode);

simple_astnode!(Syntax, EnumVariant, Syntax::EnumVariant);

impl EnumVariant {
	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0.first_token().unwrap()
	}

	#[must_use]
	pub fn initializer(&self) -> Option<Expr> {
		self.0.last_child().map(|node| Expr::cast(node).unwrap())
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(&self.0)
	}
}

// IncludeDirective ////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IncludeDirective`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IncludeDirective(SyntaxNode);

simple_astnode!(Syntax, IncludeDirective, Syntax::IncludeDirective);

impl IncludeDirective {
	/// Yielded tokens are always tagged [`Syntax::StringLit`].
	pub fn strings(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.filter(|elem| elem.kind() == Syntax::StringLit)
			.map(|elem| elem.into_token().unwrap())
	}

	/// Returns `None` if this include directive has no string arguments.
	/// Beware that the returned path will not be canonicalized.
	pub fn include_path<'p, F>(&self, root_dir: &Path, mut parent_path: F) -> Option<PathBuf>
	where
		F: FnMut() -> &'p Path,
	{
		let mut dstrings = self.strings();

		let Some(string_0) = dstrings.next() else {
			return None;
		};

		let path_0 = Path::new(string_0.text().trim_matches('"'));
		let mut comps_0 = path_0.components();

		let inc_path_absolute = matches!(comps_0.next(), Some(std::path::Component::RootDir));

		if inc_path_absolute {
			let mut full_path = root_dir.to_path_buf();

			for comp in comps_0 {
				full_path.push(comp);
			}

			for s in dstrings {
				full_path.push(s.text().trim_matches('"'));
			}

			Some(full_path)
		} else {
			let mut full_path = parent_path().to_path_buf();

			for comp in comps_0 {
				full_path.push(comp);
			}

			for s in dstrings {
				full_path.push(s.text().trim_matches('"'));
			}

			Some(full_path)
		}
	}
}

// VersionDirective ////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::VersionDirective`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VersionDirective(SyntaxNode);

simple_astnode!(Syntax, VersionDirective, Syntax::VersionDirective);

impl VersionDirective {
	/// The returned token is always tagged [`Syntax::StringLit`].
	pub fn string(&self) -> AstResult<LitToken<Syntax>> {
		let token = self.0.last_token().ok_or(AstError::Missing)?;

		match token.kind() {
			Syntax::StringLit => Ok(LitToken::new(token)),
			_ => Err(AstError::Incorrect),
		}
	}

	/// [`IntErrorKind::Empty`] is returned if the expected string literal is absent.
	pub fn version(&self) -> Result<zdoom::Version, IntErrorKind> {
		let lit = self.string().map_err(|_| IntErrorKind::Empty)?;
		let text = lit.string().unwrap();

		if text.is_empty() {
			return Err(IntErrorKind::Empty);
		};

		text.parse()
	}
}

// IdentChain //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IdentChain`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IdentChain(SyntaxNode);

simple_astnode!(Syntax, IdentChain, Syntax::IdentChain);

impl IdentChain {
	/// Each yielded token is tagged [`Syntax::Ident`].
	pub fn parts(&self) -> impl Iterator<Item = SyntaxToken> {
		self.syntax()
			.children_with_tokens()
			.filter_map(|elem| elem.into_token().filter(|tok| tok.kind() == Syntax::Ident))
	}
}

// DeprecationQual /////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::DeprecationQual`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeprecationQual(SyntaxNode);

simple_astnode!(Syntax, DeprecationQual, Syntax::DeprecationQual);

impl DeprecationQual {
	/// The returned token is always tagged [`Syntax::StringLit`].
	pub fn version(&self) -> AstResult<LitToken<Syntax>> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::StringLit)
					.map(LitToken::new)
			})
			.ok_or(AstError::Missing)
	}

	/// The returned token is always tagged [`Syntax::StringLit`].
	#[must_use]
	pub fn message(&self) -> Option<SyntaxToken> {
		self.0
			.children_with_tokens()
			.filter_map(|elem| elem.into_token())
			.skip_while(|token| token.kind() != Syntax::Comma)
			.find_map(|token| (token.kind() == Syntax::StringLit).then_some(token))
	}
}

// VersionQual /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::VersionQual`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VersionQual(SyntaxNode);

simple_astnode!(Syntax, VersionQual, Syntax::VersionQual);

impl VersionQual {
	/// The returned token is always tagged [`Syntax::StringLit`].
	pub fn string(&self) -> AstResult<LitToken<Syntax>> {
		self.0
			.children_with_tokens()
			.skip_while(|elem| elem.kind() != Syntax::ParenL)
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::StringLit)
					.map(LitToken::new)
			})
			.ok_or(AstError::Missing)
	}

	/// [`IntErrorKind::Empty`] is returned if the expected string literal is absent.
	pub fn version(&self) -> Result<zdoom::Version, IntErrorKind> {
		let lit = self.string().map_err(|_| IntErrorKind::Empty)?;
		let text = lit.string().unwrap();

		if text.is_empty() {
			return Err(IntErrorKind::Empty);
		};

		text.parse()
	}
}

// LocalVar ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::LocalVar`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LocalVar(SyntaxNode);

simple_astnode!(Syntax, LocalVar, Syntax::LocalVar);

impl LocalVar {
	pub fn type_ref(&self) -> AstResult<TypeRef> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		TypeRef::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn initializers(&self) -> impl Iterator<Item = LocalVarInit> {
		self.0.children().filter_map(LocalVarInit::cast)
	}
}

/// Wraps a node tagged [`Syntax::LocalVarInit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LocalVarInit(SyntaxNode);

simple_astnode!(Syntax, LocalVarInit, Syntax::LocalVarInit);

impl LocalVarInit {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		let Some(token) = self.0.first_token() else {
			return Err(AstError::Missing);
		};

		match token.kind() {
			Syntax::Ident => Ok(token),
			_ => Err(AstError::Incorrect),
		}
	}

	#[must_use]
	pub fn array_len(&self) -> Option<ArrayLen> {
		let Some(node) = self.0.first_child() else {
			return None;
		};
		ArrayLen::cast(node)
	}

	#[must_use]
	pub fn single_init(&self) -> Option<Expr> {
		let Some(last) = self.0.last_token() else {
			return None;
		};

		if last.kind() == Syntax::BraceR {
			return None;
		}

		let Some(node) = self.0.last_child() else {
			return None;
		};
		Expr::cast(node)
	}

	#[must_use]
	pub fn braced_inits(&self) -> Option<impl Iterator<Item = Expr>> {
		let Some(last) = self.0.last_token() else {
			return None;
		};

		if last.kind() != Syntax::BraceR {
			return None;
		}

		Some(self.0.children().filter_map(Expr::cast))
	}
}

// VarName /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::VarName`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VarName(SyntaxNode);

simple_astnode!(Syntax, VarName, Syntax::VarName);

impl VarName {
	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn ident(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::Ident);
		ret
	}

	pub fn array_lengths(&self) -> impl Iterator<Item = ArrayLen> {
		self.0.children().map(|node| {
			debug_assert_eq!(node.kind(), Syntax::ArrayLen);
			ArrayLen(node)
		})
	}
}

// ArrayLen ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ArrayLen`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArrayLen(SyntaxNode);

simple_astnode!(Syntax, ArrayLen, Syntax::ArrayLen);

impl ArrayLen {
	#[must_use]
	pub fn expr(&self) -> Option<Expr> {
		self.0.first_child().map(|node| Expr::cast(node).unwrap())
	}
}

// DocComment //////////////////////////////////////////////////////////////////

/// Wraps a [`Syntax::DocComment`] token. Provides a convenience function for
/// stripping preceding slashes and surrounding whitespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DocComment(SyntaxToken);

impl DocComment {
	/// Shorthand for `self.text().trim_matches('/').trim()`.
	#[must_use]
	pub fn text_trimmed(&self) -> &str {
		self.0.text().trim_matches('/').trim()
	}
}

impl std::ops::Deref for DocComment {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// An "interface" for any syntax node supporting preceding [zscdoc] comments.
///
/// [zscdoc]: https://gitlab.com/Gutawer/zscdoc
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Documentable(SyntaxNode);

impl Documentable {
	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(&self.0)
	}
}

impl rowan::ast::AstNode for Documentable {
	type Language = Syntax;

	fn can_cast(kind: <Self::Language as Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::ConstDef
				| Syntax::EnumDef
				| Syntax::EnumVariant
				| Syntax::ClassDef
				| Syntax::MixinClassDef
				| Syntax::StructDef
				| Syntax::FieldDecl
				| Syntax::FunctionDecl
				| Syntax::PropertyDef
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		Self::can_cast(node.kind()).then_some(Self(node))
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

// Common AST helper functions /////////////////////////////////////////////////

fn doc_comments(node: &SyntaxNode) -> impl Iterator<Item = DocComment> {
	node.children_with_tokens()
		.take_while(|elem| elem.kind().is_trivia() || elem.kind() == Syntax::DocComment)
		.filter_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syntax::DocComment)
				.map(DocComment)
		})
}

#[cfg(test)]
mod test {
	use crate::zdoom;

	use super::*;

	#[test]
	fn include_path_composition() {
		const SAMPLES: &[&str] = &[
			r##" #include "/doom-ls.zs" "##,
			r##" #include "./" "doom-ls.zs" "##,
		];

		const EXPECTED: &[&str] = &[
			"/home/user/zscript-mod/doom-ls.zs",
			"/home/user/zscript-mod/zscript/subdir/doom-ls.zs",
		];

		for (i, sample) in SAMPLES.iter().copied().enumerate() {
			let ptree = crate::parse(
				sample.trim(),
				zdoom::zscript::parse::include_directive,
				zdoom::lex::Context::ZSCRIPT_LATEST,
			);

			let directive = IncludeDirective::cast(ptree.cursor()).unwrap();

			let incpath = directive
				.include_path(Path::new("/home/user/zscript-mod"), || {
					Path::new("/home/user/zscript-mod/zscript/subdir")
				})
				.unwrap();

			assert_eq!(incpath, Path::new(EXPECTED[i]));
		}
	}
}
