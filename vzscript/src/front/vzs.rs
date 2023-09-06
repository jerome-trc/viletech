//! Symbol declaration for VZScript.

use doomfront::rowan::{ast::AstNode, TextRange};

use crate::{
	ast,
	compile::{
		intern::NsName,
		symbol::{Definition, Location, Symbol, Undefined},
		Scope,
	},
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

				let r_start = fndecl.syntax().text_range().start();
				let r_end = if let Some(ret_t) = fndecl.return_type() {
					ret_t.syntax().text_range().end()
				} else {
					fndecl.params().unwrap().syntax().text_range().end()
				};

				let result = ctx.declare(
					namespace,
					NsName::Value(ctx.names.intern(name_tok.text())),
					Symbol {
						location: Some(Location {
							file: ctx.path_ix,
							span: TextRange::new(r_start, r_end),
						}),
						source: Some(fndecl.syntax().green().into_owned()),
						def: Definition::None {
							kind: Undefined::Function,
							extra: Box::new(()),
						},
						zscript: false,
					},
				);

				if let Err((_, sym_ix)) = result {
					let symptr = ctx.symbol(sym_ix);
					let guard = symptr.load();

					let o_loc = guard.location.unwrap();
					let o_path = ctx.paths.resolve(o_loc.file);

					ctx.raise(
						Issue::new(
							ctx.path,
							TextRange::new(r_start, r_end),
							format!("attempt to re-declare symbol `{}`", name_tok.text()),
							issue::Level::Error(issue::Error::Redeclare),
						)
						.with_label(
							o_path.as_str(),
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
