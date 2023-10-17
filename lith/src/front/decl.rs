//! See [`declare_symbols`].

use cranelift_module::FuncId;
use crossbeam::atomic::AtomicCell;
use doomfront::rowan::{ast::AstNode, TextRange};
use rayon::prelude::*;
use smallvec::SmallVec;

use crate::{
	ast,
	compile::{self},
	data::{
		ArrayLength, Confinement, Datum, DatumPtr, Function, FunctionCode, FunctionFlags, Inlining,
		IrPtr, Location, Parameter, QualType, SymConst, SymPtr, Visibility,
	},
	filetree::{self, FileIx},
	front::FrontendContext,
	issue::{self, Issue},
	Compiler, LibMeta, Scope,
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

	let result = ctx.declare(scope, &ident, ast.syntax());

	let sym_ptr = match result {
		Ok(p) => p,
		Err(prev) => {
			redeclare_error(ctx, prev, super::crit_span(ast.syntax()), ident.text());
			return;
		}
	};

	let mut datum = Function {
		flags: FunctionFlags::empty(),
		visibility: Visibility::default(),
		confine: Confinement::None,
		inlining: Inlining::default(),
		params: vec![],
		return_type: if let Some(ret_tspec) = ast.return_type() {
			process_type_spec(ctx, ret_tspec)
		} else {
			QualType::Normal {
				inner: SymPtr::null(),
				array_dims: SmallVec::default(),
				optional: false,
				reference: false,
			}
		},
		code: FunctionCode::Ir {
			ir: IrPtr::null(),
			id: AtomicCell::new(FuncId::from_bits(u32::MAX)),
		},
	};

	let param_list = ast.params().unwrap();

	for param in param_list.iter() {
		datum.params.push(Parameter {
			name: ctx.names.intern(&ast.name().unwrap()),
			qtype: process_type_spec(ctx, param.type_spec().unwrap()),
			consteval: param.is_const(),
		});
	}

	for anno in ast.annotations() {
		match anno.name().unwrap().text() {
			("builtin", None) => {
				// TODO
			}
			("cold", None) => {
				super::anno::cold_fndecl(ctx, anno, &mut datum.flags);
			}
			("confine", None) => {
				super::anno::confine(ctx, anno, &mut datum.confine);
			}
			("inline", None) => {
				super::anno::inline_fndecl(ctx, anno, &mut datum.inlining);
			}
			("native", None) => {
				// TODO
			}
			other => {
				super::anno::unknown_annotation_error(ctx, anno, other);
			}
		} // TODO: a more generalized system for handling these.
	}

	// TODO: if function is not native or builtin but has no body, raise an error.

	let sym = sym_ptr.try_ref().unwrap();
	let datum_ptr = DatumPtr::alloc(ctx.arena, Datum::Function(datum));
	sym.datum.store(datum_ptr.as_ptr().unwrap());
}

fn declare_symconst(ctx: &FrontendContext, scope: &mut Scope, ast: ast::SymConst) {
	let ident = ast.name().unwrap();

	if !ctx.check_name(&ident) {
		return;
	}

	let result = ctx.declare(scope, &ident, ast.syntax());

	let sym_ptr = match result {
		Ok(p) => p,
		Err(prev) => {
			redeclare_error(ctx, prev, super::crit_span(ast.syntax()), ident.text());
			return;
		}
	};

	let datum = SymConst {
		visibility: Visibility::default(),
		qtype: process_type_spec(ctx, ast.type_spec().unwrap()),
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

	let sym = sym_ptr.try_ref().unwrap();
	let datum_ptr = DatumPtr::alloc(ctx.arena, Datum::SymConst(datum));
	sym.datum.store(datum_ptr.as_ptr().unwrap());
}

// Details /////////////////////////////////////////////////////////////////////

#[must_use]
fn process_type_spec(_: &FrontendContext, ast: ast::TypeSpec) -> QualType {
	let texpr = ast.expr().unwrap();

	let ast::Expr::Type(e_t) = texpr else {
		return QualType::Normal {
			inner: SymPtr::null(),
			array_dims: SmallVec::default(),
			optional: false,
			reference: false,
		};
	};

	match e_t {
		ast::ExprType::Any(_) => QualType::Any { optional: false },
		ast::ExprType::TypeT(_) => QualType::Type {
			array_dims: SmallVec::default(),
			optional: false,
		},
		ast::ExprType::Prefixed(e_t_pfx) => {
			let mut array_dims = SmallVec::default();

			for prefix in e_t_pfx.prefixes() {
				match prefix {
					ast::TypePrefix::Array(_) => array_dims.push(ArrayLength::default()),
				}
			}

			QualType::Normal {
				inner: SymPtr::null(),
				array_dims,
				optional: false,
				reference: false,
			}
		}
	}
}

fn redeclare_error(ctx: &FrontendContext, prev_ptr: SymPtr, crit_span: TextRange, name_str: &str) {
	let prev = prev_ptr.try_ref().unwrap();
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
