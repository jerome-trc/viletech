//! Symbol declaration for ZScript.

use doomfront::{
	rowan::{ast::AstNode, GreenNode, TextRange},
	zdoom::zscript::{ast, ParseTree, SyntaxNode},
};
use parking_lot::RwLock;
use util::SmallString;

use crate::{
	compile::{
		intern::NsName,
		symbol::{Location, Symbol, SymbolKind},
	},
	issue::{self, Issue},
};

use super::{DeclContext, Scope};

pub(super) fn declare_symbols(ctx: &DeclContext, namespace: &mut Scope, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	// Pass 1 makes mixin classes known, so that their contents
	// can be expanded into class definitions later.

	for top in ast.clone() {
		let ast::TopLevel::MixinClassDef(mixindef) = top else {
			continue;
		};

		let name_tok = mixindef.name().unwrap();
		let short_end = name_tok.text_range().end();

		let result = ctx.declare(
			namespace,
			NsName::Type(ctx.names.intern(name_tok.text())),
			mixindef.syntax().text_range(),
			short_end,
			SymbolKind::Mixin,
		);

		if let Err(sym_ix) = result {}
	}

	for top in ast.clone() {
		match top {
			ast::TopLevel::ClassDef(classdef) => {
				declare_class(ctx, namespace, classdef);
			}
			ast::TopLevel::ConstDef(constdef) => {
				declare_constant(ctx, namespace, constdef);
			}
			ast::TopLevel::EnumDef(enumdef) => {
				declare_enum(ctx, namespace, enumdef);
			}
			ast::TopLevel::StructDef(structdef) => {
				declare_struct(ctx, namespace, structdef);
			}
			ast::TopLevel::MixinClassDef(_)
			| ast::TopLevel::ClassExtend(_)
			| ast::TopLevel::StructExtend(_)
			| ast::TopLevel::Include(_)
			| ast::TopLevel::Version(_) => {}
		}
	}

	// Pass 3 takes care of extensions.

	for top in ast.clone() {
		match top {
			ast::TopLevel::ClassExtend(classext) => {
				extend_class(ctx, namespace, classext);
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
		Symbol {
			location: Some(Location {
				lib_ix: ctx.lib_ix,
				file_ix: ctx.file_ix,
				span: TextRange::new(r_start, r_end),
			}),
			source: Some(classdef.syntax().green().into_owned()),
			def: Definition::None {
				kind: Undefined::Class,
				extra: Box::new(RwLock::new(scope)),
			},
			zscript: true,
		},
	);

	if let Err((_, sym_ix)) = result {
		let symptr = ctx.symbol(sym_ix);
		let guard = symptr.load();

		let mut issue = Issue::new(
			ctx.path,
			TextRange::new(r_start, r_end),
			format!("attempt to re-declare symbol `{}`", name_tok.text()),
			issue::Level::Error(issue::Error::Redeclare),
		);

		if let Some(o_loc) = guard.location {
			let o_path = ctx.resolve_path(o_loc);

			issue = issue.with_label(
				o_path,
				o_loc.span,
				"previous declaration is here".to_string(),
			);
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
		ast::ClassInnard::Const(constdef) => {
			declare_constant(ctx, scope, constdef);
		}
		ast::ClassInnard::Enum(enumdef) => {
			declare_enum(ctx, scope, enumdef);
		}
		ast::ClassInnard::Struct(structdef) => {
			declare_struct(ctx, scope, structdef);
		}
		ast::ClassInnard::StaticConst(sconst) => {
			declare_static_const(ctx, scope, sconst);
		}
		ast::ClassInnard::Function(fndecl) => {
			declare_function(ctx, scope, fndecl);
		}
		ast::ClassInnard::Field(field) => {
			declare_field(ctx, scope, field);
		}
		ast::ClassInnard::Mixin(mixin) => {
			expand_mixin(ctx, namespace, scope, mixin);
		}
		ast::ClassInnard::Property(property) => {
			let name_tok = property.name().unwrap();
			let result = ctx.declare(
				scope,
				NsName::Property(ctx.names.intern(name_tok.text())),
				Symbol {
					location: Some(Location {
						lib_ix: ctx.lib_ix,
						file_ix: ctx.file_ix,
						span: property.syntax().text_range(),
					}),
					source: Some(property.syntax().green().into_owned()),
					def: Definition::None {
						kind: Undefined::Property,
						extra: Box::new(()),
					},
					zscript: true,
				},
			);

			if let Err((_, sym_ix)) = result {
				let symptr = ctx.symbol(sym_ix);
				let guard = symptr.load();

				let o_loc = guard.location.unwrap();
				let o_path = ctx.resolve_path(o_loc);

				ctx.raise(
					Issue::new(
						ctx.path,
						property.syntax().text_range(),
						format!("attempt to re-declare property `{}`", name_tok.text()),
						issue::Level::Error(issue::Error::Redeclare),
					)
					.with_label(
						o_path,
						o_loc.span,
						"previous property declaration is here".to_string(),
					),
				);
			}
		}
		ast::ClassInnard::Flag(flagdef) => {
			let name_tok = flagdef.name().unwrap();

			let result = ctx.declare(
				scope,
				NsName::FlagDef(ctx.names.intern(name_tok.text())),
				Symbol {
					location: Some(Location {
						lib_ix: ctx.lib_ix,
						file_ix: ctx.file_ix,
						span: flagdef.syntax().text_range(),
					}),
					source: Some(flagdef.syntax().green().into_owned()),
					def: Definition::None {
						kind: Undefined::FlagDef,
						extra: Box::new(()),
					},
					zscript: true,
				},
			);

			if let Err((_, sym_ix)) = result {
				let symptr = ctx.symbol(sym_ix);
				let guard = symptr.load();

				let o_loc = guard.location.unwrap();
				let o_path = ctx.resolve_path(o_loc);

				ctx.raise(
					Issue::new(
						ctx.path,
						flagdef.syntax().text_range(),
						format!("attempt to re-declare flag `{}`", name_tok.text()),
						issue::Level::Error(issue::Error::Redeclare),
					)
					.with_label(
						o_path,
						o_loc.span,
						"previous flagdef declaration is here".to_string(),
					),
				);

				return;
			}

			let varname = SmallString::from_iter(["b", name_tok.text()].into_iter());

			// Fun fact: as of GZDoom 4.10.0, a field name can shadow a flagdef's
			// fake boolean without any compiler complaint.

			let result = ctx.declare(
				scope,
				NsName::Value(ctx.names.intern(&varname)),
				Symbol {
					location: Some(Location {
						lib_ix: ctx.lib_ix,
						file_ix: ctx.file_ix,
						span: flagdef.syntax().text_range(),
					}),
					source: Some(flagdef.syntax().green().into_owned()),
					def: Definition::None {
						kind: Undefined::Value,
						extra: Box::new(()),
					},
					zscript: true,
				},
			);

			if let Err((_, sym_ix)) = result {
				let symptr = ctx.symbol(sym_ix);
				let o_loc = symptr.load().location.unwrap();
				let o_path = ctx.resolve_path(o_loc);

				ctx.raise(
					Issue::new(
						ctx.path,
						flagdef.syntax().text_range(),
						format!("flagdef's fake boolean `{varname}` shadows a field"),
						issue::Level::Error(issue::Error::Redeclare),
					)
					.with_label(o_path, o_loc.span, "field is defined here".to_string()),
				);
			}
		}
		ast::ClassInnard::States(_) | ast::ClassInnard::Default(_) => {}
	}
}

fn declare_struct_innard(ctx: &DeclContext, scope: &mut Scope, innard: ast::StructInnard) {
	match innard {
		ast::StructInnard::Const(constdef) => declare_constant(ctx, scope, constdef),
		ast::StructInnard::Enum(enumdef) => declare_enum(ctx, scope, enumdef),
		ast::StructInnard::StaticConst(sconst) => declare_static_const(ctx, scope, sconst),
		ast::StructInnard::Function(fndecl) => declare_function(ctx, scope, fndecl),
		ast::StructInnard::Field(field) => declare_field(ctx, scope, field),
	}
}

fn declare_constant(ctx: &DeclContext, scope: &mut Scope, constdef: ast::ConstDef) {
	let name_tok = constdef.name().unwrap();

	declare_value(
		ctx,
		scope,
		constdef.syntax().green().into_owned(),
		name_tok.text(),
		constdef.syntax().text_range(),
	);
}

fn declare_enum(ctx: &DeclContext, outer: &mut Scope, enumdef: ast::EnumDef) {
	let mut scope = Scope::default();

	for variant in enumdef.variants() {
		let name_tok = variant.name();
		let name_str = name_tok.text();
		let name_ix = ctx.names.intern(name_str);

		let result = ctx.declare(
			&mut scope,
			NsName::Value(name_ix),
			Symbol {
				location: Some(Location {
					lib_ix: ctx.lib_ix,
					file_ix: ctx.file_ix,
					span: variant.syntax().text_range(),
				}),
				source: Some(variant.syntax().green().into_owned()),
				def: Definition::None {
					kind: Undefined::Value,
					extra: Box::new(()),
				},
				zscript: true,
			},
		);

		let sym_ix = match result {
			Ok(i) => i,
			Err((_, sym_ix)) => {
				let symptr = ctx.symbol(sym_ix);
				let guard = symptr.load();
				let o_loc = guard.location.unwrap();

				ctx.raise(
					Issue::new(
						ctx.path,
						variant.syntax().text_range(),
						format!("attempt to re-declare enum variant `{name_str}`"),
						issue::Level::Error(issue::Error::Redeclare),
					)
					.with_label(ctx.path, o_loc.span, "previously declared here".to_string()),
				);

				continue;
			}
		};

		outer.insert(NsName::Value(name_ix), sym_ix);
	}

	let name_tok = enumdef.name().unwrap();
	let r_start = enumdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	let result = ctx.declare(
		outer,
		NsName::Type(ctx.names.intern(name_tok.text())),
		Symbol {
			location: Some(Location {
				lib_ix: ctx.lib_ix,
				file_ix: ctx.file_ix,
				span: TextRange::new(r_start, r_end),
			}),
			source: Some(enumdef.syntax().green().into_owned()),
			def: Definition::None {
				kind: Undefined::Enum,
				extra: Box::new(RwLock::new(scope)),
			},
			zscript: true,
		},
	);

	if let Err((_, sym_ix)) = result {}
}

fn declare_field(ctx: &DeclContext, scope: &mut Scope, field: ast::FieldDecl) {
	for name in field.names() {
		let result = ctx.declare(
			scope,
			NsName::Value(ctx.names.intern(name.ident().text())),
			Symbol {
				location: Some(Location {
					lib_ix: ctx.lib_ix,
					file_ix: ctx.file_ix,
					span: name.syntax().text_range(),
				}),
				source: Some(field.syntax().green().into_owned()),
				def: Definition::None {
					kind: Undefined::Value,
					extra: Box::new(()),
				},
				zscript: true,
			},
		);

		if let Err((_, sym_ix)) = result {}
	}
}

fn declare_function(ctx: &DeclContext, scope: &mut Scope, fndecl: ast::FunctionDecl) {
	let name_tok = fndecl.name();

	let r_start = fndecl.syntax().text_range().start();
	let r_end = match fndecl.const_keyword() {
		Some(kw) => kw.text_range().end(),
		None => fndecl.param_list().unwrap().syntax().text_range().end(),
	};

	let result = ctx.declare(
		scope,
		NsName::Value(ctx.names.intern(name_tok.text())),
		Symbol {
			location: Some(Location {
				lib_ix: ctx.lib_ix,
				file_ix: ctx.file_ix,
				span: TextRange::new(r_start, r_end),
			}),
			source: Some(fndecl.syntax().green().into_owned()),
			def: Definition::None {
				kind: Undefined::Function,
				extra: Box::new(()),
			},
			zscript: true,
		},
	);

	if let Err((_, sym_ix)) = result {}
}

fn declare_static_const(ctx: &DeclContext, scope: &mut Scope, sconst: ast::StaticConstStat) {
	let name_tok = sconst.name().unwrap();
	let r_start = sconst.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	declare_value(
		ctx,
		scope,
		sconst.syntax().green().into_owned(),
		name_tok.text(),
		TextRange::new(r_start, r_end),
	);
}

fn declare_struct(ctx: &DeclContext, outer: &mut Scope, structdef: ast::StructDef) {
	let mut scope = Scope::default();

	let name_tok = structdef.name().unwrap();
	let r_start = structdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	for innard in structdef.innards() {
		declare_struct_innard(ctx, &mut scope, innard);
	}

	let result = ctx.declare(
		outer,
		NsName::Type(ctx.names.intern(name_tok.text())),
		Symbol {
			location: Some(Location {
				lib_ix: ctx.lib_ix,
				file_ix: ctx.file_ix,
				span: TextRange::new(r_start, r_end),
			}),
			source: Some(structdef.syntax().green().into_owned()),
			def: Definition::None {
				kind: Undefined::Struct,
				extra: Box::new(RwLock::new(scope)),
			},
			zscript: true,
		},
	);

	if let Err((_, sym_ix)) = result {}
}

fn declare_value(
	ctx: &DeclContext,
	scope: &mut Scope,
	source: GreenNode,
	name_str: &str,
	span: TextRange,
) {
	let result = ctx.declare(
		scope,
		NsName::Value(ctx.names.intern(name_str)),
		Symbol {
			location: Some(Location {
				lib_ix: ctx.lib_ix,
				file_ix: ctx.file_ix,
				span,
			}),
			source: Some(source),
			def: Definition::None {
				kind: Undefined::Value,
				extra: Box::new(()),
			},
			zscript: true,
		},
	);

	if let Err((_, sym_ix)) = result {}
}

fn extend_class(ctx: &DeclContext, namespace: &Scope, classext: ast::ClassExtend) {
	let name_tok = classext.name().unwrap();
	let nsname = NsName::Type(ctx.names.intern(name_tok.text()));

	let Some(&sym_ix) = ctx.globals.get(&nsname) else {
		ctx.raise(Issue::new(
			ctx.path,
			name_tok.text_range(),
			format!("class `{}` not found in this scope", name_tok.text()),
			issue::Level::Error(issue::Error::SymbolNotFound),
		));

		return;
	};

	let symptr = ctx.symbol(sym_ix);

	symptr.rcu(|symbol| {
		let Definition::None { kind, extra } = &symbol.def else {
			unreachable!()
		};

		if *kind != Undefined::Class {
			ctx.raise(Issue::new(
				ctx.path,
				name_tok.text_range(),
				"can not use `extend class` on a non-class type".to_string(),
				issue::Level::Error(issue::Error::SymbolKindMismatch),
			));

			return symbol.clone();
		}

		let lock = extra.downcast_ref::<RwLock<Scope>>().unwrap();
		let mut scope = lock.write();

		for innard in classext.innards() {
			declare_class_innard(ctx, namespace, &mut scope, innard);
		}

		symbol.clone()
	});
}

fn extend_struct(ctx: &DeclContext, structext: ast::StructExtend) {
	let name_tok = structext.name().unwrap();
	let nsname = NsName::Type(ctx.names.intern(name_tok.text()));

	let Some(&sym_ix) = ctx.globals.get(&nsname) else {
		ctx.raise(Issue::new(
			ctx.path,
			name_tok.text_range(),
			format!("struct `{}` not found in this scope", name_tok.text()),
			issue::Level::Error(issue::Error::SymbolNotFound),
		));

		return;
	};

	let symptr = ctx.symbol(sym_ix);

	symptr.rcu(|symbol| {
		let Definition::None { kind, extra } = &symbol.def else {
			unreachable!()
		};

		if *kind != Undefined::Struct {
			ctx.raise(Issue::new(
				ctx.path,
				name_tok.text_range(),
				"can not use `extend struct` on a non-struct type".to_string(),
				issue::Level::Error(issue::Error::SymbolKindMismatch),
			));

			return symbol.clone();
		}

		let lock = extra.downcast_ref::<RwLock<Scope>>().unwrap();
		let mut scope = lock.write();

		for innard in structext.innards() {
			declare_struct_innard(ctx, &mut scope, innard);
		}

		symbol.clone()
	});
}

fn expand_mixin(ctx: &DeclContext, namespace: &Scope, scope: &mut Scope, mixin: ast::MixinStat) {
	let name_tok = mixin.name().unwrap();
	let nsname = NsName::Type(ctx.names.intern(name_tok.text()));

	let Some(&sym_ix) = namespace.get(&nsname) else {
		ctx.raise(Issue::new(
			ctx.path,
			name_tok.text_range(),
			format!("mixin `{}` not found in this scope", name_tok.text()),
			issue::Level::Error(issue::Error::SymbolNotFound),
		));

		return;
	};

	let symptr = ctx.symbol(sym_ix);
	let guard = symptr.load();

	let Definition::Mixin = &guard.def else {
		let mut issue = Issue::new(
			ctx.path,
			name_tok.text_range(),
			format!("expected symbol `{}` to be a mixin", name_tok.text()),
			issue::Level::Error(issue::Error::SymbolKindMismatch),
		);

		if let Some(o_loc) = guard.location {
			let o_path = ctx.resolve_path(o_loc);

			issue = issue.with_label(
				o_path,
				o_loc.span,
				format!("found {} `{}` here", guard.def.user_facing_name(), name_tok.text())
			);
		} else {
			issue = issue.with_note(format!("`{}` is a primitive type", name_tok.text()));
		}

		ctx.raise(issue);

		return;
	};

	let green = guard.source.as_ref().unwrap().clone();
	let cursor = ast::MixinClassDef::cast(SyntaxNode::new_root(green)).unwrap();

	for innard in cursor.innards() {
		declare_class_innard(ctx, namespace, scope, innard);
	}
}
