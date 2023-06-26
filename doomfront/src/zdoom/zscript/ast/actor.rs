//! AST nodes for parts of definitions for classes inheriting from `Actor`.

use rowan::ast::AstNode;

use crate::{
	simple_astnode,
	zdoom::{ast::LitToken, decorate::ast::StateUsage},
};

use super::{ArgList, CompoundStat, Expr, IdentChain, Syn, SyntaxNode, SyntaxToken};

// FlagDef /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FlagDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlagDef(pub(super) SyntaxNode);

simple_astnode!(Syn, FlagDef, Syn::FlagDef);

impl FlagDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn backing_field(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.skip_while(|elem| elem.kind() != Syn::Colon)
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// The returned token is always tagged [`Syn::IntLit`].
	#[must_use]
	pub fn bit(&self) -> LitToken<Syn> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::IntLit)
					.map(LitToken::new)
			})
			.unwrap()
	}

	/// All returned tokens are tagged [`Syn::DocComment`].
	pub fn docs(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() == Syn::DocComment)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::DocComment)
			})
	}
}

// PropertyDef /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PropertyDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropertyDef(pub(super) SyntaxNode);

simple_astnode!(Syn, PropertyDef, Syn::PropertyDef);

impl PropertyDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// Yielded tokens are always tagged [`Syn::Ident`].
	pub fn backing_fields(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.skip_while(|elem| elem.kind() != Syn::Colon)
			.filter_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
	}

	/// All yielded tokens are tagged [`Syn::DocComment`].
	pub fn docs(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() == Syn::DocComment)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::DocComment)
			})
	}
}

// DefaultBlock ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::DefaultBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefaultBlock(pub(super) SyntaxNode);

simple_astnode!(Syn, DefaultBlock, Syn::DefaultBlock);

impl DefaultBlock {
	pub fn innards(&self) -> impl Iterator<Item = DefaultInnard> {
		self.0.children().filter_map(|node| match node.kind() {
			Syn::FlagSetting => Some(DefaultInnard::FlagSetting(FlagSetting(node))),
			Syn::PropertySetting => Some(DefaultInnard::PropertySetting(PropertySetting(node))),
			_ => None,
		})
	}
}

// DefaultInnard ///////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DefaultInnard {
	FlagSetting(FlagSetting),
	PropertySetting(PropertySetting),
}

impl AstNode for DefaultInnard {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::FlagSetting | Syn::PropertySetting)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::FlagSetting => Some(Self::FlagSetting(FlagSetting(node))),
			Syn::PropertySetting => Some(Self::PropertySetting(PropertySetting(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			DefaultInnard::FlagSetting(inner) => inner.syntax(),
			DefaultInnard::PropertySetting(inner) => inner.syntax(),
		}
	}
}

// FlagSetting /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FlagSetting`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlagSetting(pub(super) SyntaxNode);

simple_astnode!(Syn, FlagSetting, Syn::FlagSetting);

impl FlagSetting {
	#[must_use]
	pub fn is_adding(&self) -> bool {
		self.0.first_token().unwrap().kind() == Syn::Plus
	}

	#[must_use]
	pub fn is_removing(&self) -> bool {
		self.0.first_token().unwrap().kind() == Syn::Minus
	}

	#[must_use]
	pub fn name(&self) -> IdentChain {
		IdentChain::cast(self.syntax().last_child().unwrap()).unwrap()
	}
}

// PropertySetting /////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PropertySetting`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropertySetting(pub(super) SyntaxNode);

simple_astnode!(Syn, PropertySetting, Syn::PropertySetting);

impl PropertySetting {
	#[must_use]
	pub fn name(&self) -> IdentChain {
		IdentChain::cast(self.0.first_child().unwrap()).unwrap()
	}

	/// Mind that the returned iterator may yield no items.
	pub fn exprs(&self) -> impl Iterator<Item = Expr> {
		self.0.children().skip(1).filter_map(Expr::cast)
	}
}

// StatesBlock /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StatesBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StatesBlock(pub(super) SyntaxNode);

simple_astnode!(Syn, StatesBlock, Syn::StatesBlock);

