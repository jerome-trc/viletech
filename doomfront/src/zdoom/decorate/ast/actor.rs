use rowan::{ast::AstNode, SyntaxElement};

use crate::{
	simple_astnode,
	zdoom::decorate::{Syntax, SyntaxElem, SyntaxNode, SyntaxToken},
};

use super::{ConstDef, EnumDef, IdentChain};

/// Wraps a node tagged [`Syntax::ActorDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActorDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, ActorDef, Syntax::ActorDef);

impl ActorDef {
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| {
				if elem.kind() == Syntax::Ident {
					Some(elem.into_token().unwrap())
				} else {
					None
				}
			})
			.unwrap()
	}

	#[must_use]
	pub fn base_class(&self) -> Option<SyntaxToken> {
		for node in self.syntax().children() {
			if node.kind() == Syntax::InheritSpec {
				return Some(node.last_token().unwrap());
			}

			if Innard::can_cast(node.kind()) {
				break;
			}
		}

		None
	}

	#[must_use]
	pub fn replaced_class(&self) -> Option<SyntaxToken> {
		for node in self.syntax().children() {
			if node.kind() == Syntax::ReplacesClause {
				return Some(node.last_token().unwrap());
			}

			if Innard::can_cast(node.kind()) {
				break;
			}
		}

		None
	}

	#[must_use]
	pub fn editor_number(&self) -> Option<SyntaxToken> {
		for node in self.syntax().children() {
			if node.kind() == Syntax::EditorNumber {
				return Some(node.last_token().unwrap());
			}

			if Innard::can_cast(node.kind()) {
				break;
			}
		}

		None
	}

	pub fn innards(&self) -> impl Iterator<Item = Innard> {
		self.syntax().children().filter_map(Innard::cast)
	}
}

// Innard //////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Innard {
	ConstDef(ConstDef),
	EnumDef(EnumDef),
	Settings(ActorSettings),
	StatesDef(StatesDef),
	UserVar(UserVar),
}

impl AstNode for Innard {
	type Language = Syntax;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::ConstDef
				| Syntax::EnumDef
				| Syntax::ActorSettings
				| Syntax::StatesDef
				| Syntax::UserVar
		)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::ConstDef => Some(Self::ConstDef(ConstDef(node))),
			Syntax::EnumDef => Some(Self::EnumDef(EnumDef(node))),
			Syntax::ActorSettings => Some(Self::Settings(ActorSettings(node))),
			Syntax::StatesDef => Some(Self::StatesDef(StatesDef(node))),
			Syntax::UserVar => Some(Self::UserVar(UserVar(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			Self::ConstDef(inner) => inner.syntax(),
			Self::EnumDef(inner) => inner.syntax(),
			Self::Settings(inner) => inner.syntax(),
			Self::StatesDef(inner) => inner.syntax(),
			Self::UserVar(inner) => inner.syntax(),
		}
	}
}

// UserVar /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::UserVar`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UserVar(pub(super) SyntaxNode);

simple_astnode!(Syntax, UserVar, Syntax::UserVar);

impl UserVar {
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| {
				let SyntaxElement::Token(token) = elem else {
					return None;
				};
				(token.kind() == Syntax::Ident).then_some(token)
			})
			.unwrap()
	}

	#[must_use]
	pub fn type_spec(&self) -> UserVarType {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| {
				let SyntaxElement::Token(token) = elem else {
					return None;
				};

				match token.kind() {
					Syntax::KwInt => Some(UserVarType::Int),
					Syntax::KwFloat => Some(UserVarType::Float),
					_ => None,
				}
			})
			.unwrap()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UserVarType {
	Int,
	Float,
}

// Settings ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ActorSettings`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActorSettings(SyntaxNode);

simple_astnode!(Syntax, ActorSettings, Syntax::ActorSettings);

// StatesDef ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StatesDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StatesDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, StatesDef, Syntax::StatesDef);

impl StatesDef {
	#[must_use]
	pub fn usage_quals(&self) -> Option<impl Iterator<Item = StateUsage>> {
		self.syntax()
			.first_child()
			.filter(|node| node.kind() == Syntax::StatesUsage)
			.map(|node| {
				node.children_with_tokens().filter_map(|elem| {
					let Some(token) = elem.into_token() else {
						return None;
					};

					if token.text().eq_ignore_ascii_case("actor") {
						Some(StateUsage::Actor(token))
					} else if token.text().eq_ignore_ascii_case("item") {
						Some(StateUsage::Item(token))
					} else if token.text().eq_ignore_ascii_case("overlay") {
						Some(StateUsage::Overlay(token))
					} else if token.text().eq_ignore_ascii_case("weapon") {
						Some(StateUsage::Weapon(token))
					} else {
						None
					}
				})
			})
	}

	pub fn items(&self) -> impl Iterator<Item = StatesItem> {
		self.syntax().children().filter_map(StatesItem::cast)
	}
}

/// All wrapped tokens are always tagged [`Syntax::Ident`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateUsage {
	Actor(SyntaxToken),
	Item(SyntaxToken),
	Overlay(SyntaxToken),
	Weapon(SyntaxToken),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StatesItem {
	State(StateDef),
	Label(StateLabel),
	Flow(SyntaxNode),
}

impl AstNode for StatesItem {
	type Language = Syntax;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::StateDef | Syntax::StateLabel | Syntax::StateFlow
		)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::StateDef => Some(Self::State(StateDef(node))),
			Syntax::StateLabel => Some(Self::Label(StateLabel(node))),
			Syntax::StateFlow => Some(Self::Flow(node)),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			StatesItem::State(inner) => inner.syntax(),
			StatesItem::Label(inner) => inner.syntax(),
			StatesItem::Flow(inner) => inner,
		}
	}
}

