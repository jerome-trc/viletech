//! Functions for parsing Lith source code.

mod item;

use doomfront::{
	rowan::{GreenNode, GreenToken, SyntaxKind},
	GreenElement,
};
use logos::Logos;

use crate::Syn;

pub type Error = peg::error::ParseError<usize>;

impl doomfront::LangExt for Syn {
	type Token = Self;
	const EOF: Self::Token = Self::Eof;
	const ERR_NODE: Self::Kind = Self::Error;
}

pub fn file(source: &str) -> Result<GreenNode, Error> {
	let lexer = Syn::lexer_with_extras(source, crate::Version::new(0, 0, 0))
		.spanned()
		.map(|(result, span)| match result {
			Ok(t) => (t, span.start, span.end),
			Err(t) => (t, span.start, span.end),
		});

	let tokens = lexer.collect::<Vec<_>>();
	experimental::file(&tokens, source)
}

peg::parser! {
	pub grammar experimental(source: &'input str) for [(Syn, usize, usize)] {
		pub(super) rule file() -> GreenNode
			= items:(trivia() / annotation() / stat_import())* ![_]
		{
			GreenNode::new(Syn::FileRoot.into(), items)
		}

		pub(super) rule annotation() -> GreenElement
			=	pound:token(Syn::Pound)
				bang:token(Syn::Bang)?
				bl:token(Syn::BracketL)
				t0:trivia()*
				id:ident_chain()
				t1:trivia()*
				br:token(Syn::BracketR)
		{
			coalesce_node((pound, bang, bl, t0, id, t1, br), Syn::Annotation).into()
		}

		// Items ///////////////////////////////////////////////////////////////

		rule func_decl() -> GreenNode
			= 	annos:annotation()*
				func:token(Syn::KwFunc)
				t0:trivia()*
				id:ident()
				t1:trivia()*
				params:param_list()
				t2:trivia()*
				ret:type_spec()?
				t3:trivia()*
				body:(func_body() / elem(Syn::Semicolon))
		{
			coalesce_node((annos, func, t0, id, t1, params, t2, ret, t3, body), Syn::FuncDecl)
		}

		rule param_list() -> GreenNode
			= 	pl:token(Syn::ParenL)
				t0:trivia()*
				param0:param()?
				params:param_successive()*
				comma:token(Syn::Comma)?
				t1:trivia()*
				pr:token(Syn::ParenR)
		{
			coalesce_node((pl, t0, param0, params, comma, t1, pr), Syn::ParamList)
		}

		rule param() -> GreenNode
			= id:ident() triv:trivia()* ty:type_spec()
		{
			coalesce_node((id, triv, ty), Syn::Parameter)
		}

		rule param_successive()
			-> (Vec<GreenElement>, GreenToken, Vec<GreenElement>, GreenNode)
			= t0:trivia()* comma:token(Syn::Comma) t1:trivia()* p:param()
		{
			(t0, comma, t1, p)
		}

		rule func_body() -> GreenElement
			=	bl:token(Syn::BraceL)
				stats:(statement() / trivia())*
				br:token(Syn::BraceR)
		{
			coalesce_node((bl, stats, br), Syn::FuncBody).into()
		}

		// Statements //////////////////////////////////////////////////////////

		rule statement() -> GreenElement
			= (
				stat_break() /
				stat_continue() /
				stat_expr() /
				stat_import() /
				stat_return()
			)

		rule stat_break() -> GreenElement
			=	br:token(Syn::KwBreak)
				t0:trivia()*
				label:block_label()?
				t1:trivia()*
				term:token(Syn::Semicolon)
		{
			coalesce_node((br, t0, label, t1, term), Syn::BreakStat).into()
		}

		rule stat_continue() -> GreenElement
			= 	cont:token(Syn::KwContinue)
				t0:trivia()*
				label:block_label()?
				t1:trivia()*
				term:token(Syn::Semicolon)
		{
			coalesce_node((cont, t0, label, t1, term), Syn::ContinueStat).into()
		}

		rule stat_expr() -> GreenElement
			= e:expr() triv:trivia()* term:token(Syn::Semicolon)
		{
			coalesce_node((e, triv, term), Syn::ExprStat).into()
		}

		rule stat_import() -> GreenElement
			= 	import:token(Syn::KwImport)
				t0:trivia()*
				path:token(Syn::StringLit)
				t1:trivia()*
				colon:token(Syn::Colon)
				t2:trivia()*
				imps:(import_list() / import_entry() / import_all())
				t3:trivia()*
				term:token(Syn::Semicolon)
		{
			coalesce_node(
				(import, t0, path, t1, colon, t2, imps, t3, term),
				Syn::ImportStat
			).into()
		}

		rule import_list() -> GreenNode
			=	bl:token(Syn::BraceL)
				t0:trivia()*
				i0:import_entry()
				successive:import_entry_successive()*
				t1:trivia()*
				br:token(Syn::BraceR)
		{
			coalesce_node((bl, t0, i0, successive, t1, br), Syn::ImportList)
		}

		rule import_entry_successive() -> Vec<GreenElement>
			=	t0:trivia()*
				comma:token(Syn::Comma)
				t1:trivia()*
				entry:import_entry()
		{
			coalesce_vec((t0, comma, t1, entry))
		}

		rule import_entry() -> GreenNode
			=	name:ident()
				rename:import_rename()?
		{
			coalesce_node((name, rename), Syn::ImportEntry)
		}

		rule import_rename() -> Vec<GreenElement>
			=	t0:trivia()*
				arrow:token(Syn::ThickArrow)
				t1:trivia()*
				rename:ident()
		{
			coalesce_vec((t0, arrow, t1, rename))
		}

		rule import_all() -> GreenNode
			=	aster:token(Syn::Asterisk)
				rename:import_rename()?
		{
			coalesce_node((aster, rename), Syn::ImportEntry)
		}

		rule stat_return() -> GreenElement
			= 	ret:token(Syn::KwReturn)
				t0:trivia()*
				e:expr()?
				t1:trivia()*
				term:token(Syn::Semicolon)
		{
			coalesce_node((ret, t0, e, t1, term), Syn::ReturnStat).into()
		}

		// Expressions /////////////////////////////////////////////////////////

		pub(super) rule expr() -> GreenNode = precedence! {
			// Assignment //////////////////////////////////////////////////////
			lhs:@ t0:trivia()* eq:token(Syn::Eq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, eq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* angl2eq:token(Syn::AngleL2Eq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, angl2eq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* angr2eq:token(Syn::AngleR2Eq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, angr2eq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* pleq:token(Syn::PlusEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, pleq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* mineq:token(Syn::MinusEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, mineq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* astereq:token(Syn::AsteriskEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, astereq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* slasheq:token(Syn::SlashEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, slasheq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* pcteq:token(Syn::PercentEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, pcteq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* ampeq:token(Syn::AmpersandEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, ampeq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* pipeeq:token(Syn::PipeEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, pipeeq, t1, rhs), Syn::BinExpr)
			}
			lhs:@ t0:trivia()* careteq:token(Syn::CaretEq) t1:trivia()* rhs:(@) {
				coalesce_node((lhs, t0, careteq, t1, rhs), Syn::BinExpr)
			}
			--
			// Binary, logical comparison //////////////////////////////////////
			lhs:(@) t0:trivia()* pipe2:token(Syn::Pipe2) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, pipe2, t1, rhs), Syn::BinExpr)
			}
			--
			lhs:(@) t0:trivia()* amp2:token(Syn::Ampersand2) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, amp2, t1, rhs), Syn::BinExpr)
			}
			--
			lhs:(@) t0:trivia()* eq2:token(Syn::Eq2) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, eq2, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* be:token(Syn::BangEq) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, be, t1, rhs), Syn::BinExpr)
			}
			// Binary, ordered comparison //////////////////////////////////////
			lhs:(@) t0:trivia()* angreq:token(Syn::AngleREq) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, angreq, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* angleq:token(Syn::AngleLEq) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, angleq, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* angr:token(Syn::AngleR) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, angr, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* angl:token(Syn::AngleL) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, angl, t1, rhs), Syn::BinExpr)
			}
			--
			// Binary, bitwise /////////////////////////////////////////////////
			lhs:(@) t0:trivia()* pipe:token(Syn::Pipe) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, pipe, t1, rhs), Syn::BinExpr)
			}
			--
			lhs:(@) t0:trivia()* caret:token(Syn::Caret) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, caret, t1, rhs), Syn::BinExpr)
			}
			--
			lhs:(@) t0:trivia()* amp:token(Syn::Ampersand) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, amp, t1, rhs), Syn::BinExpr)
			}
			--
			lhs:(@) t0:trivia()* angl2:token(Syn::AngleL2) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, angl2, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* angr2:token(Syn::AngleR2) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, angr2, t1, rhs), Syn::BinExpr)
			}
			--
			// Binary, arithmetic //////////////////////////////////////////////
			lhs:(@) t0:trivia()* plus:token(Syn::Plus) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, plus, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* minus:token(Syn::Minus) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, minus, t1, rhs), Syn::BinExpr)
			}
			--
			lhs:(@) t0:trivia()* asterisk:token(Syn::Asterisk) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, asterisk, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* slash:token(Syn::Slash) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, slash, t1, rhs), Syn::BinExpr)
			}
			lhs:(@) t0:trivia()* pct:token(Syn::Percent) t1:trivia()* rhs:@ {
				coalesce_node((lhs, t0, pct, t1, rhs), Syn::BinExpr)
			}
			--
			// Unary, prefix ///////////////////////////////////////////////////
			minus:token(Syn::Minus) t:trivia()* operand:@ {
				coalesce_node((minus, t, operand), Syn::PrefixExpr)
			}
			bang:token(Syn::Bang) t:trivia()* operand:@ {
				coalesce_node((bang, t, operand), Syn::PrefixExpr)
			}
			tilde:token(Syn::Tilde) t:trivia()* operand:@  {
				coalesce_node((tilde, t, operand), Syn::PrefixExpr)
			}
			--
			// Non-atomic primary (call, indexing) /////////////////////////////
			lhs:(@) triv:trivia()* args:arg_list() {
				coalesce_node((lhs, triv, args), Syn::CallExpr)
			}
			lhs:(@)
			t0:trivia()*
			bl:token(Syn::BracketL)
			t1:trivia()*
			index:expr()
			t2:trivia()*
			br:token(Syn::BracketR) {
				coalesce_node((lhs, t0, bl, t1, index, t2, br), Syn::IndexExpr)
			}
			--
			// Field access ////////////////////////////////////////////////////
			lhs:@ t0:trivia()* dot:token(Syn::Dot) t1:trivia()* id:ident() {
				coalesce_node((lhs, t0, dot, t1, id), Syn::FieldExpr)
			}
			--
			// Method call /////////////////////////////////////////////////////
			lhs:@ t0:trivia()* dot:token(Syn::Dot) t1:trivia()* id:ident() args:arg_list() {
				coalesce_node((lhs, t0, dot, t1, id, args), Syn::MethodExpr)
			}
			--
			// Atoms ///////////////////////////////////////////////////////////
			pl:token(Syn::ParenL)
			t0:trivia()*
			inner:expr()
			t1:trivia()*
			pr:token(Syn::ParenR) {
				coalesce_node((pl, t0, inner, t1, pr), Syn::GroupExpr)
			}
			id:ident() {
				GreenNode::new(Syn::IdentExpr.into(), [id.into()])
			}
			lit:(
				token(Syn::FalseLit) /
				token(Syn::FloatLit) /
				token(Syn::IntLit) /
				token(Syn::StringLit) /
				token(Syn::TrueLit)
			) {
				GreenNode::new(Syn::Literal.into(), [lit.into()])
			}
		}

		rule arg_list() -> GreenNode
			=	pl:token(Syn::ParenL)
				t0:trivia()*
				arg0:expr()?
				args:argument()*
				comma:token(Syn::Comma)?
				t1:trivia()*
				pr:token(Syn::ParenR)
		{
			coalesce_node((pl, t0, arg0, args, comma, t1, pr), Syn::ArgList)
		}

		rule argument()
			-> (Vec<GreenElement>, GreenToken, Vec<GreenElement>, GreenNode)
			= 	t0:trivia()*
				comma:token(Syn::Comma)
				t1:trivia()*
				e:expr()
		{
			(t0, comma, t1, e)
		}

		// Common //////////////////////////////////////////////////////////////

		rule block_label() -> GreenNode
			=	c0:token(Syn::Colon2)
				t0:trivia()*
				id:ident()
				t1:trivia()*
				c1:token(Syn::Colon2)
		{
			coalesce_node((c0, t0, id, t1, c1), Syn::BlockLabel)
		}

		rule ident() -> GreenToken = token(Syn::Ident) / token(Syn::IdentRaw)

		rule ident_chain() -> GreenNode
			=	id0:ident()
				successive:ident_chain_part()*
		{
			coalesce_node((id0, successive), Syn::IdentChain)
		}

		rule ident_chain_part() -> Vec<GreenElement>
			=	id:ident()
				t0:trivia()*
				dot:token(Syn::Dot)
				t1:trivia()*
		{
			coalesce_vec((id, t0, dot, t1))
		}

		rule trivia() -> GreenElement
			= t:(token(Syn::Whitespace) / token(Syn::Comment))
		{
			GreenElement::from(t)
		}

		rule type_spec() -> GreenNode
			=	c:token(Syn::Colon)
				t0:trivia()*
				e:expr()
				t1:trivia()*
		{
			coalesce_node((c, t0, e, t1), Syn::TypeSpec)
		}

		// Helpers /////////////////////////////////////////////////////////////

		rule token(syn: Syn) -> GreenToken = t:[_] {?
			if t.0 == syn {
				Ok(GreenToken::new(t.0.into(), &source[t.1..t.2]))
			} else {
				Err(syn.pretty())
			}
		}

		rule elem(syn: Syn) -> GreenElement = t:[_] {?
			if t.0 == syn {
				Ok(GreenToken::new(t.0.into(), &source[t.1..t.2]).into())
			} else {
				Err(syn.pretty())
			}
		}
	}
}

