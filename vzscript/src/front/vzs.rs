//! Symbol declaration for VZScript.

use doomfront::rowan::{ast::AstNode, TextRange, TextSize};
use util::rstring::RString;

use crate::{
	ast,
	compile::{
		builtins,
		intern::NsName,
		symbol::{DefIx, Definition, FunctionCode, SymbolKind},
		Scope,
	},
	issue::{self, Issue},
	rti,
	tsys::TypeDef,
	zname::ZName,
	ParseTree, SyntaxToken,
};

use super::DeclContext;

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
		NsName::Value(ctx.names.intern(&name_tok)),
		constdef.syntax().text_range(),
		short_end,
		SymbolKind::Value,
		Scope::default(),
	);

	if let Err(sym_ix) = result {
		let other = ctx.symbol(sym_ix);

		let mut issue = Issue::new(
			ctx.path,
			constdef.syntax().text_range(),
			format!("attempt to re-declare symbol `{}`", name_tok.text()),
			issue::Level::Error(issue::Error::Redeclare),
		);

		if let Some(o_loc) = other.location {
			let o_path = ctx.resolve_path(o_loc);

			issue = issue.with_label(
				o_path,
				o_loc.span,
				"previous declaration is here".to_string(),
			);
		} else {
			unreachable!()
		}

		ctx.raise(issue);
	}
}

fn declare_function(ctx: &DeclContext, scope: &mut Scope, fndecl: ast::FuncDecl) {
	let name_tok = fndecl.name().unwrap();

	let short_end = if let Some(ret_t) = fndecl.return_type() {
		ret_t.syntax().text_range().end()
	} else {
		fndecl.params().unwrap().syntax().text_range().end()
	};

	if ctx.lib.native {
		if let Some(attr) = fndecl
			.attributes()
			.find(|attr| attr.name().unwrap().text() == "builtin")
		{
			let def_ix = register_builtin(ctx, attr);

			return;
		}
	}

	let result = ctx.declare(
		scope,
		NsName::Value(ctx.names.intern(&name_tok)),
		fndecl.syntax().text_range(),
		short_end,
		SymbolKind::Function,
		Scope::default(),
	);

	if let Err(o_sym_ix) = result {
		let symbol = ctx.symbol(o_sym_ix);
		let o_loc = symbol.location.unwrap();
		let o_path = ctx.resolve_path(o_loc);

		ctx.raise(
			Issue::new(
				ctx.path,
				TextRange::new(fndecl.syntax().text_range().start(), short_end),
				format!("attempt to re-declare symbol `{}`", name_tok.text()),
				issue::Level::Error(issue::Error::Redeclare),
			)
			.with_label(
				o_path,
				o_loc.span,
				"previous declaration is here".to_string(),
			),
		);
	}
}

#[must_use]
fn register_builtin(ctx: &DeclContext, attr: ast::Attribute) -> DefIx {
	#[must_use]
	fn define_function(ctx: &DeclContext, qname: &'static str, code: FunctionCode) -> DefIx {
		todo!()
	}

	let arglist = attr.args().unwrap();
	let mut args = arglist.iter();

	let arg0 = args.next().unwrap();

	let ast::Expr::Literal(lit) = arg0.expr().unwrap() else {
		unreachable!();
	};

	let token = lit.token();

	let def_ix = match token.string().unwrap() {
		"int_t" => define_function(
			ctx,
			"vzs.int_t",
			FunctionCode::BuiltinCEval(builtins::int_t),
		),
		"uint_t" => define_function(
			ctx,
			"vzs.uint_t",
			FunctionCode::BuiltinCEval(builtins::uint_t),
		),
		"type_of" => define_function(
			ctx,
			"vzs.type_of",
			FunctionCode::BuiltinCEval(builtins::type_of),
		),
		"rtti_of" => define_function(
			ctx,
			"vzs.rtti_of",
			FunctionCode::BuiltinCEval(builtins::rtti_of),
		),
		other => panic!("unknown builtin name: `{other}`"),
	};

	assert!(args.next().is_none());
	def_ix
}
