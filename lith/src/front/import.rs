//! See [`resolve_imports`].

use doomfront::rowan::{ast::AstNode, TextRange};
use rayon::prelude::*;

use crate::{
	ast, compile,
	data::{Datum, DatumPtr, Location, SymPtr, Symbol, SymbolId},
	filetree::{self, FileIx},
	issue::{self, Issue},
	Compiler, LibMeta, LutSym, Scope, Syn,
};

use super::FrontendContext;

/// The first stage in the Lith frontend; resolving imports.
pub fn resolve_imports(compiler: &mut Compiler) {
	assert!(!compiler.failed);
	assert_eq!(compiler.stage, compile::Stage::Import);

	for (_, (lib, lib_root)) in compiler.libs.iter().enumerate() {
		ftree_recur(compiler, lib, *lib_root);
	}

	if compiler.any_errors() {
		compiler.failed = true;
	} else {
		compiler.stage = compile::Stage::Sema;
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

					let scope_key = Location::full_file(i);
					let mut scope = compiler.scopes.get(&scope_key).unwrap().clone();
					resolve_container_imports(&ctx, &mut scope);
					let overwritten = compiler.scopes.insert(scope_key, scope);
					debug_assert!(overwritten.is_some());
				}
				filetree::Node::Folder { .. } => {
					ftree_recur(compiler, lib, i);
				}
				filetree::Node::Root => unreachable!(),
			}
		});
}

fn resolve_container_imports(ctx: &FrontendContext, scope: &mut Scope) {
	let cursor = ctx.ptree.cursor();

	for import in cursor.children().filter_map(ast::Import::cast) {
		resolve_import(ctx, scope, import);
	}
}

fn resolve_import(ctx: &FrontendContext, scope: &mut Scope, import: ast::Import) {
	let path_tok = import.path().unwrap();
	let path_str = path_tok.string().unwrap();

	let mut importee;
	let mut components;

	if let Some(stripped) = path_str.strip_prefix('/') {
		importee = FileIx::new(0);
		components = stripped.split('/')
	} else {
		importee = ctx.ftree.parent_of(ctx.file_ix).unwrap();
		components = path_str.split('/')
	};

	loop {
		let Some(component) = components.next() else {
			break;
		};

		match component {
			"." => continue,
			".." => {
				let Some(parent) = ctx.ftree.parent_of(importee) else {
					ctx.raise(
						Issue::new(
							ctx.path,
							path_tok.text_range(),
							issue::Level::Error(issue::Error::ImportPath),
						)
						.with_message(format!(
							"folder {} has no parent folder",
							ctx.ftree.graph[importee].path()
						)),
					);

					return;
				};

				importee = parent;
			}
			name => {
				let Some(child) = ctx.ftree.find_child(importee, name) else {
					// TODO: use Levenshtein edit distance to provide suggestions.
					ctx.raise(
						Issue::new(
							ctx.path,
							path_tok.text_range(),
							issue::Level::Error(issue::Error::ImportPath),
						)
						.with_message(format!(
							"folder {f} has no child named {name}",
							f = ctx.ftree.graph[importee].path(),
						)),
					);

					return;
				};

				importee = child;
			}
		}
	}

	if !ctx.ftree.graph[importee].is_file() {
		ctx.raise(
			Issue::new(
				ctx.path,
				path_tok.text_range(),
				issue::Level::Error(issue::Error::FolderImport),
			)
			.with_message(format!(
				"cannot import from folder {}",
				ctx.ftree.graph[importee].path(),
			)),
		);

		return;
	}

	if importee == ctx.file_ix {
		ctx.raise(
			Issue::new(
				ctx.path,
				path_tok.text_range(),
				issue::Level::Error(issue::Error::SelfImport),
			)
			.with_message(format!(
				"container {} cannot import from itself",
				ctx.ftree.graph[importee].path(),
			)),
		);

		return;
	}

	match import {
		ast::Import::List { list, .. } => {
			for entry in list.entries() {
				import_single(ctx, scope, importee, entry);
			}
		}
		ast::Import::All { inner, .. } => {
			import_all(ctx, scope, importee, inner);
		}
	}
}

