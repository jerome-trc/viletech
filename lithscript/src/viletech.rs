//! A decoupled wing of LithScript for serving the VileTech Engine's specific needs.

mod decorate;
mod zscript;

use std::{
	hash::{Hash, Hasher},
	ops::Deref,
};

use crate::{
	ast,
	compile::{Compiler, IName, LibSource, Scope},
	extend::CompExt,
};
use doomfront::rowan::GreenNode;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use util::rstring::RString;

#[derive(Debug)]
pub enum CompilerExt {
	Declaration(Mutex<ExtCrit>),
	Import(ExtCrit),
	Resolution(Mutex<ExtCrit>),
	Checking(ExtCrit),
	Nil,
}

#[derive(Debug, Default)]
pub struct ExtCrit {
	pub(crate) cur_lib: usize,
	pub(crate) scopes: Vec<LibScope>,
}

impl CompExt for CompilerExt {
	type Input = LibInput;

	fn alt_import(compiler: &Compiler<Self>, imports: &mut Scope, ast: ast::Import) {
		let CompilerExt::Import(crit) = &compiler.ext else {
			unreachable!()
		};

		let path_tok = ast.file_path().unwrap();

		let Some(lib) = crit.scopes.iter().find(|l| {
			l.name == path_tok.text()
		}) else {
			return;
		};
	}

	fn declare_symbols(compiler: &Compiler<Self>, input: &LibSource<Self>) {
		assert!(
			!input.ext.zscript.any_errors(),
			"cannot compile due to ZScript parse errors"
		);

		assert!(
			!input.ext.decorate.any_errors(),
			"cannot compile due to DECORATE parse errors"
		);

		let CompilerExt::Declaration(mutex) = &compiler.ext else {
			unreachable!()
		};

		let mut crit = mutex.lock();

		crit.scopes.push(LibScope {
			name: input.name.clone(),
			scope: Scope::default(),
			mixins: FxHashMap::default(),
		});

		for file in &input.ext.zscript.files {
			let mut pass = zscript::DeclPass {
				compiler,
				ext: &mut crit,
				ptree: file,
				cur_path: &NIL_STRING,
			};

			pass.run();
		}

		for _ in &input.ext.decorate.files {}
	}

	fn post_decl(&mut self) {
		let inner = std::mem::replace(self, CompilerExt::Nil);

		let CompilerExt::Declaration(mutex) = inner else {
			unreachable!()
		};

		*self = CompilerExt::Import(mutex.into_inner());
	}

	fn post_imports(&mut self) {
		let inner = std::mem::replace(self, CompilerExt::Nil);

		let CompilerExt::Import(crit) = inner else {
			unreachable!()
		};

		*self = CompilerExt::Resolution(Mutex::new(crit));
	}

	fn resolve_names(compiler: &Compiler<Self>, input: &LibSource<Self>) {
		todo!()
	}

	fn post_nameres(&mut self) {
		let inner = std::mem::replace(self, CompilerExt::Nil);

		let CompilerExt::Resolution(mutex) = inner else {
			unreachable!()
		};

		*self = CompilerExt::Checking(mutex.into_inner());
	}

	fn semantic_checks(compiler: &Compiler<Self>, input: &LibSource<Self>) {
		let CompilerExt::Checking(mutex) = &compiler.ext else {
			unreachable!()
		};

		todo!();
	}

	fn post_checks(&mut self) {
		let inner = std::mem::replace(self, CompilerExt::Nil);

		let CompilerExt::Checking(mut crit) = inner else {
			unreachable!()
		};

		crit.cur_lib += 1;
		*self = CompilerExt::Declaration(Mutex::new(crit));
	}
}

impl ExtCrit {
	#[must_use]
	#[allow(unused)]
	pub(self) fn cur_scope(&self) -> &LibScope {
		&self.scopes[self.cur_lib]
	}

	#[must_use]
	#[allow(unused)]
	pub(self) fn cur_scope_mut(&mut self) -> &mut LibScope {
		&mut self.scopes[self.cur_lib]
	}
}

impl Default for CompilerExt {
	fn default() -> Self {
		Self::Declaration(Mutex::default())
	}
}

/// See [`crate::compile::Extension::Input`].
#[derive(Debug)]
pub struct LibInput {
	pub zscript: doomfront::zdoom::zscript::IncludeTree,
	pub decorate: doomfront::zdoom::decorate::IncludeTree,
}

#[derive(Debug)]
pub(crate) struct LibScope {
	pub(crate) name: String,
	pub(crate) scope: Scope<ZName>,
	pub(crate) mixins: FxHashMap<ZName, GreenNode>,
}

// Details /////////////////////////////////////////////////////////////////////

pub(self) const NIL_STRING: String = String::new();

/// A counterpart to [`IName`] for (G)ZDoom with a namespace discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ZName {
	Type(IName),
	Value(IName),
}

/// A counterpart to [`RString`] for (G)ZDoom,
/// with ASCII case-insensitive equality comparison and hashing.
#[derive(Debug, Clone)]
pub(crate) struct ZString(RString);

impl Deref for ZString {
	type Target = RString;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl PartialEq for ZString {
	fn eq(&self, other: &Self) -> bool {
		self.0.deref().eq_ignore_ascii_case(other.0.as_ref())
	}
}

impl Eq for ZString {}

impl Hash for ZString {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for c in self.0.deref().chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}

impl std::borrow::Borrow<str> for ZString {
	fn borrow(&self) -> &str {
		self.0.deref()
	}
}

impl From<&RString> for ZString {
	fn from(value: &RString) -> Self {
		Self(value.clone())
	}
}