impl StatesBlock {
	#[must_use]
	pub fn usage_quals(&self) -> Option<impl Iterator<Item = StateUsage>> {
		self.0
			.first_child()
			.filter(|node| node.kind() == Syn::StatesUsage)
			.map(|node| {
				node.children_with_tokens().filter_map(|elem| {
					let Some(token) = elem.into_token() else { return None; };

					if token.text().eq_ignore_ascii_case("actor") {
						Some(StateUsage::Actor)
					} else if token.text().eq_ignore_ascii_case("item") {
						Some(StateUsage::Item)
					} else if token.text().eq_ignore_ascii_case("overlay") {
						Some(StateUsage::Overlay)
					} else if token.text().eq_ignore_ascii_case("weapon") {
						Some(StateUsage::Weapon)
					} else {
						None
					}
				})
			})
	}

	pub fn innards(&self) -> impl Iterator<Item = StatesInnard> {
		self.0.children().filter_map(|node| match node.kind() {
			Syn::StateDef => Some(StatesInnard::State(StateDef(node))),
			Syn::StateFlow => Some(StatesInnard::Flow(StateFlow(node))),
			Syn::StateLabel => Some(StatesInnard::Label(StateLabel(node))),
			_ => None,
		})
	}
}

/// Wraps a node tagged [`Syn::ActionQual`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionQual(pub(super) SyntaxNode);

simple_astnode!(Syn, ActionQual, Syn::ActionQual);

// StatesInnard ////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StatesInnard {
	Flow(StateFlow),
	Label(StateLabel),
	State(StateDef),
}

// StateLabel //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StateLabel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateLabel(SyntaxNode);

simple_astnode!(Syn, StateLabel, Syn::StateLabel);

impl StateLabel {
	#[must_use]
	pub fn name(&self) -> IdentChain {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::IdentChain);
		IdentChain(ret)
	}
}

// StateFlow ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StateFlow`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateFlow(SyntaxNode);

simple_astnode!(Syn, StateFlow, Syn::StateFlow);

impl StateFlow {
	#[must_use]
	pub fn kind(&self) -> StateFlowKind {
		let elem = self.0.first_child_or_token().unwrap();

		match elem.kind() {
			Syn::KwGoto => {
				let node = elem.into_node().unwrap();
				let scope = node.children_with_tokens().find_map(|elem| {
					elem.into_token()
						.filter(|token| matches!(token.kind(), Syn::Ident | Syn::KwSuper))
				});
				let name = node.children().find_map(IdentChain::cast).unwrap();
				let offset = node.children_with_tokens().find_map(|elem| {
					elem.into_token()
						.filter(|token| token.kind() == Syn::IntLit)
						.map(LitToken::new)
				});

				StateFlowKind::Goto {
					scope,
					name,
					offset,
				}
			}
			Syn::KwFail => StateFlowKind::Fail(elem.into_token().unwrap()),
			Syn::KwLoop => StateFlowKind::Loop(elem.into_token().unwrap()),
			Syn::KwStop => StateFlowKind::Stop(elem.into_token().unwrap()),
			Syn::KwWait => StateFlowKind::Wait(elem.into_token().unwrap()),
			_ => unreachable!(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateFlowKind {
	Fail(SyntaxToken),
	Goto {
		/// Always tagged either [`Syn::Ident`] or [`Syn::KwSuper`].
		scope: Option<SyntaxToken>,
		name: IdentChain,
		/// Always tagged [`Syn::IntLit`] if present.
		offset: Option<LitToken<Syn>>,
	},
	Loop(SyntaxToken),
	Stop(SyntaxToken),
	Wait(SyntaxToken),
}

// StateDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StateDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateDef(SyntaxNode);

simple_astnode!(Syn, StateDef, Syn::StateDef);

impl StateDef {
	#[must_use]
	pub fn sprite(&self) -> StateSprite {
		let token = self.0.first_token().unwrap();
		debug_assert_eq!(token.kind(), Syn::StateSprite);
		let text = token.text();

		if text.starts_with('"') && text.ends_with('"') {
			StateSprite::Quoted(LitToken::new(token))
		} else {
			StateSprite::Unquoted(token)
		}
	}

	#[must_use]
	pub fn frames(&self) -> StateFrames {
		let token = self
			.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::StateFrames)
			})
			.unwrap();

		let text = token.text();

		if text.starts_with('"') && text.ends_with('"') {
			StateFrames::Quoted(LitToken::new(token))
		} else {
			StateFrames::Unquoted(token)
		}
	}

	pub fn quals(&self) -> impl Iterator<Item = StateQual> {
		let quals = self
			.0
			.children()
			.find(|node| node.kind() == Syn::StateQuals)
			.unwrap();

		quals
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syn::KwBright => Some(StateQual::Bright(elem.into_token().unwrap())),
				Syn::KwCanRaise => Some(StateQual::CanRaise(elem.into_token().unwrap())),
				Syn::KwFast => Some(StateQual::Fast(elem.into_token().unwrap())),
				Syn::StateLight => Some(StateQual::Light(StateLight(elem.into_node().unwrap()))),
				Syn::KwNoDelay => Some(StateQual::NoDelay(elem.into_token().unwrap())),
				Syn::StateOffset => Some(StateQual::Offset(StateOffset(elem.into_node().unwrap()))),
				Syn::KwSlow => Some(StateQual::Slow(elem.into_token().unwrap())),
				_ => None,
			})
	}

	#[must_use]
	pub fn duration(&self) -> Expr {
		self.0.children().find_map(Expr::cast).unwrap()
	}

	#[must_use]
	pub fn action(&self) -> Option<ActionFunction> {
		ActionFunction::cast(self.0.last_child().unwrap())
	}
}

