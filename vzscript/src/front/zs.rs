//! Symbol declaration for ZScript.

use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, ParseTree},
};
use util::SmallString;

use crate::{
	compile::{intern::NsName, symbol::SymbolKind},
	issue::{self, Issue},
};

use super::{DeclContext, Scope};

/// Make mixin classes known.
pub(super) fn declare_symbols_early(ctx: &DeclContext, namespace: &mut Scope, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	for top in ast {
		let ast::TopLevel::MixinClassDef(mixindef) = top else {
			continue;
		};

		let name_tok = mixindef.name().unwrap();
		let short_end = name_tok.text_range().end();

		let mut scope = Scope::default();

		for innard in mixindef.innards() {
			declare_class_innard(ctx, namespace, &mut scope, innard);
		}

		let result = ctx.declare(
			namespace,
			NsName::Type(ctx.names.intern(name_tok.text())),
			mixindef.syntax().text_range(),
			short_end,
			SymbolKind::Mixin,
			scope,
		);

		if let Err(sym_ix) = result {}
	}
}

pub(super) fn declare_symbols(ctx: &DeclContext, namespace: &mut Scope, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	for top in ast {
		match top {
			ast::TopLevel::ClassDef(classdef) => {
				declare_class(ctx, namespace, classdef);
			}
			ast::TopLevel::ClassExtend(_)
			| ast::TopLevel::MixinClassDef(_)
			| ast::TopLevel::Include(_)
			| ast::TopLevel::StructExtend(_)
			| ast::TopLevel::Version(_) => {}
			// TODO: Implement these.
			ast::TopLevel::ConstDef(_)
			| ast::TopLevel::StructDef(_)
			| ast::TopLevel::EnumDef(_) => {}
		}
	}
}

fn declare_class(ctx: &DeclContext, namespace: &mut Scope, classdef: ast::ClassDef) {
	let mut scope = Scope::default();

	let name_tok = classdef.name().unwrap();
	let r_start = classdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	for innard in classdef.innards() {
		declare_class_innard(ctx, namespace, &mut scope, innard);
	}

	let result = ctx.declare(
		namespace,
		NsName::Type(ctx.names.intern(name_tok.text())),
		classdef.syntax().text_range(),
		r_end,
		SymbolKind::Class,
		scope,
	);

	if let Err(sym_ix) = result {
		let other = ctx.symbol(sym_ix);

		let mut issue = Issue::new(
			ctx.path,
			TextRange::new(r_start, r_end),
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

fn declare_class_innard(
	ctx: &DeclContext,
	namespace: &Scope,
	scope: &mut Scope,
	innard: ast::ClassInnard,
) {
	match innard {
		ast::ClassInnard::Function(fndecl) => declare_function(ctx, scope, fndecl),
		ast::ClassInnard::Field(field) => declare_field(ctx, scope, field),
		ast::ClassInnard::Mixin(mixin) => expand_mixin(ctx, namespace, scope, mixin),
		// Handled by Sema.
		ast::ClassInnard::Default(_) | ast::ClassInnard::States(_) => {}
		// TODO: Implement these.
		ast::ClassInnard::Const(_)
		| ast::ClassInnard::Enum(_)
		| ast::ClassInnard::Struct(_)
		| ast::ClassInnard::StaticConst(_)
		| ast::ClassInnard::Property(_)
		| ast::ClassInnard::Flag(_) => {}
	}
}

fn declare_field(ctx: &DeclContext, scope: &mut Scope, field: ast::FieldDecl) {
	for name in field.names() {
		let result = ctx.declare(
			scope,
			NsName::Value(ctx.names.intern(name.ident().text())),
			name.syntax().text_range(),
			name.syntax().text_range().end(),
			SymbolKind::Field,
			Scope::default(),
		);

		if let Err(sym_ix) = result {}
	}
}

fn declare_function(ctx: &DeclContext, scope: &mut Scope, fndecl: ast::FunctionDecl) {
	let name_tok = fndecl.name();

	let result = ctx.declare(
		scope,
		NsName::Value(ctx.names.intern(name_tok.text())),
		fndecl.syntax().text_range(),
		match fndecl.const_keyword() {
			Some(kw) => kw.text_range().end(),
			None => fndecl.param_list().unwrap().syntax().text_range().end(),
		},
		SymbolKind::Function,
		Scope::default(),
	);

	if let Err(sym_ix) = result {}
}

fn expand_mixin(ctx: &DeclContext, namespace: &Scope, scope: &mut Scope, mixin: ast::MixinStat) {
	let name_tok = mixin.name().unwrap();
	let nsname = NsName::Type(ctx.names.intern(name_tok.text()));

	let lib_ix = ctx.lib_ix as usize;

	let Some(&sym_ix) = ctx.namespaces[..lib_ix].iter().rev().find_map(|ns| {
		ns.get(&nsname)
	}) else {
		ctx.raise(Issue::new(
			ctx.path,
			name_tok.text_range(),
			format!("mixin `{}` not found", name_tok.text()),
			issue::Level::Error(issue::Error::SymbolNotFound),
		));

		return;
	};

	let symbol = ctx.symbol(sym_ix);

	if symbol.kind != SymbolKind::Mixin {
		let mut issue = Issue::new(
			ctx.path,
			name_tok.text_range(),
			format!("expected symbol `{}` to be a mixin", name_tok.text()),
			issue::Level::Error(issue::Error::SymbolKindMismatch),
		);

		if let Some(o_loc) = symbol.location {
			let o_path = ctx.resolve_path(o_loc);

			issue = issue.with_label(
				o_path,
				o_loc.span,
				format!(
					"found {} `{}` here",
					symbol.kind.user_facing_name(),
					name_tok.text()
				),
			);
		} else {
			issue = issue.with_note(format!("`{}` is a primitive type", name_tok.text()));
		}

		ctx.raise(issue);

		return;
	};

	for kvp in symbol.scope.iter() {
		let mixin_sym = ctx.symbol(*kvp.1);
		let location = mixin_sym.location.unwrap();

		let result = ctx.declare(
			scope,
			*kvp.0,
			location.span,
			location.short_end,
			mixin_sym.kind,
			mixin_sym.scope.clone(),
		);

		if let Err(sym_ix) = result {}
	}
}
