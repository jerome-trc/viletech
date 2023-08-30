//! Semantic checking and name resolution for VZScript.

use doomfront::rowan::{ast::AstNode, TextRange};

use crate::{ast, compile::Compiler, ParseTree};

use super::{DeclContext, Location, Scope, Symbol, SymbolPtr, UndefKind};

pub(super) fn declare_symbols(ctx: DeclContext, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	// A first pass to make mixins known, so that their contents
	// can be exapnded into class and struct definitions by the second pass.

	for top in ast.clone() {
		let ast::TopLevel::MixinDef(mixindef) = top else {
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
						scope: Scope::default(),
					},
				));
			}
			ast::TopLevel::ClassDef(_) => todo!(),
			ast::TopLevel::ConstDef(_) => todo!(),
			ast::TopLevel::EnumDef(_) => todo!(),
			ast::TopLevel::StructDef(_) => todo!(),
			ast::TopLevel::UnionDef(_) => todo!(),
			ast::TopLevel::Annotation(_)
			| ast::TopLevel::ClassExtend(_)
			| ast::TopLevel::MixinDef(_)
			| ast::TopLevel::StructExtend(_) => continue,
		}
	}

	// A third pass to take care of extensions.

	for top in ast {
		match top {
			ast::TopLevel::ClassExtend(_) => todo!(),
			ast::TopLevel::StructExtend(_) => todo!(),
			_ => continue,
		}
	}
}

pub(super) fn resolve_names(compiler: &Compiler, ptree: &ParseTree) {}