// Parsing helper trait ////////////////////////////////////////////////////////

#[must_use]
fn coalesce_node<C: Coalesce>(input: C, syn: impl Into<SyntaxKind>) -> GreenNode {
	let mut elems = vec![];
	input.coalesce(&mut elems);
	GreenNode::new(syn.into(), elems)
}

#[must_use]
fn coalesce_vec<C: Coalesce>(input: C) -> Vec<GreenElement> {
	let mut ret = vec![];
	input.coalesce(&mut ret);
	ret
}

trait Coalesce: 'static {
	fn coalesce(self, container: &mut Vec<GreenElement>);
}

impl Coalesce for GreenNode {
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		container.push(self.into());
	}
}

impl Coalesce for GreenToken {
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		container.push(self.into());
	}
}

impl Coalesce for GreenElement {
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		container.push(self);
	}
}

impl<T> Coalesce for Option<T>
where
	T: Coalesce,
{
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		if let Some(input) = self {
			input.coalesce(container);
		}
	}
}

impl<T> Coalesce for Vec<T>
where
	T: Coalesce,
{
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		for input in self {
			input.coalesce(container);
		}
	}
}

#[impl_trait_for_tuples::impl_for_tuples(1, 10)]
impl Coalesce for Tuple {
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		let _ = for_tuples!((#(Tuple::coalesce(self.Tuple, container)),*));
	}
}