// StateQual ///////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateQual {
	Bright(SyntaxToken),
	CanRaise(SyntaxToken),
	Fast(SyntaxToken),
	Light(StateLight),
	NoDelay(SyntaxToken),
	Offset(StateOffset),
	Slow(SyntaxToken),
}

/// Wraps a node tagged [`Syn::StateLight`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateLight(SyntaxNode);

simple_astnode!(Syn, StateLight, Syn::StateLight);

impl StateLight {
	/// Each yielded token is tagged [`Syn::StringLit`] or [`Syn::NameLit`].
	pub fn lights(&self) -> impl Iterator<Item = LitToken<Syn>> {
		self.0.children_with_tokens().filter_map(|elem| {
			elem.into_token()
				.filter(|token| matches!(token.kind(), Syn::NameLit | Syn::StringLit))
				.map(LitToken::new)
		})
	}
}

/// Wraps a node tagged [`Syn::StateOffset`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateOffset(SyntaxNode);

simple_astnode!(Syn, StateOffset, Syn::StateOffset);

impl StateOffset {
	#[must_use]
	pub fn x(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn y(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}
}

// StateSprite /////////////////////////////////////////////////////////////////

/// Each variant wraps a token tagged [`Syn::StateSprite`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateSprite {
	Quoted(LitToken<Syn>),
	Unquoted(SyntaxToken),
}

impl StateSprite {
	#[must_use]
	pub fn hold(&self) -> bool {
		match self {
			Self::Quoted(lit) => lit.string().unwrap() == "----",
			Self::Unquoted(token) => token.text() == "----",
		}
	}

	#[must_use]
	pub fn hold_selective(&self) -> bool {
		match self {
			Self::Quoted(lit) => lit.string().unwrap() == "####",
			Self::Unquoted(token) => token.text() == "####",
		}
	}
}

// StateFrames /////////////////////////////////////////////////////////////////

/// Each variant wraps a token tagged [`Syn::StateFrames`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateFrames {
	/// Use [`LitToken::<zscript::Syn>::string`] to get the un-delimited content.
	Quoted(LitToken<Syn>),
	/// The text of the token within comprises an arbitrary combination of
	/// ASCII letters, `[`, `]`, and `\`.
	Unquoted(SyntaxToken),
}

impl StateFrames {
	/// Returns `true` if the token's text is either `#` or `"#"`.
	#[must_use]
	pub fn hold_selective(&self) -> bool {
		match self {
			Self::Quoted(token) => token.string().unwrap() == "#",
			Self::Unquoted(token) => token.text() == "#",
		}
	}
}

// ActionFunction //////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ActionFunction`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionFunction(SyntaxNode);

simple_astnode!(Syn, ActionFunction, Syn::ActionFunction);

impl ActionFunction {
	/// Token `0` is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn into_call(&self) -> Option<(SyntaxToken, Option<ArgList>)> {
		let ident = match self
			.0
			.first_token()
			.filter(|token| token.kind() == Syn::Ident)
		{
			Some(t) => t,
			None => return None,
		};

		let args = match self.0.last_child() {
			Some(n) => ArgList::cast(n),
			None => None,
		};

		Some((ident, args))
	}

	#[must_use]
	pub fn into_anon(&self) -> Option<CompoundStat> {
		match self.0.first_child() {
			Some(node) => CompoundStat::cast(node),
			None => None,
		}
	}
}