//! The parts of Lithica's compiler frontend not concerned with [lexing] or [parsing].
//!
//! [lexing]: crate::syn
//! [parsing]: crate::parse

pub(crate) mod anno;
pub(crate) mod decl;
pub(crate) mod import;

use doomfront::rowan::{ast::AstNode, TextRange};

use crate::{
	ast,
	compile::Scope,
	data::{DatumPtr, Location, SymPtr, Symbol},
	filetree::{self, FileIx},
	issue::{self, Issue},
	Compiler, LibMeta, LutSym, ParseTree, Syn, SyntaxNode, SyntaxToken,
};

pub use self::{decl::*, import::*};

struct FrontendContext<'c> {
	compiler: &'c Compiler,
	arena: &'c bumpalo::Bump,
	lib: &'c LibMeta,
	file_ix: FileIx,
	path: &'c str,
	ptree: &'c ParseTree,
}

impl FrontendContext<'_> {
	fn declare(
		&self,
		scope: &mut Scope,
		name: &SyntaxToken,
		node: &SyntaxNode,
	) -> Result<SymPtr, SymPtr> {
		let location = Location {
			file_ix: self.file_ix,
			span: node.text_range(),
		};

		let name = self.names.intern(name);

		let sym_ptr = match scope.entry(name) {
			im::hashmap::Entry::Vacant(vac) => {
				let sym = Symbol {
					location,
					datum: DatumPtr::null(),
				};

				let sym_ptr = SymPtr::alloc(self.arena, sym);
				self.symbols.insert(location, sym_ptr.clone());

				vac.insert(LutSym {
					inner: sym_ptr.clone(),
					imported: false,
				});

				sym_ptr
			}
			im::hashmap::Entry::Occupied(occ) => {
				return Err(occ.get().clone().inner);
			}
		};

		Ok(sym_ptr)
	}

	#[must_use]
	fn check_name(&self, ident: &SyntaxToken) -> bool {
		if self.lib.native {
			return true;
		}

		if ident.text().starts_with("__") || ident.text().ends_with("__") {
			self.raise(
				Issue::new(
					self.path,
					ident.text_range(),
					issue::Level::Error(issue::Error::IllegalSymbolName),
				)
				.with_message_static("user symbol names may not start or end with `__`")
				.with_note_static("`__` prefix/suffix is reserved for internal use"),
			);

			return false;
		}

		true
	}

	#[must_use]
	fn resolve_file(&self, sym: &Symbol) -> (&String, &ParseTree) {
		let prev_ftn = &self.ftree.graph[sym.location.file_ix];

		let filetree::Node::File { path, ptree } = prev_ftn else {
			unreachable!()
		};

		(path, ptree)
	}
}

impl std::ops::Deref for FrontendContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

/// A symbol's "critical span" is the part that is used to present diagnostics.
///
/// For example, a function definition's critical span starts at its
/// first qualifier keyword or return type token and ends at the last token
/// of its parameter list (or return type, if there is one).
#[must_use]
pub(crate) fn crit_span(node: &SyntaxNode) -> TextRange {
	if let Some(fndecl) = ast::FunctionDecl::cast(node.clone()) {
		let start = fndecl
			.syntax()
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() != Syn::DocComment))
			.unwrap()
			.text_range()
			.start();

		let end = if let Some(ret_t) = fndecl.return_type() {
			ret_t.syntax().text_range().end()
		} else if let Ok(param_list) = fndecl.params() {
			param_list.syntax().text_range().end()
		} else if let Ok(name) = fndecl.name() {
			name.text_range().end()
		} else {
			fndecl.syntax().text_range().end()
		};

		TextRange::new(start, end)
	} else if let Some(symconst) = ast::SymConst::cast(node.clone()) {
		let start = symconst
			.syntax()
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() != Syn::DocComment))
			.unwrap()
			.text_range()
			.start();

		let end = symconst.syntax().last_token().unwrap().text_range().end();

		TextRange::new(start, end)
	} else {
		unreachable!()
	}
}
