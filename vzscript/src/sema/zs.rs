//! Semantic mid-section for ZScript.

mod ceval;
mod class;

use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, SyntaxNode},
};

use crate::{
	compile::symbol::{DefIx, Definition, Symbol, SymbolKind},
	issue::{self, Issue},
	vir,
};

use super::SemaContext;

#[must_use]
pub(super) fn define(ctx: &SemaContext, root: &SyntaxNode, symbol: &Symbol) -> DefIx {
	let ast = root
		.covering_element(ctx.location.span)
		.into_node()
		.unwrap();

	match symbol.kind {
		SymbolKind::Class => class::define(ctx, symbol, ast::ClassDef::cast(ast).unwrap()),
		// TODO: Implement all of these.
		SymbolKind::Value => define_constant(ctx, symbol, ast::ConstDef::cast(ast).unwrap()),
		SymbolKind::Enum
		| SymbolKind::Field
		| SymbolKind::FlagDef
		| SymbolKind::Function
		| SymbolKind::Mixin
		| SymbolKind::Primitive
		| SymbolKind::Property
		| SymbolKind::Rename(_)
		| SymbolKind::Struct => {
			ctx.raise(Issue::new(
				ctx.resolve_path(ctx.location),
				TextRange::new(ctx.location.span.start(), ctx.location.short_end),
				"not yet implemented".to_string(),
				issue::Level::Error(issue::Error::Internal),
			));

			DefIx::Error
		}
		SymbolKind::Import(_) | SymbolKind::Union => unreachable!(),
	}
}

#[must_use]
fn define_constant(ctx: &SemaContext, symbol: &Symbol, constdef: ast::ConstDef) -> DefIx {
	let init = constdef.initializer().unwrap();

	let Ok(consteval) = ceval::expr(ctx, init) else {
		return DefIx::Error;
	};

	let Some(typedef) = consteval.typedef else {
		ctx.raise(Issue::new(
			ctx.path, constdef.syntax().text_range(),
			"type of constant expression could not be inferred".to_string(),
			issue::Level::Error(issue::Error::UnknownExprType),
		));

		return DefIx::Error;
	};

	let def_ix = ctx.defs.push(Definition::Constant {
		tdef: typedef,
		init: vir::Block::from(consteval.ir),
	});

	DefIx::Some(def_ix as u32)
}
