//! Name resolution and semantic checking for ZScript.

use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, ParseTree},
};

use crate::{
	compile::{Compiler, SymbolKey},
	issue::{self, FileSpan, Issue, IssueLevel},
};

use super::{DeclContext, Location, Scope, Symbol, SymbolPtr, UndefKind};

pub(super) fn declare_symbols(ctx: DeclContext, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	// A first pass to make mixin classes known, so that their contents
	// can be expanded into class definitions by the second pass.

	for top in ast.clone() {
		let ast::TopLevel::MixinClassDef(mixindef) = top else {
			continue;
		};

		let name_tok = mixindef.name().unwrap();
		let name_k = ctx.names.intern(name_tok.text());

		ctx.sym_q.push((
			name_k,
			Symbol::Mixin {
				green: mixindef.syntax().green().into_owned(),
			},
		));
	}

	for top in ast.clone() {
		match top {
			ast::TopLevel::ClassDef(classdef) => {
				declare_class(ctx, classdef);
			}
			ast::TopLevel::ConstDef(constdef) => {
				let name_tok = constdef.name().unwrap();
				let name_k = ctx.names.intern(name_tok.text());
				let span = constdef.syntax().text_range();

				ctx.sym_q.push((
					name_k,
					Symbol::Undefined {
						location: Location {
							file: ctx.path_k,
							span,
						},
						kind: UndefKind::Value { mutable: false },
						scope: Scope::default(),
					},
				));
			}
			ast::TopLevel::EnumDef(enumdef) => {
				declare_enum(ctx, None, enumdef);
			}
			ast::TopLevel::StructDef(structdef) => {
				declare_struct(ctx, structdef);
			}
			ast::TopLevel::MixinClassDef(_)
			| ast::TopLevel::ClassExtend(_)
			| ast::TopLevel::StructExtend(_)
			| ast::TopLevel::Include(_)
			| ast::TopLevel::Version(_) => {}
		}
	}

	// A third pass to take care of extensions.

	for top in ast.clone() {
		match top {
			ast::TopLevel::ClassExtend(classext) => {
				extend_class(ctx, classext);
			}
			ast::TopLevel::StructExtend(structext) => {
				extend_struct(ctx, structext);
			}
			ast::TopLevel::ClassDef(_)
			| ast::TopLevel::ConstDef(_)
			| ast::TopLevel::EnumDef(_)
			| ast::TopLevel::MixinClassDef(_)
			| ast::TopLevel::Include(_)
			| ast::TopLevel::StructDef(_)
			| ast::TopLevel::Version(_) => continue,
		}
	}
}

fn declare_class(ctx: DeclContext, classdef: ast::ClassDef) {
	let mut scope = Scope::default();

	let name_tok = classdef.name().unwrap();
	let name_k = ctx.names.intern(name_tok.text());
	let r_start = classdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	ctx.sym_q.push((
		name_k,
		Symbol::Undefined {
			location: Location {
				file: ctx.path_k,
				span: TextRange::new(r_start, r_end),
			},
			kind: UndefKind::Class,
			scope,
		},
	));
}

fn declare_enum(ctx: DeclContext, outer: Option<&mut Scope>, enumdef: ast::EnumDef) {
	if let Some(o_scope) = outer {}

	let mut scope = Scope::default();

	let name_tok = enumdef.name().unwrap();
	let name_k = ctx.names.intern(name_tok.text());
	let r_start = enumdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	ctx.sym_q.push((
		name_k,
		Symbol::Undefined {
			location: Location {
				file: ctx.path_k,
				span: TextRange::new(r_start, r_end),
			},
			kind: UndefKind::Enum,
			scope,
		},
	));
}

fn declare_struct(ctx: DeclContext, structdef: ast::StructDef) {
	let mut scope = Scope::default();

	let name_tok = structdef.name().unwrap();
	let name_k = ctx.names.intern(name_tok.text());
	let r_start = structdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	ctx.sym_q.push((
		name_k,
		Symbol::Undefined {
			location: Location {
				file: ctx.path_k,
				span: TextRange::new(r_start, r_end),
			},
			kind: UndefKind::Struct,
			scope,
		},
	));
}

fn extend_class(ctx: DeclContext, classext: ast::ClassExtend) {}

fn extend_struct(ctx: DeclContext, structext: ast::StructExtend) {
	let name_tok = structext.name().unwrap();
	let name_k = ctx.names.intern(name_tok.text());

	let Some(sym_k) = ctx.global.get(&name_k) else {
		todo!("raise an issue");
		return;
	};

	let symptr = ctx.get_symbol(*sym_k);

	symptr.rcu(|undef| {
		let Symbol::Undefined { location, kind: UndefKind::Struct, scope } = undef.as_ref() else {
			todo!("raise an issue");
			return *undef;
		};

		todo!()
	});
}

pub(super) fn resolve_names(ctx: &Compiler, ptree: &ParseTree) {}
