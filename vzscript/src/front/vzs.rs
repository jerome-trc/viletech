//! Name resolution and semantic checking for VZScript.

use doomfront::rowan::{ast::AstNode, TextRange};
use parking_lot::RwLock;

use crate::{ast, compile::Compiler, ParseTree};

use super::{DeclContext, Location, Scope, Symbol, SymbolPtr, UndefKind};

pub(super) fn declare_symbols(ctx: DeclContext, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	for top in ast.clone() {
		match top {
			ast::TopLevel::FuncDecl(fndecl) => {
				let name_tok = fndecl.name().unwrap();
				let name_k = ctx.names.intern(name_tok.text());

				let r_start = fndecl.syntax().text_range().start();
				let r_end = fndecl.params().unwrap().syntax().text_range().end();

				ctx.sym_q.push((
					name_k,
					Symbol::Undefined {
						location: Location {
							file: ctx.path_k,
							span: TextRange::new(r_start, r_end),
						},
						kind: UndefKind::Function,
						scope: RwLock::new(Scope::default()),
					},
				));
			}
			ast::TopLevel::ClassDef(_)
			| ast::TopLevel::ConstDef(_)
			| ast::TopLevel::EnumDef(_)
			| ast::TopLevel::StructDef(_)
			| ast::TopLevel::UnionDef(_) => unimplemented!(),
			ast::TopLevel::Annotation(_) => continue,
		}
	}
}

pub(super) fn resolve_names(compiler: &Compiler, ptree: &ParseTree) {}
