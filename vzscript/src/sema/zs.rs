//! Semantic mid-section for ZScript.

mod ceval;
mod class;

use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, SyntaxNode},
};
use triomphe::Arc;

use crate::{
	compile::{
		symbol::{DefKind, DefStatus, Definition, Symbol, Undefined},
		Scope,
	},
	issue::{self, Issue},
	vir,
};

use super::SemaContext;

#[must_use]
pub(super) fn define(ctx: &SemaContext, root: &SyntaxNode, symbol: &Symbol) -> DefStatus {
	let mut ret = DefStatus::Err;

	symbol.def.rcu(move |undef| {
		let ast = root
			.covering_element(ctx.location.span)
			.into_node()
			.unwrap();

		let DefKind::None { kind, qname } = &undef.kind else {
			unreachable!()
		};

		let success = match kind {
			Undefined::Class => {
				match class::define(
					ctx,
					qname.clone(),
					undef.scope.clone(),
					ast::ClassDef::cast(ast).unwrap(),
				) {
					Ok(arc) => arc,
					Err(()) => return undef.clone(),
				}
			}
			Undefined::Value => {
				match define_constant(ctx, undef, ast::ConstDef::cast(ast).unwrap()) {
					Ok(arc) => arc,
					Err(()) => return undef.clone(),
				}
			}
			Undefined::Enum
			| Undefined::Field
			| Undefined::FlagDef
			| Undefined::Function
			| Undefined::Property
			| Undefined::Mixin
			| Undefined::Struct
			| Undefined::Union => {
				ctx.raise(Issue::new(
					ctx.resolve_path(ctx.location),
					TextRange::new(ctx.location.span.start(), ctx.location.short_end),
					"not yet implemented".to_string(),
					issue::Level::Error(issue::Error::Internal),
				));

				return undef.clone();
			}
		};

		ret = DefStatus::Ok;
		success
	});

	ret
}

fn define_constant(
	ctx: &SemaContext,
	symbol: &Arc<Definition>,
	constdef: ast::ConstDef,
) -> Result<Arc<Definition>, ()> {
	let init = constdef.initializer().unwrap();

	let Ok(consteval) = ceval::expr(ctx, init) else {
		return Err(());
	};

	Ok(Arc::new(Definition {
		kind: DefKind::Value(consteval),
		scope: Scope::default(),
	}))
}
