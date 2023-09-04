//! Name resolution and semantic checking for ZScript.

use std::{fs::File, sync::Arc};

use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, ParseTree, SyntaxNode},
};
use parking_lot::RwLock;
use util::SmallString;

use crate::{
	compile::Compiler,
	issue::{self, FileSpan, Issue, IssueLevel},
};

use super::{DeclContext, Location, Scope, Symbol, SymbolPtr, Undefined};

pub(super) fn declare_symbols(ctx: DeclContext, ptree: &ParseTree) {
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
		let iname = ctx.intern_name(name_tok.text());
		let r_start = mixindef.syntax().text_range().start();
		let r_end = name_tok.text_range().end();

		ctx.sym_q.push((
			iname,
			Symbol::Mixin {
				location: Location {
					file: ctx.ipath.clone(),
					span: TextRange::new(r_start, r_end),
				},
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
				let iname = ctx.intern_name(name_tok.text());
				let span = constdef.syntax().text_range();

				ctx.sym_q.push((
					iname,
					Symbol::Undefined {
						location: Location {
							file: ctx.ipath.clone(),
							span,
						},
						kind: Undefined::Value,
						scope: RwLock::new(Scope::default()),
					},
				));
			}
			ast::TopLevel::EnumDef(enumdef) => {
				declare_enum(ctx, None, enumdef);
			}
			ast::TopLevel::StructDef(structdef) => {
				declare_struct(ctx, None, structdef);
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
	let iname = ctx.intern_name(name_tok.text());
	let r_start = classdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	for innard in classdef.innards() {
		declare_class_innard(ctx, &mut scope, innard);
	}

	ctx.sym_q.push((
		iname,
		Symbol::Undefined {
			location: Location {
				file: ctx.ipath.clone(),
				span: TextRange::new(r_start, r_end),
			},
			kind: Undefined::Class,
			scope: RwLock::new(scope),
		},
	));
}

fn declare_class_innard(ctx: DeclContext, scope: &mut Scope, innard: ast::ClassInnard) {
	match innard {
		ast::ClassInnard::Const(constdef) => {
			declare_constant(ctx, scope, constdef);
		}
		ast::ClassInnard::Enum(enumdef) => {
			declare_enum(ctx, Some(scope), enumdef);
		}
		ast::ClassInnard::Struct(structdef) => {
			declare_struct(ctx, Some(scope), structdef);
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
			expand_mixin(ctx, scope, mixin);
		}
		ast::ClassInnard::Property(property) => {
			let name_tok = property.name().unwrap();
			let iname = ctx.intern_name(name_tok.text());

			if let Err((sym, sym_ix)) = ctx.declare(
				scope,
				iname,
				Symbol::Undefined {
					location: Location {
						file: ctx.ipath.clone(),
						span: property.syntax().text_range(),
					},
					kind: Undefined::Property,
					scope: RwLock::new(Scope::default()),
				},
			) {
				ctx.raise([Issue {
					id: FileSpan::new(ctx.ipath.as_str(), property.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::Redeclare),
					message: format!("attempt to re-declare property `{}`", name_tok.text()),
					label: todo!(),
				}]);

				return;
			}
		}
		ast::ClassInnard::Flag(flagdef) => {
			let name_tok = flagdef.name().unwrap();
			let iname = ctx.intern_name(name_tok.text());

			if let Err((sym, sym_ix)) = ctx.declare(
				scope,
				iname,
				Symbol::Undefined {
					location: Location {
						file: ctx.ipath.clone(),
						span: flagdef.syntax().text_range(),
					},
					kind: Undefined::FlagDef,
					scope: RwLock::new(Scope::default()),
				},
			) {
				ctx.raise([Issue {
					id: FileSpan::new(ctx.ipath.as_str(), flagdef.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::Redeclare),
					message: format!("attempt to re-declare flag `{}`", name_tok.text()),
					label: todo!(),
				}]);

				return;
			}

			let varname = SmallString::from_iter(["b", name_tok.text()].into_iter());
			let iname = ctx.intern_name(&varname);

			let _ = ctx
				.declare(
					scope,
					iname,
					Symbol::Undefined {
						location: Location {
							file: ctx.ipath.clone(),
							span: flagdef.syntax().text_range(),
						},
						kind: Undefined::Value,
						scope: RwLock::new(Scope::default()),
					},
				)
				.unwrap();
		}
		ast::ClassInnard::States(_) | ast::ClassInnard::Default(_) => {}
	}
}

fn declare_struct_innard(ctx: DeclContext, scope: &mut Scope, innard: ast::StructInnard) {
	match innard {
		ast::StructInnard::Const(constdef) => declare_constant(ctx, scope, constdef),
		ast::StructInnard::Enum(enumdef) => declare_enum(ctx, Some(scope), enumdef),
		ast::StructInnard::StaticConst(sconst) => declare_static_const(ctx, scope, sconst),
		ast::StructInnard::Function(fndecl) => declare_function(ctx, scope, fndecl),
		ast::StructInnard::Field(field) => declare_field(ctx, scope, field),
	}
}

fn declare_constant(ctx: DeclContext, scope: &mut Scope, constdef: ast::ConstDef) {
	let name_tok = constdef.name().unwrap();
	declare_value(
		ctx,
		scope,
		name_tok.text(),
		constdef.syntax().text_range(),
		true,
		false,
	);
}

fn declare_enum(ctx: DeclContext, mut outer: Option<&mut Scope>, enumdef: ast::EnumDef) {
	let mut scope = Scope::default();

	for variant in enumdef.variants() {
		let iname = ctx.intern_name(variant.name().text());

		let Ok(sym_ix) = ctx.declare(&mut scope, iname.clone(), Symbol::Undefined {
				location: Location {
					file: ctx.ipath.clone(),
					span: variant.syntax().text_range(),
				},
				kind: Undefined::Value,
				scope: RwLock::new(Scope::default()),
			}) else {
			unreachable!()
		};

		let Some(o_scope) = outer.as_deref_mut() else { continue; };
		o_scope.insert(iname, sym_ix);
	}

	let name_tok = enumdef.name().unwrap();
	let iname = ctx.intern_name(name_tok.text());
	let r_start = enumdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	ctx.sym_q.push((
		iname,
		Symbol::Undefined {
			location: Location {
				file: ctx.ipath.clone(),
				span: TextRange::new(r_start, r_end),
			},
			kind: Undefined::Enum,
			scope: RwLock::new(scope),
		},
	));
}

fn declare_field(ctx: DeclContext, scope: &mut Scope, field: ast::FieldDecl) {
	for name in field.names() {
		let iname = ctx.intern_name(name.ident().text());
		let mut comptime = false;
		let mut mutable = true;

		for qual in field.qualifiers().iter() {
			match qual {
				ast::MemberQual::Meta(_) => comptime = true,
				ast::MemberQual::ReadOnly(_) => mutable = false,
				_ => continue,
			}
		}

		ctx.sym_q.push((
			iname,
			Symbol::Undefined {
				location: Location {
					file: ctx.ipath.clone(),
					span: name.syntax().text_range(),
				},
				kind: Undefined::Value,
				scope: RwLock::new(Scope::default()),
			},
		));
	}
}

fn declare_function(ctx: DeclContext, scope: &mut Scope, fndecl: ast::FunctionDecl) {
	let name_tok = fndecl.name();
	let iname = ctx.intern_name(name_tok.text());

	let r_start = fndecl.syntax().text_range().start();
	let r_end = match fndecl.const_keyword() {
		Some(kw) => kw.text_range().end(),
		None => fndecl.param_list().unwrap().syntax().text_range().end(),
	};

	ctx.sym_q.push((
		iname,
		Symbol::Undefined {
			location: Location {
				file: ctx.ipath.clone(),
				span: TextRange::new(r_start, r_end),
			},
			kind: Undefined::Function,
			scope: RwLock::new(Scope::default()),
		},
	));
}

fn declare_static_const(ctx: DeclContext, scope: &mut Scope, sconst: ast::StaticConstStat) {
	let name_tok = sconst.name().unwrap();
	let r_start = sconst.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	declare_value(
		ctx,
		scope,
		name_tok.text(),
		TextRange::new(r_start, r_end),
		true,
		false,
	);
}

fn declare_struct(ctx: DeclContext, mut outer: Option<&mut Scope>, structdef: ast::StructDef) {
	let mut scope = Scope::default();

	let name_tok = structdef.name().unwrap();
	let iname = ctx.intern_name(name_tok.text());
	let r_start = structdef.syntax().text_range().start();
	let r_end = name_tok.text_range().end();

	ctx.sym_q.push((
		iname,
		Symbol::Undefined {
			location: Location {
				file: ctx.ipath.clone(),
				span: TextRange::new(r_start, r_end),
			},
			kind: Undefined::Struct,
			scope: RwLock::new(scope),
		},
	));
}

fn declare_value(
	ctx: DeclContext,
	scope: &mut Scope,
	name: &str,
	span: TextRange,
	comptime: bool,
	mutable: bool,
) {
	let iname = ctx.intern_name(name);

	ctx.sym_q.push((
		iname,
		Symbol::Undefined {
			location: Location {
				file: ctx.ipath.clone(),
				span,
			},
			kind: Undefined::Value,
			scope: RwLock::new(Scope::default()),
		},
	));
}

fn extend_class(ctx: DeclContext, classext: ast::ClassExtend) {
	let name_tok = classext.name().unwrap();

	let Some(symptr) = ctx.globals.get(name_tok.text()) else {
		ctx.raise([
			Issue {
				id: FileSpan::new(ctx.ipath.as_str(), name_tok.text_range()),
				level: IssueLevel::Error(issue::Error::SymbolNotFound),
				message: format!("class `{}` not found in this scope", name_tok.text()),
				label: None,
			}
		]);

		return;
	};

	symptr.rcu(|undef| {
		let Symbol::Undefined {
			location,
			kind: Undefined::Class { .. },
			scope
		} = undef.as_ref() else {
			ctx.raise([
				Issue {
					id: FileSpan::new(ctx.ipath.as_str(), name_tok.text_range()),
					level: IssueLevel::Error(issue::Error::SymbolKindMismatch),
					message: "can not use `extend class` on a non-class type".to_string(),
					label: None,
				}
			]);

			return undef.clone();
		};

		let mut scope = scope.write();

		for innard in classext.innards() {
			declare_class_innard(ctx, &mut scope, innard);
		}

		undef.clone()
	});
}

fn extend_struct(ctx: DeclContext, structext: ast::StructExtend) {
	let name_tok = structext.name().unwrap();
	let iname = ctx.intern_name(name_tok.text());

	let Some(symptr) = ctx.globals.get(&iname) else {
		ctx.raise([
			Issue {
				id: FileSpan::new(ctx.ipath.as_str(), name_tok.text_range()),
				level: IssueLevel::Error(issue::Error::SymbolNotFound),
				message: format!("struct `{}` not found in this scope", name_tok.text()),
				label: None,
			}
		]);

		return;
	};

	symptr.rcu(|undef| {
		let Symbol::Undefined { location, kind: Undefined::Struct, scope } = undef.as_ref() else {
			ctx.raise([
				Issue {
					id: FileSpan::new(ctx.ipath.as_str(), name_tok.text_range()),
					level: IssueLevel::Error(issue::Error::SymbolKindMismatch),
					message: "can not use `extend struct` on a non-struct type".to_string(),
					label: None,
				}
			]);

			return undef.clone();
		};

		let mut scope = scope.write();

		for innard in structext.innards() {
			declare_struct_innard(ctx, &mut scope, innard);
		}

		undef.clone()
	});
}

fn expand_mixin(ctx: DeclContext, scope: &mut Scope, mixin: ast::MixinStat) {
	let name_tok = mixin.name().unwrap();

	let Some(symptr) = ctx.globals.get(name_tok.text()) else {
		ctx.raise([
			Issue {
				id: FileSpan::new(ctx.ipath.as_str(), name_tok.text_range()),
				level: IssueLevel::Error(issue::Error::SymbolNotFound),
				message: format!("mixin `{}` not found in this scope", name_tok.text()),
				label: None,
			}
		]);

		return;
	};

	let guard = symptr.load();

	let Symbol::Mixin { location, green } = guard.as_ref() else {
		ctx.raise([
			Issue {
				id: FileSpan::new(ctx.ipath.as_str(), name_tok.text_range()),
				level: IssueLevel::Error(issue::Error::SymbolKindMismatch),
				message: format!("expected symbol `{}` to be a mixin", name_tok.text()),
				label: None,
			}
		]);

		return;
	};

	let cursor = ast::MixinClassDef::cast(SyntaxNode::new_root(green.clone())).unwrap();

	for innard in cursor.innards() {
		declare_class_innard(ctx, scope, innard);
	}
}
