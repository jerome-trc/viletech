//! Parts of the frontend not concerned with [parsing] or [lexing].
//!
//! [parsing]: crate::parse
//! [lexing]: crate::Syn

mod deco;
mod vzs;
mod zs;

use crossbeam::atomic::AtomicCell;
use doomfront::{
	rowan::{ast::AstNode, TextRange, TextSize},
	zdoom::zscript,
	ParseTree,
};
use parking_lot::Mutex;

use rayon::prelude::*;
use triomphe::Arc;
use util::rstring::RString;

use crate::{
	ast,
	compile::{
		self,
		intern::{NsName, SymbolIx},
		symbol::{DefKind, DefStatus, Definition, Location, Symbol, Undefined},
		CEvalBuiltin, Compiler, LibSource, Scope,
	},
	inctree::SourceKind,
	issue::{self, Issue},
	sema::CEval,
	ArcSwap, FxDashMap, ZName,
};

pub fn declare_symbols(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Declaration);
	debug_assert!(!compiler.any_errors());

	for (i, libsrc) in compiler.sources.iter().enumerate() {
		let namespace = libsrc
			.inctree
			.files
			.par_iter()
			.enumerate()
			.fold(Scope::default, |mut scope, (ii, pfile)| {
				let libname = libsrc.name.clone();

				let mut ctx = DeclContext {
					compiler,
					lib: libsrc,
					path: &pfile.path,
					lib_ix: i as u16,
					file_ix: ii as u16,
					zscript: false,
					name_stack: vec![libsrc.name.clone()],
				};

				match &pfile.inner {
					SourceKind::Vzs(ptree) => vzs::declare_symbols_early(&ctx, &mut scope, ptree),
					SourceKind::Zs(ptree) => {
						ctx.zscript = true;
						zs::declare_symbols_early(&ctx, &mut scope, ptree);
					}
				}

				scope
			})
			.reduce(Scope::default, |u1, u2| {
				u1.union_with(u2, |u1_k, u2_k| {
					report_redeclare(compiler, u1_k, u2_k);
					u2_k
				})
			});

		compiler.namespaces.push(namespace);
	}

	for (i, libsrc) in compiler.sources.iter().enumerate() {
		let namespace = libsrc
			.inctree
			.files
			.par_iter()
			.enumerate()
			.fold(Scope::default, |mut scope, (ii, pfile)| {
				let mut ctx = DeclContext {
					compiler,
					lib: libsrc,
					path: &pfile.path,
					lib_ix: i as u16,
					file_ix: ii as u16,
					zscript: false,
					name_stack: vec![libsrc.name.clone()],
				};

				match &pfile.inner {
					SourceKind::Vzs(ptree) => vzs::declare_symbols(&ctx, &mut scope, ptree),
					SourceKind::Zs(ptree) => {
						ctx.zscript = true;
						zs::declare_symbols(&ctx, &mut scope, ptree);
					}
				}

				scope
			})
			.reduce(Scope::default, |u1, u2| {
				u1.union_with(u2, |u1_k, u2_k| {
					report_redeclare(compiler, u1_k, u2_k);
					u2_k
				})
			});

		let ns = std::mem::take(&mut compiler.namespaces[i]);

		compiler.namespaces[i] = ns.union_with(namespace, |u1_k, u2_k| {
			report_redeclare(compiler, u1_k, u2_k);
			u2_k
		});
	}

	for i in 0..compiler.namespaces.len() {
		let mut u = compiler.namespaces[i].clone();
		let background = &compiler.namespaces[..i];

		for background in &compiler.namespaces[..i] {
			u = u.union(background.clone());
		}

		compiler.namespaces[i] = u;
	}

	compiler.stage = compile::Stage::Semantic;

	if compiler.any_errors() {
		compiler.failed = true;
	}
}

#[derive(Debug)]
struct DeclContext<'c> {
	compiler: &'c Compiler,
	lib: &'c LibSource,
	path: &'c str,
	lib_ix: u16,
	file_ix: u16,
	zscript: bool,
	name_stack: Vec<String>,
}

impl DeclContext<'_> {
	/// If `Err` is returned, it contains the index
	/// to the symbol that would have been overwritten.
	#[allow(clippy::too_many_arguments)]
	fn declare(
		&self,
		outer: &mut Scope,
		name_str: &str,
		nsname: NsName,
		span: TextRange,
		short_end: TextSize,
		kind: Undefined,
		scope: Scope,
	) -> Result<SymbolIx, SymbolIx> {
		match outer.entry(nsname) {
			im::hashmap::Entry::Vacant(vac) => {
				let name_str_arr = [name_str];

				let str_iter = self
					.name_stack
					.iter()
					.map(|s| s.as_str())
					.chain(name_str_arr.iter().copied());

				let qname = ZName::new(RString::from_str_iter(str_iter));

				let symbol = Symbol {
					name: nsname.index(),
					location: Location {
						lib_ix: self.lib_ix,
						file_ix: self.file_ix,
						span,
						short_end,
					},
					zscript: self.zscript,
					status: AtomicCell::new(DefStatus::None),
					def: ArcSwap::new(triomphe::Arc::new(Definition {
						kind: DefKind::None { kind, qname },
						scope,
					})),
				};

				let ix = SymbolIx(self.symbols.push(symbol) as u32);
				vac.insert(ix);

				Ok(ix)
			}
			im::hashmap::Entry::Occupied(occ) => Err(*occ.get()),
		}
	}

	fn declare_builtin(
		&self,
		outer: &mut Scope,
		fndecl: ast::FuncDecl,
		qname: &'static str,
		function: CEvalBuiltin,
	) {
		let short_end = if let Some(ret_t) = fndecl.return_type() {
			ret_t.syntax().text_range().end()
		} else {
			fndecl.params().unwrap().syntax().text_range().end()
		};

		let name_ix = self.names.intern(&fndecl.name().unwrap());
		let nsname = NsName::Value(name_ix);

		let symbol = Symbol {
			name: name_ix,
			location: Location {
				lib_ix: self.lib_ix,
				file_ix: self.file_ix,
				span: fndecl.syntax().text_range(),
				short_end,
			},
			zscript: false,
			status: AtomicCell::new(DefStatus::Ok),
			def: ArcSwap::new(Arc::new(Definition {
				kind: DefKind::Builtin { function },
				scope: Scope::default(),
			})),
		};

		let sym_ix = SymbolIx(self.symbols.push(symbol) as u32);
		let clobbered = outer.insert(nsname, sym_ix);
		assert!(clobbered.is_none());
	}
}

impl std::ops::Deref for DeclContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

fn report_redeclare(compiler: &Compiler, u1_k: SymbolIx, u2_k: SymbolIx) {
	let sym1 = compiler.symbol(u1_k);
	let sym2 = compiler.symbol(u2_k);
	let sym1_name_str = compiler.names.resolve(sym1.name);

	compiler.raise(
		Issue::new(
			compiler.resolve_path(sym1.location),
			TextRange::new(sym1.location.span.start(), sym1.location.short_end),
			format!("attempt to re-declare symbol `{sym1_name_str}`"),
			issue::Level::Error(issue::Error::Redeclare),
		)
		.with_label(
			compiler.resolve_path(sym2.location),
			TextRange::new(sym2.location.span.start(), sym2.location.short_end),
			"previous declaration is here".to_string(),
		),
	);
}
