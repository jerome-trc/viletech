//! See [`declare_symbols`].

use doomfront::rowan::{ast::AstNode, TextRange};
use rayon::prelude::*;
use util::pushvec::PushVec;

use crate::{
	ast,
	compile::{self},
	filetree::{self, FileIx},
	front::FrontendContext,
	issue::{self, Issue},
	sym::{
		self, Confinement, ConstInit, Datum, Function, FunctionFlags, FunctionKind, Inlining,
		Location, ParamRef, ParamType, Parameter, SymConst, Symbol, Visibility,
	},
	types::{Scope, SymPtr, TypeNPtr},
	Compiler, LibMeta,
};

/// The first stage in the Lith frontend; declaring symbols.
///
/// This only extends to symbols declared outside of "code" (i.e. function bodies
/// and initializers for container-level symbolic constants).
pub fn declare_symbols(compiler: &mut Compiler) {
	assert_eq!(compiler.arenas.len(), rayon::current_num_threads());
	assert_eq!(compiler.stage, compile::Stage::Declaration);
	debug_assert!(!compiler.any_errors());
	debug_assert!(!compiler.failed);

	for (_, (lib, lib_root)) in compiler.libs.iter().enumerate() {
		ftree_recur(compiler, lib, *lib_root);
	}

	if compiler.any_errors() {
		compiler.failed = true;
	} else {
		compiler.stage = compile::Stage::Import;
	}
}

fn ftree_recur(compiler: &Compiler, lib: &LibMeta, file_ix: FileIx) {
	compiler
		.ftree
		.graph
		.neighbors_directed(file_ix, petgraph::Outgoing)
		.par_bridge()
		.for_each(|i| {
			let ftn = &compiler.ftree.graph[i];

			match ftn {
				filetree::Node::File { ptree, path } => {
					let arena = compiler.arenas[rayon::current_thread_index().unwrap()].lock();

					let ctx = FrontendContext {
						compiler,
						arena: &arena,
						lib,
						file_ix: i,
						path: path.as_str(),
						ptree,
					};

					let scope = declare_container_symbols(&ctx);
					let overriden = compiler.scopes.insert(Location::full_file(i), scope);
					debug_assert!(overriden.is_none());
				}
				filetree::Node::Folder { .. } => {
					ftree_recur(compiler, lib, i);
				}
				filetree::Node::Root => unreachable!(),
			}
		});
}

#[must_use]
fn declare_container_symbols(ctx: &FrontendContext) -> Scope {
	let cursor = ctx.ptree.cursor();
	let mut scope = Scope::default();

	for item in cursor.children().filter_map(ast::Item::cast) {
		match item {
			ast::Item::Function(fndecl) => declare_function(ctx, &mut scope, fndecl),
			ast::Item::SymConst(symconst) => declare_symconst(ctx, &mut scope, symconst),
		}
	}

	scope
}

fn declare_function(ctx: &FrontendContext, scope: &mut Scope, ast: ast::FunctionDecl) {
	let ident = ast.name().unwrap();

	if !ctx.check_name(&ident) {
		return;
	}

	let init = || {
		let mut datum = Function {
			flags: FunctionFlags::empty(),
			_visibility: Visibility::default(),
			confine: Confinement::None,
			inlining: Inlining::default(),
			params: vec![],
			ret_type: match ast.return_type() {
				Some(t) => process_type_spec(t),
				// The void type will be filled in by sema.
				None => sym::TypeSpec::Normal(TypeNPtr::null()),
			},
			kind: FunctionKind::Ir,
		};

		let param_list = ast.params().unwrap();

		for param in param_list.iter() {
			let ptype = process_param_type_spec(param.type_spec().unwrap());

			let default = match (param.default(), &ptype) {
				(None, ParamType::Normal(_)) | (None, ParamType::Any) | (None, ParamType::Type) => {
					None
				}
				(Some(_), ParamType::Normal(_)) | (Some(_), ParamType::Any) => {
					Some(ConstInit::Value(PushVec::new()))
				}
				(Some(_), ParamType::Type) => Some(ConstInit::Type(TypeNPtr::null())),
			};

			let consteval = param.is_const();

			if matches!(&ptype, ParamType::Type) && !consteval {
				ctx.raise(
					Issue::new(
						ctx.path,
						ast.syntax().text_range(),
						issue::Level::Error(issue::Error::NonConstTypeParam),
					)
					.with_message(format!(
						"`type_t` parameter `{}` must be `const`",
						ident.text()
					)),
				); // TODO: show the change that would resolve this.

				continue;
			}

			datum.params.push(Parameter {
				name: ctx.names.intern(&ast.name().unwrap()),
				ptype,
				consteval: param.is_const(),
				reference: match param.ref_spec() {
					ast::ParamRefSpec::None => ParamRef::None,
					ast::ParamRefSpec::Ref(_) => ParamRef::Immutable,
					ast::ParamRefSpec::RefVar(_, _) => ParamRef::Mutable,
				},
				default,
			});
		}

		for anno in ast.annotations() {
			match anno.name().unwrap().text() {
				("builtin", None) => {
					super::anno::builtin_fndecl(ctx, anno, &mut datum);
				}
				("cold", None) => {
					super::anno::cold_fndecl(ctx, anno, &mut datum.flags);
				}
				("confine", None) => {
					super::anno::confine(ctx, anno, &mut datum.confine);
				}
				("crucial", None) => {
					super::anno::crucial_fndecl(ctx, anno);
				}
				("inline", None) => {
					super::anno::inline_fndecl(ctx, anno, &mut datum.inlining);
				}
				("native", None) => {
					super::anno::native_fndecl(ctx, anno, &mut datum);
				}
				other => {
					super::anno::unknown_annotation_error(ctx, anno, other);
				}
			} // TODO: a more generalized system for handling these.
		}

		match datum.kind {
			FunctionKind::Ir => {
				if ast.body().is_none() {
					ctx.raise(Issue::new(
						ctx.path,
						ast.syntax().text_range(),
						issue::Level::Error(issue::Error::MissingFnBody)
					).with_message(
						format!("declaration of function `{}` has no body", ident.text())
					).with_note_static("only functions marked `#[native]` or `#[builtin]` can be declared without a body"));
				}

				if param_list.intrinsic_params() {
					ctx.raise(Issue::new(
						ctx.path,
						param_list.syntax().text_range(),
						issue::Level::Error(issue::Error::IllegalOpaqueParams)
					).with_message_static(
						"only functions marked `#[builtin]` can be declared with a `...` parameter list"
					));
				}
			}
			FunctionKind::Builtin { .. } => {
				assert!(
					ast.body().is_none(),
					"declaration of intrinsic `{}` has illegal body",
					ident.text()
				);
			}
			FunctionKind::Native { .. } => {
				if let Some(body) = ast.body() {
					ctx.raise(
						Issue::new(
							ctx.path,
							body.syntax().text_range(),
							issue::Level::Error(issue::Error::IllegalFnBody),
						)
						.with_message(format!(
							"declaration of native function `{}` has illegal body",
							ident.text()
						)),
					);
				}
			}
		}

		Some(Datum::Function(datum))
	};

	let result = ctx.declare(scope, &ident, ast.syntax(), init);

	if let Err(prev) = result {
		redeclare_error(ctx, prev, super::crit_span(ast.syntax()), ident.text());
	}
}

