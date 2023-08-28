//! A trait enabling the Lith compiler to be customized.

use crate::{
	ast,
	compile::{Compiler, LibSource, Scope},
};

pub trait CompExt: 'static + Sized {
	type Input;

	/// Import items first check for a previously-visited Lith container.
	/// If one can not be found, this will be used a fallback.
	/// If this function gets called and the count of `imports` has decreased,
	/// the program will panic; if it is left unchanged, an error will be raised.
	fn alt_import(compiler: &Compiler<Self>, imports: &mut Scope, ast: ast::Import) {}

	/// Corollary to [`crate::front::declare_symbols`].
	fn declare_symbols(compiler: &Compiler<Self>, input: &LibSource<Self>) {}
	fn post_decl(&mut self) {}

	/// Corollary to [`crate::front::resolve_imports`].
	fn resolve_imports(compiler: &Compiler<Self>, input: &LibSource<Self>) {}
	fn post_imports(&mut self) {}

	/// Corollary to [`crate::front::resolve_names`].
	fn resolve_names(compiler: &Compiler<Self>, input: &LibSource<Self>) {}
	fn post_nameres(&mut self) {}

	/// Corollary to [`crate::front::semantic_checks`].
	fn semantic_checks(compiler: &Compiler<Self>, input: &LibSource<Self>) {}
	fn post_checks(&mut self) {}
}

#[derive(Debug, Default)]
pub struct NoExt;

impl CompExt for NoExt {
	type Input = ();
}
