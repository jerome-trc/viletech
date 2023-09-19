//! Symbol declaration for ZScript.

use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, ParseTree},
};
use util::SmallString;

use crate::{
	compile::{
		intern::NsName,
		symbol::{DefKind, Undefined},
	},
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
			name_tok.text(),
			NsName::Type(ctx.names.intern(&name_tok)),
			mixindef.syntax().text_range(),
			short_end,
			Undefined::Mixin,
			scope,
		);

		if let Err(sym_ix) = result {
			todo!()
		}
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
		name_tok.text(),
		NsName::Type(ctx.names.intern(&name_tok)),
		classdef.syntax().text_range(),
		r_end,
		Undefined::Class,
		scope,
	);

	if let Err(sym_ix) = result {
		let other = ctx.symbol(sym_ix);

		ctx.raise(
			Issue::new(
				ctx.path,
				TextRange::new(r_start, r_end),
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
		let ident = name.ident();

		let result = ctx.declare(
			scope,
			ident.text(),
			NsName::Value(ctx.names.intern(&ident)),
			name.syntax().text_range(),
			name.syntax().text_range().end(),
			Undefined::Field,
			Scope::default(),
		);

		if let Err(sym_ix) = result {}
	}
}

fn declare_function(ctx: &DeclContext, scope: &mut Scope, fndecl: ast::FunctionDecl) {
	let name_tok = fndecl.name();

	let result = ctx.declare(
		scope,
		name_tok.text(),
		NsName::Value(ctx.names.intern(&name_tok)),
		fndecl.syntax().text_range(),
		match fndecl.const_keyword() {
			Some(kw) => kw.text_range().end(),
			None => fndecl.param_list().unwrap().syntax().text_range().end(),
		},
		Undefined::Function,
		Scope::default(),
	);

	if let Err(sym_ix) = result {}
}

fn expand_mixin(ctx: &DeclContext, namespace: &Scope, scope: &mut Scope, mixin: ast::MixinStat) {
	let name_tok = mixin.name().unwrap();
	let nsname = NsName::Type(ctx.names.intern(&name_tok));

	let lib_ix = ctx.lib_ix as usize;

	let Some(&sym_ix) = ctx.namespaces[..=lib_ix]
		.iter()
		.rev()
		.find_map(|ns| ns.get(&nsname))
	else {
		ctx.raise(Issue::new(
			ctx.path,
			name_tok.text_range(),
			format!("mixin `{}` not found", &name_tok),
			issue::Level::Error(issue::Error::SymbolNotFound),
		));

		return;
	};

	let mixin_sym = ctx.symbol(sym_ix);
	let mixin_def = mixin_sym.def.load();

	if !matches!(
		mixin_def.kind,
		DefKind::None {
			kind: Undefined::Mixin,
			..
		}
	) {
		ctx.raise(
			Issue::new(
				ctx.path,
				name_tok.text_range(),
				format!("expected symbol `{}` to be a mixin", &name_tok),
				issue::Level::Error(issue::Error::SymbolKindMismatch),
			)
			.with_label(
				ctx.resolve_path(mixin_sym.location),
				mixin_sym.location.span,
				format!(
					"found {} `{}` here",
					mixin_def.kind.user_facing_name(),
					name_tok.text()
				),
			),
		);

		return;
	};

	*scope = scope
		.clone()
		.union_with(mixin_def.scope.clone(), |self_k, other_k| {
			todo!("raise an issue")
		});
}
