//! Parts of the frontend not concerned with [parsing] or [lexing].
//!
//! [parsing]: crate::parse
//! [lexing]: crate::syn

use std::sync::Arc;

use arc_swap::ArcSwap;
use doomfront::rowan::ast::AstNode;
use petgraph::graph::DiGraph;
use rayon::prelude::*;

use crate::{
	ast,
	compile::{self, Compiler, IName, Interner, Scope, ScopeStack, StackedScope, SymbolPtr},
	extend::CompExt,
	issue::{self, FileSpan, Issue, IssueLevel},
	rti,
	tsys::{FuncType, TypeDef, TypeHandle},
	FileTree, FxDashMap,
};

#[derive(Debug)]
pub enum Symbol {
	Enum {
		iname: IName,
		scope: Scope,
	},
	Function {
		iname: IName,
		typedef: TypeHandle<FuncType>,
		body: Option<DiGraph<Scope, ()>>,
	},
	Struct {
		iname: IName,
		scope: Scope,
	},
	Union {
		iname: IName,
		scope: Scope,
	},
	Value {
		typedef: rti::Handle<TypeDef>,
		iname: IName,
		mutable: bool,
	},
	Undefined {
		kind: UndefKind,
		scope: Scope,
	},
}

#[derive(Debug)]
pub enum UndefKind {
	Enum,
	Function,
	Struct,
	Union,
	Value { mutable: bool },
}

impl From<Symbol> for SymbolPtr {
	fn from(value: Symbol) -> Self {
		Arc::new(ArcSwap::new(Arc::new(value)))
	}
}

pub fn declare_symbols<E: CompExt>(compiler: &mut Compiler<E>)
where
	E: Send + Sync,
	E::Input: Send + Sync,
{
	assert_eq!(compiler.stage, compile::Stage::Declaration);
	assert!(compiler.cur_lib < compiler.sources.len());
	assert!(!compiler.any_errors());

	let lsrc = &compiler.sources[compiler.cur_lib];

	rayon::join(
		|| {
			let pass = DeclPass {
				interner: &compiler.interner,
				ftree: &lsrc.files,
				containers: &compiler.containers,
			};

			pass.run();
		},
		|| {
			E::declare_symbols(compiler, lsrc);
		},
	);

	compiler.stage = compile::Stage::Import;
}

pub fn resolve_imports<E: CompExt>(compiler: &mut Compiler<E>)
where
	E: Send + Sync,
	E::Input: Send + Sync,
{
	assert_eq!(compiler.stage, compile::Stage::Import);
	assert!(compiler.cur_lib < compiler.sources.len());
	assert!(!compiler.any_errors());

	let lsrc = &compiler.sources[compiler.cur_lib];

	rayon::join(
		|| {
			let pass = ImportPass {
				compiler,
				ftree: &lsrc.files,
			};

			pass.run();
		},
		|| {},
	);

	compiler.stage = compile::Stage::Resolution;
}

pub fn resolve_names<E: CompExt>(compiler: &mut Compiler<E>)
where
	E: Send + Sync,
	E::Input: Send + Sync,
{
	assert_eq!(compiler.stage, compile::Stage::Resolution);
	assert!(compiler.cur_lib < compiler.sources.len());
	assert!(!compiler.any_errors());

	let lsrc = &compiler.sources[compiler.cur_lib];

	rayon::join(
		|| {
			let pass = ResolvePass {
				interner: &compiler.interner,
				ftree: &lsrc.files,
				containers: &compiler.containers,
			};

			pass.run();
		},
		|| {},
	);

	compiler.stage = compile::Stage::Checking;
}

pub fn semantic_checks<E: CompExt>(compiler: &mut Compiler<E>) {
	assert_eq!(compiler.stage, compile::Stage::Checking);
	assert!(compiler.cur_lib < compiler.sources.len());
	assert!(!compiler.any_errors());

	compiler.cur_lib += 1;
	compiler.stage = compile::Stage::Declaration;
}

