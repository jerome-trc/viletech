//! Include trees for DECORATE, MAPINFO, and ZScript.
//!
//! These three languages all share the ability to inline other translation units
//! using top-level directives, and so they all get a generic structure for
//! gathering their [`ParseTree`]s.

use std::{
	borrow::Cow,
	collections::VecDeque,
	path::{Path, PathBuf},
};

use crate::{parser::Parser, LangExt, ParseTree};

use super::Token;

#[derive(Debug, Default)]
pub struct IncludeTree<L: LangExt<Token = Token>> {
	pub files: Vec<FileParseTree<L>>,
	/// Paths of files that were included, but could not be found.
	pub missing: Vec<PathBuf>,
}

impl<L: LangExt<Token = Token>> IncludeTree<L> {
	/// Traverses an include tree, starting from a virtualized root path.
	#[must_use]
	pub fn new<F>(
		path: impl AsRef<Path>,
		mut filesystem: F,
		parser: fn(&mut Parser<L>),
		lex_ctx: super::lex::Context,
		inc_directive: L::Kind,
		string_lit: L::Kind,
	) -> Self
	where
		F: FnMut(&Path) -> Option<Cow<str>>,
	{
		let mut all_files = vec![];
		let mut missing = vec![];
		let mut queue = VecDeque::from([path.as_ref().to_path_buf()]);

		while let Some(queued) = queue.pop_front() {
			let source = match filesystem(&queued) {
				Some(s) => s,
				None => {
					if !missing.contains(&queued) {
						missing.push(queued);
					}

					continue;
				}
			};

			let fptree = FileParseTree {
				inner: crate::parse(source.as_ref(), parser, lex_ctx),
				path: queued,
			};

			for elem in fptree.root.children() {
				let Some(node) = elem.into_node() else {
					continue;
				};

				if node.kind() != L::kind_to_raw(inc_directive) {
					continue;
				}

				let string = node.children().last().unwrap().into_token().unwrap();

				debug_assert_eq!(string.kind(), L::kind_to_raw(string_lit));

				let text = string.text();

				if !text.is_empty() {
					queue.push_back(PathBuf::from(&text[1..(text.len() - 1)]));
				} else {
					queue.push_back(PathBuf::default());
				}
			}

			all_files.push(fptree);
		}

		Self {
			files: all_files,
			missing,
		}
	}

	/// Like [`Self::new`] but taking advantage of [`rayon`]'s global thread pool.
	#[cfg(feature = "parallel")]
	pub fn new_par<F>(
		path: impl AsRef<Path>,
		filesystem: F,
		parser: fn(&mut Parser<L>),
		lex_ctx: super::lex::Context,
		inc_directive: L::Kind,
		string_lit: L::Kind,
	) -> Self
	where
		F: Send + Sync + Fn(&Path) -> Option<Cow<str>>,
		L::Kind: Send + Sync,
	{
		use crossbeam::queue::SegQueue;
		use parking_lot::Mutex;
		use rayon::prelude::*;

		let queue = SegQueue::<PathBuf>::default();
		queue.push(path.as_ref().to_path_buf());
		let missing = Mutex::new(vec![]);
		let all_files = Mutex::new(vec![]);

		loop {
			(0..rayon::current_num_threads())
				.par_bridge()
				.for_each(|_| {
					let Some(queued) = queue.pop() else {
						return;
					};

					let source = match filesystem(&queued) {
						Some(s) => s,
						None => {
							let mut m = missing.lock();

							if !m.contains(&queued) {
								m.push(queued);
							}

							return;
						}
					};

					let fptree = FileParseTree {
						inner: crate::parse(source.as_ref(), parser, lex_ctx),
						path: queued,
					};

					for elem in fptree.root.children() {
						let Some(node) = elem.into_node() else {
							continue;
						};

						if node.kind() != L::kind_to_raw(inc_directive) {
							continue;
						}

						let string = node.children().last().unwrap().into_token().unwrap();

						debug_assert_eq!(string.kind(), L::kind_to_raw(string_lit));

						let text = string.text();

						if !text.is_empty() {
							queue.push(PathBuf::from(&text[1..(text.len() - 1)]));
						} else {
							queue.push(PathBuf::default());
						}
					}

					all_files.lock().push(fptree);
				});

			if queue.is_empty() {
				break;
			}
		}

		Self {
			files: all_files.into_inner(),
			missing: missing.into_inner(),
		}
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		self.files.iter().any(|fptree| fptree.errors.is_empty())
	}
}

#[derive(Debug)]
pub struct FileParseTree<L: LangExt<Token = Token>> {
	inner: ParseTree<L>,
	path: PathBuf,
}

impl<L: LangExt<Token = Token>> FileParseTree<L> {
	#[must_use]
	pub fn path(&self) -> &Path {
		&self.path
	}

	#[must_use]
	pub fn into_inner(self) -> ParseTree<L> {
		self.inner
	}
}

impl<L: LangExt<Token = Token>> std::ops::Deref for FileParseTree<L> {
	type Target = ParseTree<L>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[cfg(test)]
mod test {
	use crate::zdoom::{self, zscript};

	use super::*;

	const SOURCE_A: &str = r#"
	#include "file/b.zs"
	"#;

	const SOURCE_B: &str = r#"
	#include "file/c.zs"
	"#;

	const SOURCE_C: &str = r#"
	class Zauberer {}
	"#;

	fn inctree_lookup(path: &Path) -> Option<Cow<str>> {
		if path == Path::new("file/a.zs") {
			Some(Cow::Owned(SOURCE_A.to_string()))
		} else if path == Path::new("file/b.zs") {
			Some(Cow::Owned(SOURCE_B.to_string()))
		} else if path == Path::new("file/c.zs") {
			Some(Cow::Owned(SOURCE_C.to_string()))
		} else {
			None
		}
	}

	#[test]
	fn smoke_include_tree() {
		let inctree = IncludeTree::<zscript::Syntax>::new(
			"file/a.zs",
			inctree_lookup,
			zscript::parse::file,
			zdoom::lex::Context::ZSCRIPT_LATEST,
			zscript::Syntax::IncludeDirective,
			zscript::Syntax::StringLit,
		);

		assert!(inctree.missing.is_empty());
	}

	#[test]
	#[cfg(feature = "parallel")]
	fn smoke_include_tree_par() {
		let inctree = IncludeTree::<zscript::Syntax>::new_par(
			"file/a.zs",
			inctree_lookup,
			zscript::parse::file,
			zdoom::lex::Context::ZSCRIPT_LATEST,
			zscript::Syntax::IncludeDirective,
			zscript::Syntax::StringLit,
		);

		assert!(inctree.missing.is_empty());
	}
}
