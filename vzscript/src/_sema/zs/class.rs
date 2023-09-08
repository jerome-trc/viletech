use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, SyntaxToken},
};

use crate::{
	compile::{
		intern::NsName,
		symbol::{ClassDef, Definition, Symbol},
		Scope,
	},
	issue::{self, Issue},
	sema::{swap_for_pending, ScopeStack, SemaContext, StackedScope},
	tsys::{self, ClassType},
};

#[must_use]
pub(super) fn define(ctx: &SemaContext, classdef: ast::ClassDef, scope: Scope) -> Symbol {
	let mut scopes = ctx.scope_stack(true);

	if let Err(()) = resolve_ancestry(ctx, &classdef, &mut scopes) {
		return ctx.error_symbol(true);
	}

	scopes.push(StackedScope::Unguarded(&scope));

	let mut typedef = ClassType {
		is_abstract: false,
		restrict: tsys::Restrict::None,
	};

	if let Err(()) = process_qualifiers(ctx, &classdef, &mut typedef) {
		return ctx.error_symbol(true);
	}

	let mut valid = true;

	for sym_ix in scope.values().copied() {
		let symptr = ctx.symbol(sym_ix);

		let (location, is_undefined) = {
			let g = symptr.load();
			(g.location.unwrap(), g.is_undefined())
		};

		if !is_undefined {
			// Defined by the compiler itself, or pending a definition.
			continue;
		}

		let undef = swap_for_pending(symptr, location, true);
		let new_ctx = ctx.clone_with(location);
		let _ = super::define(&new_ctx, undef);
	}

	for innard in classdef.innards() {
		match innard {
			ast::ClassInnard::Default(_) => todo!(),
			ast::ClassInnard::States(_) => todo!(),
			ast::ClassInnard::Const(_)
			| ast::ClassInnard::Enum(_)
			| ast::ClassInnard::Struct(_)
			| ast::ClassInnard::StaticConst(_)
			| ast::ClassInnard::Function(_)
			| ast::ClassInnard::Field(_)
			| ast::ClassInnard::Mixin(_)
			| ast::ClassInnard::Property(_)
			| ast::ClassInnard::Flag(_) => continue,
		}
	}

	if !valid {
		return ctx.error_symbol(true);
	}

	Symbol {
		location: Some(ctx.location),
		source: None,
		def: Definition::Class(Box::new(ClassDef {
			tdef: todo!(),
			scope,
		})),
		zscript: true,
	}
}

#[cfg(any())]
pub(super) fn _define(ctx: &SemaContext, classdef: ast::ClassDef) {
	#[must_use]
	fn define(
		ctx: &SemaContext,
		scope: Scope,
		classdef: ast::ClassDef,
		location: Location,
	) -> Symbol {
		for innard in classdef.innards() {
			match innard {
				ast::ClassInnard::Const(constdef) => {
					super::define_constant(ctx, constdef);
				}
				ast::ClassInnard::Enum(_) => todo!(),
				ast::ClassInnard::Struct(_) => todo!(),
				ast::ClassInnard::StaticConst(_) => todo!(),
				ast::ClassInnard::Function(fndecl) => {
					let mut valid = true;

					for qual in fndecl.qualifiers().iter() {
						match qual {
							ast::MemberQual::Action(_) => todo!(),
							ast::MemberQual::Abstract(_) => todo!(),
							ast::MemberQual::ClearScope(_) => todo!(),
							ast::MemberQual::Final(_) => todo!(),
							ast::MemberQual::Internal(_) => {
								todo!("raise an issue");
								valid = false;
							}
							ast::MemberQual::Meta(_) => todo!(),
							ast::MemberQual::Native(_) => {
								todo!("raise an issue");
								valid = false;
							}
							ast::MemberQual::Override(_) => todo!(),
							ast::MemberQual::Play(_) => todo!(),
							ast::MemberQual::Private(_) => todo!(),
							ast::MemberQual::Protected(_) => todo!(),
							ast::MemberQual::ReadOnly(_) => todo!(),
							ast::MemberQual::Static(_) => todo!(),
							ast::MemberQual::Transient(_) => todo!(),
							ast::MemberQual::Ui(_) => todo!(),
							ast::MemberQual::VarArg(_) => todo!(),
							ast::MemberQual::Virtual(_) => todo!(),
							ast::MemberQual::VirtualScope(_) => todo!(),
							ast::MemberQual::Deprecation(_) => todo!(),
							ast::MemberQual::Version(_) => {}
						}
					}

					if !valid {
						return Symbol {
							location: Some(location),
							def: Definition::Error,
						};
					}
				}
				ast::ClassInnard::Field(field) => todo!(),
				ast::ClassInnard::Default(_) => todo!(),
				ast::ClassInnard::States(_) => todo!(),
				ast::ClassInnard::Property(property) => {}
				ast::ClassInnard::Flag(flagdef) => {
					let back_tok = flagdef.backing_field().unwrap();
					let back_name_ix = ctx.names.intern(back_tok.text());
					let back_sym_ix = scope
						.get(todo!("also need to check parent class(es)"))
						.unwrap();

					todo!("resolve field, use its type to check bit validity");

					let bit_tok = flagdef.bit().unwrap();

					// Fun fact: as of GZDoom 4.10.0, the compiler places no
					// constraints on the bit to which a flagdef applies, even
					// though only 32-bit integers can back one.

					let bit = match bit_tok.int().unwrap() {
						Ok(b) => b,
						Err(err) => {
							ctx.raise([Issue {
								id: FileSpan::new(ctx.path, bit_tok.syntax().text_range()),
								level: issue::Level::Error(issue::Error::ParseInt),
								message: format!("invalid integer: {err}"),
								label: None,
							}]);

							return Symbol {
								location: Some(location),
								def: Definition::Error,
							};
						}
					};

					if bit >= 32 {
						ctx.raise([Issue {
							id: FileSpan::new(ctx.path, bit_tok.syntax().text_range()),
							level: issue::Level::Error(issue::Error::FlagDefBitOverflow),
							message: format!("bit {bit} is out of range"),
							label: None,
						}]);
					}
				}
				ast::ClassInnard::Mixin(_) => continue,
			}
		}
	}
}

fn resolve_ancestry(
	ctx: &SemaContext,
	classdef: &ast::ClassDef,
	scopes: &mut ScopeStack,
) -> Result<(), ()> {
	let Some(parent) = classdef.parent_class() else {
		let symptr = ctx.get_corelib_type("Object");
		let sscope = StackedScope::guarded(symptr.load());
		scopes.push(sscope);
		return Ok(());
	};

	let ins_pos = scopes.len();

	loop {
		let nsname = NsName::Type(ctx.names.intern(parent.text()));

		let Some(sym_ix) = ctx.global_backlookup(nsname) else {
			ctx.raise(Issue::new(
				ctx.path,
				parent.text_range(),
				format!("class `{}` not found in this scope", parent.text()),
				issue::Level::Error(issue::Error::SymbolNotFound)
			));

			return Err(());
		};

		let symptr = ctx.symbol(sym_ix);
		let guard = symptr.load();
		let sscope = StackedScope::guarded(guard);
		scopes.insert(ins_pos, sscope);
	}
}