#[derive(Debug)]
pub(crate) struct Container {
	pub(crate) imports: Scope,
	pub(crate) decls: Scope,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DeclPass<'c> {
	pub(crate) interner: &'c Interner,
	pub(crate) ftree: &'c FileTree,
	pub(crate) containers: &'c FxDashMap<String, Container>,
}

impl DeclPass<'_> {
	pub(crate) fn run(self) {
		self.ftree.par_iter().for_each(|(path, ptree)| {
			let mut scope = Scope::default();
			let ast = ptree.cursor();

			for child in ast.children() {
				let top = ast::TopLevel::cast(child).unwrap();

				match top {
					ast::TopLevel::FuncDecl(fndecl) => {
						self.declare_function(&mut scope, fndecl);
					}
					ast::TopLevel::Annotation(_) | ast::TopLevel::Import(_) => {}
				}
			}

			let _ = self.containers.insert(
				path.clone(),
				Container {
					imports: Scope::default(),
					decls: scope,
				},
			);
		});
	}

	fn declare_function(self, scope: &mut Scope, fndecl: ast::FuncDecl) {
		let name_token = fndecl.name().unwrap();
		let iname = self.interner.intern(name_token.text());

		let _ = scope.insert(
			iname,
			SymbolPtr::from(Symbol::Undefined {
				kind: UndefKind::Function,
				scope: Scope::default(),
			}),
		);
	}
}

#[derive(Clone, Copy)]
pub(crate) struct ImportPass<'c, E: CompExt> {
	pub(crate) compiler: &'c Compiler<E>,
	pub(crate) ftree: &'c FileTree,
}

impl<E: CompExt> ImportPass<'_, E>
where
	E: Send + Sync,
	E::Input: Send + Sync,
{
	pub(crate) fn run(&self) {
		self.ftree.par_iter().for_each(|(cont_path, ptree)| {
			let mut imports = Scope::default();
			let ast = ptree.cursor();

			for child in ast.children() {
				let top = ast::TopLevel::cast(child).unwrap();
				let ast::TopLevel::Import(import) = top else { continue; };
				let path_tok = import.file_path().unwrap();

				let Some(icont) = self.compiler.containers.get(path_tok.text()) else {
					self.alternative_import(cont_path, &mut imports, import);
					continue;
				};

				if let Some(import_1) = import.single() {
					self.resolve_import(&mut imports, icont.value(), import_1);
				} else if let Some(import_g) = import.group() {
					for entry in import_g.entries() {
						self.resolve_import(&mut imports, icont.value(), entry);
					}
				} else {
					// Loudly indicate if the AST code has bugs.
					unreachable!()
				}
			}

			let mut container = self.compiler.containers.get_mut(cont_path).unwrap();
			container.imports = imports;
		});
	}

	fn resolve_import(&self, imports: &mut Scope, cont: &Container, entry: ast::ImportEntry) {
		let name = entry.name().unwrap();
		let iname = self.compiler.interner().intern(name.text());

		let mut stack = ScopeStack::default();

		stack.push(StackedScope {
			inner: &cont.decls,
			is_addendum: false,
		});

		if let Some(rename) = entry.rename() {}
	}

	fn alternative_import(&self, cont_path: &str, imports: &mut Scope, ast: ast::Import) {
		let prev_len = imports.len();

		E::alt_import(self.compiler, imports, ast.clone());

		if imports.len() < prev_len {
			panic!();
		} else if imports.len() == prev_len {
			let path_tok = ast.file_path().unwrap();

			self.compiler.raise(Issue {
				id: FileSpan::new(cont_path, path_tok.text_range()),
				level: IssueLevel::Error(issue::Error::ImportNotFound),
				message: format!("file `{}` not found", path_tok.text()),
				label: None,
			});
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvePass<'c> {
	pub(crate) interner: &'c Interner,
	pub(crate) ftree: &'c FileTree,
	pub(crate) containers: &'c FxDashMap<String, Container>,
}

impl ResolvePass<'_> {
	pub(crate) fn run(self) {
		self.ftree.par_iter().for_each(|(_, ptree)| {
			let ast = ptree.cursor();

			for child in ast.children() {
				let _ = ast::TopLevel::cast(child).unwrap();
			}
		});
	}
}
