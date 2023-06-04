//! Note that the results of these parsers are likely to be incorrect if used with
//! a [`TokenStream`] bearing a version above [`V1_0_0`](crate::zdoom::Version::V1_0_0).

mod actor;
mod common;
mod expr;
mod top;

use std::{
	borrow::Cow,
	collections::VecDeque,
	path::{Path, PathBuf},
};

use chumsky::{primitive, IterParser, Parser};

use crate::{
	util::builder::GreenCache,
	zdoom::{
		self,
		lex::{Token, TokenStream},
		Extra, ParseTree,
	},
};

pub use self::{actor::*, common::*, expr::*, top::*};

use super::Syn;

pub fn file<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		trivia(),
		actor_def(),
		include_directive(),
		const_def(),
		enum_def(),
	))
	.repeated()
	.collect::<()>()
	.boxed()
}

#[derive(Debug)]
pub struct FileParseTree {
	inner: ParseTree<'static>,
	path: PathBuf,
}

impl FileParseTree {
	#[must_use]
	pub fn path(&self) -> &Path {
		&self.path
	}

	#[must_use]
	pub fn into_inner(self) -> ParseTree<'static> {
		self.inner
	}
}

impl std::ops::Deref for FileParseTree {
	type Target = ParseTree<'static>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Default)]
pub struct IncludeTree {
	pub files: Vec<FileParseTree>,
	/// Paths of files that were included, but could not be found.
	pub missing: Vec<PathBuf>,
}

impl IncludeTree {
	/// Traverses a DECORATE "include tree", starting from a virtualized root path.
	///
	/// `Err` is returned only if one or more files included cannot be found
	/// by the given `Filesystem` implementation. The include tree within will
	/// still contain results that are otherwise valid for all other found files.
	pub fn new<F, C>(
		version: Option<zdoom::Version>,
		mut filesystem: F,
		path: impl AsRef<Path>,
		gcache: Option<C>,
	) -> Result<Self, Self>
	where
		F: FnMut(&Path) -> Option<Cow<str>>,
		C: GreenCache,
	{
		let mut all_files = vec![];
		let mut missing = vec![];
		let mut queue = VecDeque::from([path.as_ref().to_path_buf()]);

		while let Some(queued) = queue.pop_front() {
			let source = match filesystem(&queued) {
				Some(s) => s,
				None => {
					missing.push(queued);
					continue;
				}
			};

			let parser = file();
			let stream = Token::stream(source.as_ref(), version);
			let ptree = crate::parse(
				parser,
				gcache.clone(),
				Syn::Root.into(),
				source.as_ref(),
				stream,
			);

			let fptree = FileParseTree {
				inner: ParseTree {
					root: ptree.root,
					errors: ptree
						.errors
						.into_iter()
						.map(|err| err.into_owned())
						.collect(),
				},
				path: queued,
			};

			for elem in fptree.root.children() {
				let Some(node) = elem.into_node() else { continue; };

				if node.kind() != Syn::IncludeDirective.into() {
					continue;
				}

				let string = node.children().last().unwrap().into_token().unwrap();

				debug_assert_eq!(string.kind(), Syn::StringLit.into());

				let text = string.text();

				if !text.is_empty() {
					queue.push_back(PathBuf::from(&text[1..(text.len() - 1)]));
				} else {
					queue.push_back(PathBuf::default());
				}
			}

			all_files.push(fptree);
		}

		Ok(Self {
			files: all_files,
			missing,
		})
	}

	/// Like [`Self::new`] but taking advantage of [`rayon`]'s global thread pool.
	#[cfg(feature = "parallel")]
	pub fn new_par<F, C>(
		version: Option<zdoom::Version>,
		filesystem: F,
		path: impl AsRef<Path>,
		gcache: Option<C>,
	) -> Result<Self, Self>
	where
		F: Send + Sync + Fn(&Path) -> Option<Cow<str>>,
		C: Send + Sync + GreenCache,
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
					let Some(queued) = queue.pop() else { return; };

					let source = match filesystem(&queued) {
						Some(s) => s,
						None => {
							missing.lock().push(queued);
							return;
						}
					};

					let parser = file();
					let stream = Token::stream(source.as_ref(), version);
					let ptree = crate::parse(
						parser,
						gcache.clone(),
						Syn::Root.into(),
						source.as_ref(),
						stream,
					);

					let fptree = FileParseTree {
						inner: ParseTree {
							root: ptree.root,
							errors: ptree
								.errors
								.into_iter()
								.map(|err| err.into_owned())
								.collect(),
						},
						path: queued,
					};

					for elem in fptree.root.children() {
						let Some(node) = elem.into_node() else { continue; };

						if node.kind() != Syn::IncludeDirective.into() {
							continue;
						}

						let string = node.children().last().unwrap().into_token().unwrap();

						debug_assert_eq!(string.kind(), Syn::StringLit.into());

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

		Ok(Self {
			files: all_files.into_inner(),
			missing: missing.into_inner(),
		})
	}
}

#[cfg(test)]
mod test {
	use crate::{
		util::{builder::GreenCacheNoop, testing::*},
		zdoom::Version,
	};

	use super::*;

	const SOURCE_A: &str = r#"
#include "file/b.dec"
"#;

	const SOURCE_B: &str = r#"
#include "file/c.dec"
"#;

	const SOURCE_C: &str = r#"
actor BaronsBanquet {}
"#;

	fn lookup(path: &Path) -> Option<Cow<str>> {
		if path == Path::new("file/a.dec") {
			Some(Cow::Owned(SOURCE_A.to_string()))
		} else if path == Path::new("file/b.dec") {
			Some(Cow::Owned(SOURCE_B.to_string()))
		} else if path == Path::new("file/c.dec") {
			Some(Cow::Owned(SOURCE_C.to_string()))
		} else {
			None
		}
	}

	#[test]
	fn smoke_include_tree() {
		let _ = IncludeTree::new(None, lookup, "file/a.dec", Some(GreenCacheNoop)).unwrap();
	}

	#[test]
	fn smoke_include_tree_par() {
		let _ = IncludeTree::new_par(None, lookup, "file/a.dec", Some(GreenCacheNoop)).unwrap();
	}

	#[test]
	fn with_sample_data() {
		const ENV_VAR: &str = "DOOMFRONT_DECORATE_SAMPLE";

		let root_path = match std::env::var(ENV_VAR) {
			Ok(v) => PathBuf::from(v),
			Err(_) => {
				eprintln!("Environment variable not set: `{ENV_VAR}`.");
				return;
			}
		};

		if !root_path.exists() {
			eprintln!(
				"Path passed via `{ENV_VAR}` does not exist: {}",
				root_path.display()
			);
			return;
		}

		let inctree = IncludeTree::new(
			Some(Version::V1_0_0),
			|path: &Path| -> Option<Cow<str>> {
				let p = if let Some(parent) = root_path.parent() {
					parent.join(path)
				} else {
					return None;
				};

				if !p.exists() {
					return None;
				}

				let bytes = std::fs::read(p)
					.map_err(|err| panic!("File I/O failure: {err}"))
					.unwrap();
				let source = String::from_utf8_lossy(&bytes);
				Some(Cow::Owned(source.as_ref().to_owned()))
			},
			&root_path,
			Some(GreenCacheNoop),
		)
		.unwrap();

		for ptree in inctree.files {
			eprintln!("Checking `{}`...", ptree.path().display());
			assert_no_errors(&ptree);
		}
	}
}
