//! See [`declare_symbols`].

use doomfront::rowan::{ast::AstNode, TextRange};
use rayon::prelude::*;

use crate::{
	ast,
	compile::{self, Scope},
	data::{
		Confinement, DefPtr, Definition, Function, FunctionFlags, Inlining, Location, Parameter,
		SymConst, SymPtr, Visibility,
	},
	filetree::{self, FileIx},
	front::FrontendContext,
	issue::{self, Issue},
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
		return_type: SymPtr::null(),
	};

	for anno in ast.annotations() {
		let ast::AnnotationName::Unscoped(n) = anno.name().unwrap() else {
			continue;
		};

		match n.text() {
			"builtin" => process_anno_builtin(ctx, &ast, anno),
			"cold" => process_anno_cold(ctx, anno, &mut datum.flags),
			"confine" => process_anno_confinement(ctx, anno, &mut datum.confine),
			"inline" => process_anno_inline(ctx, anno, &mut datum.inlining),
			"native" => {}
			_ => continue,
		} // TODO: a generalized system is needed for these.
	}

	let param_list = ast.params().unwrap();

	for param in param_list.iter() {
		let param_ident = param.name().unwrap();

		datum.params.push(Parameter {
			name: ctx.names.intern(&param_ident),
			type_spec: SymPtr::null(),
		});
	}

	let sym = sym_ptr.as_ref();
	let def_ptr = DefPtr::alloc(ctx.arena, Definition::Function(datum));
	sym.def.store(def_ptr.as_ptr().unwrap());
}

fn declare_symconst(ctx: &FrontendContext, scope: &mut Scope, ast: ast::SymConst) {
	let ident = ast.name().unwrap();
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
		type_spec: SymPtr::null(),
	};

	let sym = sym_ptr.as_ref();
	let def_ptr = DefPtr::alloc(ctx.arena, Definition::SymConst(datum));
	sym.def.store(def_ptr.as_ptr().unwrap());
}

// Details /////////////////////////////////////////////////////////////////////

fn process_anno_builtin(
	_ctx: &FrontendContext,
	_fndecl: &ast::FunctionDecl,
	_anno: ast::Annotation,
) {
	// TODO: this is dependent on the frontend data structures representing functions,
	// which is dependent on the backend and sema interpreter.
}

fn process_anno_cold(ctx: &FrontendContext, anno: ast::Annotation, in_out: &mut FunctionFlags) {
	if let Some(arg_list) = anno.arg_list() {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg_list.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message_static("`cold` annotation takes no arguments"),
		);

		return;
	};

	in_out.insert(FunctionFlags::COLD);
}

fn process_anno_confinement(
	ctx: &FrontendContext,
	anno: ast::Annotation,
	in_out: &mut Confinement,
) {
	let Some(arg_list) = anno.arg_list() else {
		ctx.raise(
			Issue::new(
				ctx.path,
				anno.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message_static("`confine` annotation requires exactly one argument"),
		);

		return;
	};

	let mut args = arg_list.iter();

	let Some(arg0) = args.next() else {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg_list.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message_static("`confine` annotation requires exactly one argument"),
		);

		return;
	};

	if let Some(name) = arg0.name() {
		ctx.raise(
			Issue::new(
				ctx.path,
				name.inner().text_range(),
				issue::Level::Error(issue::Error::IllegalArgName),
			)
			.with_message_static("`confine` annotation does not accept named arguments"),
		);

		return;
	};

	let expr = arg0.expr().unwrap();

	let ast::Expr::Ident(e_ident) = expr else {
		ctx.raise(
			Issue::new(
				ctx.path,
				expr.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgType),
			)
			.with_message_static("`confine` annotation argument must be an identifier"),
		);

		return;
	};

	let confine = match e_ident.token().text() {
		"none" => Confinement::None,
		"ui" => Confinement::Ui,
		"sim" => Confinement::Sim,
		_ => {
			const MSG: &str = concat!(
				"`confine` annotation argument must be one of the following:",
				"\r\n- `none`",
				"\r\n- `ui`",
				"\r\n- `sim`"
			);

			ctx.raise(
				Issue::new(
					ctx.path,
					e_ident.syntax().text_range(),
					issue::Level::Error(issue::Error::AnnotationArg),
				)
				.with_message_static(MSG),
			);

			return;
		}
	};

	if let Some(arg1) = args.next() {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg1.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message_static("`confine` annotation can only accept one argument"),
		);

		return;
	}

	*in_out = confine;
}

fn process_anno_inline(ctx: &FrontendContext, anno: ast::Annotation, in_out: &mut Inlining) {
	let Some(arg_list) = anno.arg_list() else {
		*in_out = Inlining::More;
		return;
	};

	let mut args = arg_list.iter();

	let Some(arg0) = args.next() else {
		return;
	};

	if let Some(name) = arg0.name() {
		ctx.raise(
			Issue::new(
				ctx.path,
				name.inner().text_range(),
				issue::Level::Error(issue::Error::IllegalArgName),
			)
			.with_message_static("`inline` annotation does not accept named arguments"),
		);

		return;
	};

	let expr = arg0.expr().unwrap();

	let ast::Expr::Ident(e_ident) = expr else {
		ctx.raise(
			Issue::new(
				ctx.path,
				expr.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgType),
			)
			.with_message_static("`inline` annotation argument must be an identifier"),
		);

		return;
	};

	let policy = match e_ident.token().text() {
		"never" => Inlining::Never,
		"extra" => Inlining::Extra,
		_ => {
			const MSG: &str = concat!(
				"`inline` annotation argument must be one of the following:",
				"\r\n- `never`",
				"\r\n- `extra`",
			);

			ctx.raise(
				Issue::new(
					ctx.path,
					e_ident.syntax().text_range(),
					issue::Level::Error(issue::Error::AnnotationArg),
				)
				.with_message_static(MSG),
			);

			return;
		}
	};

	if let Some(arg1) = args.next() {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg1.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message_static("`inline` annotation can only accept one argument"),
		);

		return;
	}

	*in_out = policy;
}

fn redeclare_error(ctx: &FrontendContext, prev_ptr: SymPtr, crit_span: TextRange, name_str: &str) {
	let prev = prev_ptr.as_ref();
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
