//! See [`declare_symbols`].

use doomfront::rowan::{ast::AstNode, TextRange};
use parking_lot::Mutex;
use rayon::prelude::*;

use crate::{
	ast,
	compile::Scope,
	data::SymPtr,
	filetree,
	front::FrontendContext,
	issue::{self, Issue},
	Compiler,
};

/// The first stage in the Lith frontend; declaring symbols.
///
/// This only extends to symbols declared outside of "code" (i.e. function bodies
/// and initializers for container-level symbolic constants).
pub fn declare_symbols(compiler: &mut Compiler) {
	assert_eq!(compiler.arenas.len(), rayon::current_num_threads());
	debug_assert!(!compiler.any_errors());

	for (i, lib) in compiler.sources.iter().enumerate() {
		let scopes: Mutex<Vec<Scope>> = Mutex::default();

		lib.filetree
			.files
			.node_indices()
			.par_bridge()
			.for_each(|node_ix| {
				let arena = compiler.arenas[rayon::current_thread_index().unwrap()].lock();
				let ftn = &lib.filetree.files[node_ix];

				let filetree::Node::File { ptree, path } = ftn else {
					return;
				};

				let ctx = FrontendContext {
					compiler,
					arena: &arena,
					lib_ix: i as u16,
					file_ix: node_ix.index() as u32,
					path: path.as_str(),
					ptree,
				};

				let scope = declare_container_symbols(&ctx);
				scopes.lock().push(scope);
			});

		compiler.containers.push(scopes.into_inner());
	}

	if compiler.any_errors() {
		compiler.failed = true;
	}
}

#[must_use]
fn declare_container_symbols(ctx: &FrontendContext) -> Scope {
	let cursor = ctx.ptree.cursor();
	let mut scope = Scope::default();

	for item in cursor.children().map(|node| ast::Item::cast(node).unwrap()) {
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

	if let Err(prev) = result {
		redeclare_error(ctx, prev, super::crit_span(ast.syntax()), ident.text());
	}
}

fn declare_symconst(ctx: &FrontendContext, scope: &mut Scope, ast: ast::SymConst) {
	let ident = ast.name().unwrap();
	let result = ctx.declare(scope, &ident, ast.syntax());

	if let Err(prev) = result {
		redeclare_error(ctx, prev, super::crit_span(ast.syntax()), ident.text());
	}
}

fn redeclare_error(ctx: &FrontendContext, prev_ptr: SymPtr, crit_span: TextRange, name_str: &str) {
	let prev = prev_ptr.as_ref().unwrap();
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