fn import_single(
	ctx: &FrontendContext,
	scope: &mut Scope,
	importee: FileIx,
	entry: ast::ImportEntry,
) {
	let importee_scope = ctx
		.scopes
		.get(&Location::full_file(importee))
		.unwrap_or_else(|| {
			let path = ctx.ftree.graph[importee].path();
			panic!("no scope registered for container: {path}")
		});

	let ident;
	let orig_name = entry.name().unwrap();

	if let Some(rename) = entry.rename() {
		ident = rename;
	} else if orig_name.inner().kind() == Syn::LitName {
		ctx.raise(
			Issue::new(
				ctx.path,
				entry.syntax().text_range(),
				issue::Level::Error(issue::Error::MissingImportRename),
			)
			.with_message(format!(
				"name literal import `{}` requires a rename but has none",
				orig_name.inner().text()
			)),
		); // TODO: suggest a rename and show the syntax involved to do so.

		return;
	} else {
		ident = orig_name.inner().clone();
	};

	let o_name_ix = ctx.names.intern(orig_name.inner());

	let Some(imp_sym) = importee_scope
		.get(&o_name_ix)
		.filter(|lutsym| !lutsym.imported)
	else {
		ctx.raise(
			Issue::new(
				ctx.path,
				entry.syntax().text_range(),
				issue::Level::Error(issue::Error::SymbolNotFound),
			)
			.with_message(format!(
				"symbol `{n}` not found in container: {p}",
				n = orig_name.text(),
				p = ctx.ftree.graph[importee].path(),
			)),
		);

		for kvp in importee_scope.iter() {
			dbg!(ctx.names.resolve(*kvp.0));
		}

		return;
	};

	let name_ix = ctx.names.intern(&ident);

	match scope.entry(name_ix) {
		im::hashmap::Entry::Vacant(vac) => {
			vac.insert(imp_sym.clone());
		}
		im::hashmap::Entry::Occupied(occ) => {
			shadow_error(
				ctx,
				occ.get().clone().inner,
				ident.text_range(),
				ident.text(),
			);
		}
	}
}

fn import_all(ctx: &FrontendContext, scope: &mut Scope, importee: FileIx, inner: ast::ImportAll) {
	let importee_scope = ctx.scopes.get(&Location::full_file(importee)).unwrap();

	let rename = inner.rename().unwrap();
	let name_ix = ctx.names.intern(&rename);

	let vac = match scope.entry(name_ix) {
		im::hashmap::Entry::Vacant(vac) => vac,
		im::hashmap::Entry::Occupied(occ) => {
			shadow_error(
				ctx,
				occ.get().clone().inner,
				rename.text_range(),
				rename.text(),
			);

			return;
		}
	};

	let mut imports = Scope::default();

	for (n, lut_sym) in importee_scope.value() {
		if lut_sym.imported {
			continue;
		}

		imports.insert(
			*n,
			LutSym {
				inner: lut_sym.inner.clone(),
				imported: true,
			},
		);
	}

	let location = Location {
		span: rename.text_range(),
		file_ix: ctx.file_ix,
	};

	let imp_sym = Symbol {
		location,
		datum: DatumPtr::alloc(ctx.arena, Datum::Container(imports)),
	};

	let imp_sym_ptr = SymPtr::alloc(ctx.arena, imp_sym);

	ctx.symbols
		.insert(SymbolId::new(location), imp_sym_ptr.clone());

	vac.insert(LutSym {
		inner: imp_sym_ptr,
		imported: true,
	});
}

fn shadow_error(ctx: &FrontendContext, prev_ptr: SymPtr, entry_span: TextRange, entry_name: &str) {
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
			entry_span,
			issue::Level::Error(issue::Error::Redeclare),
		)
		.with_message(format!(
			"import `{entry_name}` conflicts with another declaration or import",
		))
		.with_label_static(prev_file.0, prev_span, "previous name is here"),
	);
}
