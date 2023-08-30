use crossbeam::queue::SegQueue;
use doomfront::{
	rowan::ast::AstNode,
	zdoom::{self, zscript},
	ParseTree,
};
use parking_lot::Mutex;
use rayon::prelude::*;

use crate::{ast, Version};

#[derive(Debug, Default)]
pub struct IncludeTree {
	pub files: Vec<ParsedFile>,
	/// Paths of files that were included, but could not be found.
	pub missing: Vec<String>,
}

impl IncludeTree {
	/// Traverses an include tree, starting from a VFS root path.
	/// Note that this blocks the [`rayon`] global thread pool.
	#[must_use]
	pub fn new(fref: vfs::FileRef) -> Self {
		let vfs = fref.vfs();
		let queue = SegQueue::<String>::default();
		queue.push(fref.path().to_string_lossy().into_owned());
		let missing: parking_lot::lock_api::Mutex<parking_lot::RawMutex, Vec<String>> =
			Mutex::new(vec![]);
		let all_files = Mutex::new(vec![]);

		loop {
			(0..rayon::current_num_threads())
				.par_bridge()
				.for_each(|_| {
					let Some(queued) = queue.pop() else { return; };

					let Some(source_fref) = vfs.get(&queued) else {
						let mut m = missing.lock();

						if !m.contains(&queued) {
							m.push(queued);
						}

						return;
					};

					let Ok(source) = source_fref.try_read_str() else {
						let mut m = missing.lock();

						if !m.contains(&queued) {
							m.push(queued);
						}

						return;
					};

					let pfile = if queued
						.split('.')
						.last()
						.is_some_and(|s| s.eq_ignore_ascii_case("vzs"))
					{
						Self::parse_vzscript(source, &queue)
					} else {
						Self::parse_zscript(source, &queue)
					};

					all_files.lock().push(ParsedFile {
						inner: pfile,
						path: queued,
					});
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
	fn parse_vzscript(source: &str, queue: &SegQueue<String>) -> SourceKind {
		let ptree = doomfront::parse(
			source,
			crate::parse::file,
			// TODO: Determine version.
			Version::new(0, 0, 0),
		);

		let ast = ptree.cursor().children().filter_map(ast::TopLevel::cast);

		for top in ast {
			let ast::TopLevel::Annotation(anno) = top else {
				continue;
			};

			let Ok(name) = anno.name() else {
				continue;
			};

			let text = name.text();

			if !text.is_empty() {
				queue.push(text[1..(text.len() - 1)].to_string());
			} else {
				queue.push(String::default());
			}
		}

		SourceKind::Vzs(ptree)
	}

	#[must_use]
	fn parse_zscript(source: &str, queue: &SegQueue<String>) -> SourceKind {
		let ptree = doomfront::parse(
			source,
			zscript::parse::file,
			// TODO: Determine version from root ZSCRIPT lump.
			zdoom::lex::Context::ZSCRIPT_LATEST,
		);

		for elem in ptree.root().children() {
			let Some(node) = elem.into_node() else { continue; };

			if node.kind() != zscript::Syn::IncludeDirective.into() {
				continue;
			}

			let string = node.children().last().unwrap().into_token().unwrap();

			if string.kind() != zscript::Syn::StringLit.into() {
				continue;
			}

			let text = string.text();

			if !text.is_empty() {
				queue.push(text[1..(text.len() - 1)].to_string());
			} else {
				queue.push(String::default());
			}
		}

		SourceKind::Zs(ptree)
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		self.files.iter().any(|fptree| fptree.any_errors())
	}
}

#[derive(Debug)]
pub struct ParsedFile {
	inner: SourceKind,
	path: String,
}

#[derive(Debug)]
pub enum SourceKind {
	Vzs(ParseTree<crate::Syn>),
	Zs(ParseTree<zscript::Syn>),
}

impl ParsedFile {
	#[must_use]
	pub fn path(&self) -> &str {
		&self.path
	}

	#[must_use]
	pub fn inner(&self) -> &SourceKind {
		&self.inner
	}

	#[must_use]
	pub fn into_inner(self) -> SourceKind {
		self.inner
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		match &self.inner {
			SourceKind::Vzs(ptree) => ptree.any_errors(),
			SourceKind::Zs(ptree) => ptree.any_errors(),
		}
	}
}

impl std::ops::Deref for ParsedFile {
	type Target = SourceKind;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
