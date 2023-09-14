//! Semantic mid-section for VZScript.

mod ceval;

use doomfront::rowan::ast::AstNode;

use crate::{
	ast,
	compile::{
		intern::NsName,
		symbol::{DefKind, DefStatus, Definition, Symbol, Undefined},
		Compiler,
	},
	inctree::SourceKind,
	rti,
	sema::CEval,
	tsys::TypeDef,
	SyntaxNode,
};

use super::SemaContext;

#[must_use]
pub(super) fn define(ctx: &SemaContext, root: &SyntaxNode, symbol: &Symbol) -> DefStatus {
	let ast = root
		.covering_element(ctx.location.span)
		.into_node()
		.unwrap();

	unimplemented!("...soon!")
}

#[must_use]
pub(super) fn define_primitive(
	compiler: &Compiler,
	unq_name: &'static str,
) -> rti::Handle<TypeDef> {
	let name_ix = compiler.names.intern_str(unq_name);
	let sym_ix = compiler.namespaces[0]
		.get(&NsName::Type(name_ix))
		.copied()
		.unwrap();
	let symbol = compiler.symbol(sym_ix);
	let libsrc = &compiler.sources[symbol.location.lib_ix as usize];
	let pfile = &libsrc.inctree.files[symbol.location.file_ix as usize];

	let SourceKind::Vzs(ptree) = pfile.inner() else {
		unreachable!()
	};

	let ctx = SemaContext {
		compiler,
		tcache: None,
		location: symbol.location,
		path: &pfile.path,
		zscript: false,
	};

	let root = ptree.cursor();
	let status = define(&ctx, &root, symbol);
	assert_eq!(status, DefStatus::Ok);
	symbol.status.store(status);

	let guard = symbol.def.load();

	let DefKind::Value(ce) = &guard.kind else {
		unreachable!()
	};

	let CEval::TypeDef { record } = ce else {
		unreachable!()
	};

	record.handle_type()
}