impl StatesItem {
	#[must_use]
	pub fn into_flow(self) -> Option<StateFlow> {
		match self {
			StatesItem::Flow(inner) => Some({
				let tok1 = inner.first_token().unwrap();

				match tok1.kind() {
					Syntax::KwGoto => {
						let mut target = None;
						let mut scope = None;
						let mut offset = None;

						for elem in inner.children_with_tokens() {
							if elem.kind() == Syntax::Ident || elem.kind() == Syntax::KwSuper {
								scope = Some(elem.into_token().unwrap());
							} else if elem.kind() == Syntax::GotoOffset {
								offset = elem
									.into_node()
									.unwrap()
									.last_token()
									.unwrap()
									.text()
									.parse::<u64>()
									.ok();
							} else if IdentChain::can_cast(elem.kind()) {
								target = IdentChain::cast(elem.into_node().unwrap());
							}
						}

						StateFlow::Goto {
							target: target.unwrap(),
							offset,
							scope,
						}
					}
					Syntax::KwLoop => StateFlow::Loop,
					Syntax::KwStop => StateFlow::Stop,
					Syntax::KwWait => StateFlow::Wait,
					Syntax::KwFail => StateFlow::Fail,
					_ => unreachable!(),
				}
			}),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_label(self) -> Option<StateLabel> {
		match self {
			StatesItem::Label(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_state(self) -> Option<StateDef> {
		match self {
			StatesItem::State(inner) => Some(inner),
			_ => None,
		}
	}
}

/// Wraps a node tagged [`Syntax::StateDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, StateDef, Syntax::StateDef);

impl StateDef {
	/// The returned token is always tagged [`Syntax::StateSprite`].
	///
	/// Its text content may resemble an identifier or a string literal.
	#[must_use]
	pub fn sprite(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}

	/// The returned token is always tagged [`Syntax::StateFrames`].
	///
	/// Its text content content may resemble an identifier or a string literal.
	#[must_use]
	pub fn frames(&self) -> SyntaxToken {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| {
				(elem.kind() == Syntax::StateFrames).then(|| elem.into_token().unwrap())
			})
			.unwrap()
	}

	/// If the returned value is [`rowan::NodeOrToken::Token`], its contained value
	/// is tagged [`Syntax::IntLit`] (and may have a negative number in it). If it
	/// is [`rowan::NodeOrToken::Node`], its contained value is a [`Syntax::CallExpr`].
	#[must_use]
	pub fn duration(&self) -> SyntaxElem {
		for elem in self.syntax().children_with_tokens() {
			match elem.kind() {
				Syntax::IntLit => return SyntaxElem::Token(elem.into_token().unwrap()),
				Syntax::CallExpr => return SyntaxElem::Node(elem.into_node().unwrap()),
				_ => continue,
			}
		}

		unreachable!()
	}

	pub fn qualifiers(&self) -> impl Iterator<Item = StateQual> {
		self.syntax()
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syntax::KwBright => Some(StateQual::Bright),
				Syntax::KwCanRaise => Some(StateQual::CanRaise),
				Syntax::KwFast => Some(StateQual::Fast),
				Syntax::KwNoDelay => Some(StateQual::NoDelay),
				Syntax::KwSlow => Some(StateQual::Slow),
				Syntax::StateLight => Some(StateQual::Light({
					elem.into_node()
						.unwrap()
						.children_with_tokens()
						.find_map(|e| {
							e.into_token().filter(|tok| {
								matches!(tok.kind(), Syntax::StringLit | Syntax::NameLit)
							})
						})
						.unwrap()
				})),
				Syntax::StateOffset => {
					let mut x = None;
					let mut y = None;

					for e in elem.into_node().unwrap().children_with_tokens() {
						if e.kind() == Syntax::IntLit {
							if x.is_none() {
								x = Some(e.into_token().unwrap());
							} else if y.is_none() {
								y = Some(e.into_token().unwrap());
							} else {
								break;
							}
						}
					}

					Some(StateQual::Offset(x.unwrap(), y.unwrap()))
				}
				_ => None,
			})
	}

	/// The returned token is tagged [`Syntax::NameLit`] or [`Syntax::StringLit`].
	/// If multiple `light` qualifiers are present, the last one is returned.
	#[must_use]
	pub fn light(&self) -> Option<SyntaxToken> {
		for node in self.syntax().children() {
			if node.kind() == Syntax::StateLight {
				return node
					.children_with_tokens()
					.find_map(|elem| {
						(matches!(elem.kind(), Syntax::StringLit | Syntax::NameLit))
							.then(|| elem.into_token())
					})
					.unwrap();
			}
		}

		None
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateQual {
	Bright,
	CanRaise,
	Fast,
	NoDelay,
	Slow,
	/// The contained token is tagged [`Syntax::NameLit`] or [`Syntax::StringLit`].
	Light(SyntaxToken),
	/// Each contained token is tagged [`Syntax::IntLit`].
	Offset(SyntaxToken, SyntaxToken),
}

/// Wraps a node tagged [`Syntax::StateLabel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateLabel(pub(super) SyntaxNode);

simple_astnode!(Syntax, StateLabel, Syntax::StateLabel);

impl StateLabel {
	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateFlow {
	Loop,
	Stop,
	Wait,
	Fail,
	Goto {
		target: IdentChain,
		/// Will be `None` if the integer literal can not fit into a [`u64`].
		offset: Option<u64>,
		/// Tagged either [`Syntax::KwSuper`] or [`Syntax::Ident`].
		scope: Option<SyntaxToken>,
	},
}
