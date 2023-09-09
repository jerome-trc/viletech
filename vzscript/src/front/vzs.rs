//! Symbol declaration for VZScript.

use doomfront::rowan::{ast::AstNode, TextRange};

use crate::{
	ast,
	compile::{intern::NsName, symbol::SymbolKind, Scope},
	issue::{self, Issue},
	ParseTree,
};

use super::DeclContext;

pub(super) fn declare_symbols(ctx: &DeclContext, namespace: &mut Scope, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	for top in ast.clone() {
		match top {
			ast::TopLevel::FuncDecl(fndecl) => {
				let name_tok = fndecl.name().unwrap();

				let short_end = if let Some(ret_t) = fndecl.return_type() {
					ret_t.syntax().text_range().end()
				} else {
					fndecl.params().unwrap().syntax().text_range().end()
				};

				let result = ctx.declare(
					namespace,
					NsName::Value(ctx.names.intern(&name_tok)),
					fndecl.syntax().text_range(),
					short_end,
					SymbolKind::Function,
					Scope::default(),
				);

				if let Err(o_sym_ix) = result {
					let symbol = ctx.symbol(o_sym_ix);
					let o_loc = symbol.location.unwrap();
					let o_path = ctx.resolve_path(o_loc);

					ctx.raise(
						Issue::new(
							ctx.path,
							TextRange::new(fndecl.syntax().text_range().start(), short_end),
							format!("attempt to re-declare symbol `{}`", name_tok.text()),
							issue::Level::Error(issue::Error::Redeclare),
						)
						.with_label(
							o_path,
							o_loc.span,
							"previous declaration is here".to_string(),
						),
					);
				}
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