fn declare_symconst(ctx: &FrontendContext, scope: &mut Scope, ast: ast::SymConst) {
	let ident = ast.name().unwrap();

	if !ctx.check_name(&ident) {
		return;
	}

	let init = || {
		let tspec = process_type_spec(ast.type_spec().unwrap());

		let init = match tspec {
			sym::TypeSpec::Type => ConstInit::Type(TypeNPtr::null()),
			sym::TypeSpec::Normal(_) => ConstInit::Value(PushVec::default()),
		};

		let datum = SymConst {
			_visibility: Visibility::default(),
			tspec,
			init,
		};

		for anno in ast.annotations() {
			match anno.name().unwrap().text() {
				("builtin", None) => {
					super::anno::builtin_non_fndecl(ctx, anno);
				}
				("cold", None) => {
					super::anno::cold_invalid(ctx, anno);
				}
				("confine", None) => {
					// TODO: valid?
				}
				("crucial", None) => {
					super::anno::crucial_nonfndecl(ctx, anno);
				}
				("inline", None) => {
					super::anno::inline_non_fndecl(ctx, anno);
				}
				("native", None) => {
					// TODO: valid?
				}
				other => {
					super::anno::unknown_annotation_error(ctx, anno, other);
				}
			}
		}

		Some(Datum::SymConst(datum))
	};

	let result = ctx.declare(scope, &ident, ast.syntax(), init);

	if let Err(prev) = result {
		redeclare_error(ctx, prev, super::crit_span(ast.syntax()), ident.text());
	}
}

// Details /////////////////////////////////////////////////////////////////////

#[must_use]
fn process_param_type_spec(tspec: ast::TypeSpec) -> ParamType {
	match tspec {
		ast::TypeSpec::Expr(_) => ParamType::Normal(TypeNPtr::null()),
		ast::TypeSpec::TypeT(_) => ParamType::Type,
		ast::TypeSpec::AnyT(_) => ParamType::Any,
	}
}

#[must_use]
fn process_type_spec(tspec: ast::TypeSpec) -> sym::TypeSpec {
	match tspec {
		ast::TypeSpec::Expr(_) => sym::TypeSpec::Normal(TypeNPtr::null()),
		ast::TypeSpec::TypeT(_) => sym::TypeSpec::Type,
		ast::TypeSpec::AnyT(_) => unreachable!(),
	}
}

fn redeclare_error(ctx: &FrontendContext, prev_ptr: SymPtr, crit_span: TextRange, name_str: &str) {
	let prev: &Symbol = &prev_ptr;
	let prev_file = ctx.resolve_file(prev);
	let prev_file_cursor = prev_file.1.cursor();

	let prev_node = prev_file_cursor
		.covering_element(prev.location.span)
		.into_node()
		.unwrap();
	let prev_span = prev_node.text_range();

	ctx.raise(
		Issue::new(
			ctx.path,
			crit_span,
			issue::Level::Error(issue::Error::Redeclare),
		)
		.with_message(format!("attempt to re-declare symbol `{name_str}`",))
		.with_label_static(prev_file.0, prev_span, "previous declaration is here"),
	);
}
