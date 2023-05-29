use rowan::ast::AstNode;

use crate::{
	simple_astnode,
	zdoom::decorate::{Syn, SyntaxElem, SyntaxNode, SyntaxToken},
};

use super::{ConstDef, EnumDef, IdentChain};

/// Wraps a node tagged [`Syn::ActorDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct ActorDef(pub(super) SyntaxNode);

simple_astnode!(Syn, ActorDef, Syn::ActorDef);

impl ActorDef {
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| {
				if elem.kind() == Syn::Ident {
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
			if node.kind() == Syn::InheritSpec {
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
			if node.kind() == Syn::ReplacesClause {
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
			if node.kind() == Syn::EditorNumber {
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
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum Innard {
	ConstDef(ConstDef),
	EnumDef(EnumDef),
	FlagSetting(FlagSetting),
	PropertySettings(PropertySettings),
	StatesDef(StatesDef),
}

impl AstNode for Innard {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::ConstDef
				| Syn::EnumDef | Syn::FlagSetting
				| Syn::PropertySettings
				| Syn::StatesDef
		)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ConstDef => Some(Self::ConstDef(ConstDef(node))),
			Syn::EnumDef => Some(Self::EnumDef(EnumDef(node))),
			Syn::FlagSetting => Some(Self::FlagSetting(FlagSetting(node))),
			Syn::PropertySettings => Some(Self::PropertySettings(PropertySettings(node))),
			Syn::StatesDef => Some(Self::StatesDef(StatesDef(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			Self::ConstDef(inner) => inner.syntax(),
			Self::EnumDef(inner) => inner.syntax(),
			Self::FlagSetting(inner) => inner.syntax(),
			Self::PropertySettings(inner) => inner.syntax(),
			Self::StatesDef(inner) => inner.syntax(),
		}
	}
}

impl Innard {
	#[must_use]
	pub fn into_constdef(self) -> Option<ConstDef> {
		match self {
			Self::ConstDef(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_enumdef(self) -> Option<EnumDef> {
		match self {
			Self::EnumDef(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_flagsetting(self) -> Option<FlagSetting> {
		match self {
			Self::FlagSetting(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_propsettings(self) -> Option<PropertySettings> {
		match self {
			Self::PropertySettings(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_statesdef(self) -> Option<StatesDef> {
		match self {
			Self::StatesDef(inner) => Some(inner),
			_ => None,
		}
	}
}

// FlagSetting /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FlagSetting`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct FlagSetting(pub(super) SyntaxNode);

simple_astnode!(Syn, FlagSetting, Syn::FlagSetting);

impl FlagSetting {
	#[must_use]
	pub fn name(&self) -> IdentChain {
		IdentChain::cast(self.syntax().last_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn is_adding(&self) -> bool {
		self.syntax().first_token().unwrap().text() == "+"
	}

	#[must_use]
	pub fn is_removing(&self) -> bool {
		self.syntax().first_token().unwrap().text() == "-"
	}
}

// PropertySetting /////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PropertySettings`].
///
/// DECORATE has no established grammar. The reference implementation parses a
/// single valid property setting by context-sensitively checking every token it
/// reads. This is out of scope for DoomFront, so this node may encompass multiple
/// valid property settings, and the client must decompose them accordingly.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct PropertySettings(pub(super) SyntaxNode);

simple_astnode!(Syn, PropertySettings, Syn::PropertySettings);

// StatesDef ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StatesDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct StatesDef(pub(super) SyntaxNode);

simple_astnode!(Syn, StatesDef, Syn::StatesDef);

impl StatesDef {
	#[must_use]
	pub fn usage_quals(&self) -> Option<impl Iterator<Item = StateUsage>> {
		self.syntax()
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

	pub fn items(&self) -> impl Iterator<Item = StatesItem> {
		self.syntax().children().filter_map(StatesItem::cast)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
pub enum StateUsage {
	Actor,
	Item,
	Overlay,
	Weapon,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum StatesItem {
	State(StateDef),
	Label(StateLabel),
	Flow(SyntaxNode),
}

impl AstNode for StatesItem {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::StateDef | Syn::StateLabel | Syn::StateFlow)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::StateDef => Some(Self::State(StateDef(node))),
			Syn::StateLabel => Some(Self::Label(StateLabel(node))),
			Syn::StateFlow => Some(Self::Flow(node)),
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
					Syn::KwGoto => {
						let mut target = None;
						let mut scope = None;
						let mut offset = None;

						for elem in inner.children_with_tokens() {
							if elem.kind() == Syn::Ident || elem.kind() == Syn::KwSuper {
								scope = Some(elem.into_token().unwrap());
							} else if elem.kind() == Syn::GotoOffset {
								offset = elem
									.into_node()
									.unwrap()
									.last_token()
									.unwrap()
									.text()
									.parse::<u64>()
									.ok();
							} else if IdentChain::can_cast(elem.kind()) {
								target = Some(IdentChain::cast(elem.into_node().unwrap())).unwrap();
							}
						}

						StateFlow::Goto {
							target: target.unwrap(),
							offset,
							scope,
						}
					}
					Syn::KwLoop => StateFlow::Loop,
					Syn::KwStop => StateFlow::Stop,
					Syn::KwWait => StateFlow::Wait,
					Syn::KwFail => StateFlow::Fail,
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

/// Wraps a node tagged [`Syn::StateDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct StateDef(pub(super) SyntaxNode);

simple_astnode!(Syn, StateDef, Syn::StateDef);

impl StateDef {
	/// The returned token is always tagged [`Syn::StateSprite`].
	///
	/// Its text content may resemble an identifier or a string literal.
	#[must_use]
	pub fn sprite(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}

	/// The returned token is always tagged [`Syn::StateFrames`].
	///
	/// Its text content content may resemble an identifier or a string literal.
	#[must_use]
	pub fn frames(&self) -> SyntaxToken {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| (elem.kind() == Syn::StateFrames).then(|| elem.into_token().unwrap()))
			.unwrap()
	}

	/// If the returned value is [`rowan::NodeOrToken::Token`], its contained value
	/// is tagged [`Syn::LitInt`] (and may have a negative number in it). If it
	/// is [`rowan::NodeOrToken::Node`], its contained value is a [`Syn::ExprCall`].
	#[must_use]
	pub fn duration(&self) -> SyntaxElem {
		for elem in self.syntax().children_with_tokens() {
			match elem.kind() {
				Syn::IntLit => return SyntaxElem::Token(elem.into_token().unwrap()),
				Syn::CallExpr => return SyntaxElem::Node(elem.into_node().unwrap()),
				_ => continue,
			}
		}

		unreachable!()
	}

	pub fn qualifiers(&self) -> impl Iterator<Item = StateQual> {
		self.syntax()
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syn::KwBright => Some(StateQual::Bright),
				Syn::KwCanRaise => Some(StateQual::CanRaise),
				Syn::KwFast => Some(StateQual::Fast),
				Syn::KwNoDelay => Some(StateQual::NoDelay),
				Syn::KwSlow => Some(StateQual::Slow),
				Syn::StateLight => Some(StateQual::Light({
					elem.into_node()
						.unwrap()
						.children_with_tokens()
						.find_map(|e| {
							e.into_token()
								.filter(|tok| matches!(tok.kind(), Syn::StringLit | Syn::NameLit))
						})
						.unwrap()
				})),
				Syn::StateOffset => {
					let mut x = None;
					let mut y = None;

					for e in elem.into_node().unwrap().children_with_tokens() {
						if e.kind() == Syn::IntLit {
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

	/// The returned token is tagged [`Syn::LitName`] or [`Syn::LitString`].
	/// If multiple `light` qualifiers are present, the last one is returned.
	#[must_use]
	pub fn light(&self) -> Option<SyntaxToken> {
		for node in self.syntax().children() {
			if node.kind() == Syn::StateLight {
				return node
					.children_with_tokens()
					.find_map(|elem| {
						(matches!(elem.kind(), Syn::StringLit | Syn::NameLit))
							.then(|| elem.into_token())
					})
					.unwrap();
			}
		}

		None
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum StateQual {
	Bright,
	CanRaise,
	Fast,
	NoDelay,
	Slow,
	/// The contained token is tagged [`Syn::LitName`] or [`Syn::LitString`].
	Light(SyntaxToken),
	/// Each contained token is tagged [`Syn::LitInt`].
	Offset(SyntaxToken, SyntaxToken),
}

/// Wraps a node tagged [`Syn::StateLabel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct StateLabel(pub(super) SyntaxNode);

simple_astnode!(Syn, StateLabel, Syn::StateLabel);

impl StateLabel {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum StateFlow {
	Loop,
	Stop,
	Wait,
	Fail,
	Goto {
		target: IdentChain,
		/// Will be `None` if the integer literal can not fit into a [`u64`].
		offset: Option<u64>,
		/// Tagged either [`Syn::KwSuper`] or [`Syn::Ident`].
		scope: Option<SyntaxToken>,
	},
}
