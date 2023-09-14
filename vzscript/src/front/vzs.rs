//! Symbol declaration for VZScript.

use crossbeam::atomic::AtomicCell;
use doomfront::rowan::{ast::AstNode, TextRange, TextSize};
use triomphe::Arc;
use util::rstring::RString;

use crate::{
	ast,
	compile::{
		builtins,
		intern::NsName,
		symbol::{DefKind, DefStatus, Definition, FunctionCode, Location, Symbol, Undefined},
		Scope,
	},
	issue::{self, Issue},
	rti,
	tsys::TypeDef,
	zname::ZName,
	ArcSwap, ParseTree, SyntaxToken,
};

use super::DeclContext;

/// Make mixins known.
pub(super) fn declare_symbols_early(ctx: &DeclContext, namespace: &mut Scope, ptree: &ParseTree) {}

pub(super) fn declare_symbols(ctx: &DeclContext, namespace: &mut Scope, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	for top in ast.clone() {
		match top {
			ast::TopLevel::FuncDecl(fndecl) => {
				declare_function(ctx, namespace, fndecl);
			}
			ast::TopLevel::ConstDef(constdef) => {
				declare_constant(ctx, namespace, constdef);
			}
			ast::TopLevel::Annotation(_)
			| ast::TopLevel::ClassDef(_)
			| ast::TopLevel::EnumDef(_)
			| ast::TopLevel::MixinDef(_)
			| ast::TopLevel::StructDef(_)
			| ast::TopLevel::UnionDef(_) => continue, // TODO
		}
	}
}

fn declare_constant(ctx: &DeclContext, scope: &mut Scope, constdef: ast::ConstDef) {
	let name_tok = constdef.name().unwrap();

	let short_end = match constdef.type_spec() {
		Some(tspec) => tspec.syntax().text_range().end(),
		None => name_tok.text_range().end(),
	};

	let result = ctx.declare(
		scope,
		name_tok.text(),
		NsName::Value(ctx.names.intern(&name_tok)),
		constdef.syntax().text_range(),
		short_end,
		Undefined::Value,
		Scope::default(),
	);

	if let Err(sym_ix) = result {
		let other = ctx.symbol(sym_ix);

		ctx.raise(
			Issue::new(
				ctx.path,
				constdef.syntax().text_range(),
				format!("attempt to re-declare symbol `{}`", name_tok.text()),
				issue::Level::Error(issue::Error::Redeclare),
			)
			.with_label(
				ctx.resolve_path(other.location),
				other.location.span,
				"previous declaration is here".to_string(),
			),
		);
	}
}

fn declare_function(ctx: &DeclContext, scope: &mut Scope, fndecl: ast::FuncDecl) {
	let name_tok = fndecl.name().unwrap();

	if ctx.lib.native {
		if let Some(attr) = fndecl
			.attributes()
			.find(|attr| attr.name().unwrap().text() == "builtin")
		{
			register_builtin(ctx, scope, fndecl, attr);
			return;
		}
	}

	let short_end = if let Some(ret_t) = fndecl.return_type() {
		ret_t.syntax().text_range().end()
	} else {
		fndecl.params().unwrap().syntax().text_range().end()
	};

	let result = ctx.declare(
		scope,
		name_tok.text(),
		NsName::Value(ctx.names.intern(&name_tok)),
		fndecl.syntax().text_range(),
		short_end,
		Undefined::Function,
		Scope::default(),
	);

	if let Err(o_sym_ix) = result {
		let symbol = ctx.symbol(o_sym_ix);

		ctx.raise(
			Issue::new(
				ctx.path,
				TextRange::new(fndecl.syntax().text_range().start(), short_end),
				format!("attempt to re-declare symbol `{}`", name_tok.text()),
				issue::Level::Error(issue::Error::Redeclare),
			)
			.with_label(
				ctx.resolve_path(symbol.location),
				symbol.location.span,
				"previous declaration is here".to_string(),
			),
		);
	}
}

fn register_builtin(
	ctx: &DeclContext,
	scope: &mut Scope,
	fndecl: ast::FuncDecl,
	attr: ast::Attribute,
) {
	let arglist = attr.args().unwrap();
	let mut args = arglist.iter();

	let arg0 = args.next().unwrap();

	let ast::Expr::Literal(lit) = arg0.expr().unwrap() else {
		unreachable!();
	};

	let token = lit.token();

	match token.name().unwrap() {
		"int_t" => {
			ctx.declare_builtin(scope, fndecl, "vzs.int_t", builtins::int_t);
		}
		"uint_t" => {
			ctx.declare_builtin(scope, fndecl, "vzs.uint_t", builtins::uint_t);
		}
		"type_of" => ctx.declare_builtin(scope, fndecl, "vzs.type_of", builtins::type_of),
		"rtti_of" => ctx.declare_builtin(scope, fndecl, "vzs.rtti_of", builtins::rtti_of),
		other => panic!("unknown builtin name: `{other}`"),
	}

	assert!(
		args.next().is_none(),
		"unexpected second argument to `builtin` attribute"
	);
}
