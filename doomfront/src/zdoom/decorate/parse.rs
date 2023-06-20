//! Note that the results of these parsers are likely to be incorrect if used with
//! a [`logos::Lexer`] bearing a version above [`V1_0_0`](crate::zdoom::Version::V1_0_0).

mod actor;
mod common;
mod expr;
mod top;

use std::{
	borrow::Cow,
	collections::VecDeque,
	path::{Path, PathBuf},
};

use chumsky::{primitive, recovery, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::coalesce_node,
	zdoom::{self, Token},
	GreenElement, _ParseState,
};

pub use self::{actor::*, common::*, expr::*, top::*};

use super::{ParseTree, Syn};

/// The returned parser emits a [`Syn::Root`] node.
pub fn file<'i>() -> parser_t!(GreenNode) {
	let recover_actor_inner = primitive::any()
		.filter(|token| *token == Token::Ident)
		.map_with_state(|token: Token, span: logos::Span, state: &mut _ParseState| {
			(token, &state.source[span])
		})
		.filter(|(_, text)| !text.eq_ignore_ascii_case("actor"))
		.map(|(token, text)| GreenToken::new(recover_token(token).into(), text))
		.repeated()
		.collect();

	let recover_actor = recover_general(
		comb::string_nc(Token::Ident, "actor", Syn::KwActor),
		recover_actor_inner,
		primitive::one_of([
			Token::Ident,
			Token::PoundInclude,
			Token::KwConst,
			Token::KwEnum,
		])
		.rewind()
		.ignored(),
	);

	primitive::choice((
		trivia(),
		actor_def()
			.map(GreenElement::from)
			.recover_with(recovery::via_parser(recover_actor)),
		include_directive().map(GreenElement::from),
		const_def().map(GreenElement::from),
		enum_def().map(GreenElement::from),
		damage_type_def().map(GreenElement::from),
	))
	.repeated()
	.collect::<Vec<_>>()
	.map(|group| coalesce_node(group, Syn::Root))
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
	pub fn new<F>(mut filesystem: F, path: impl AsRef<Path>) -> Result<Self, Self>
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
					missing.push(queued);
					continue;
				}
			};

			let tbuf = crate::_scan(source.as_ref(), zdoom::Version::V1_0_0);

			let ptree: ParseTree = crate::_parse(file(), source.as_ref(), &tbuf).map_or_else(
				|errors| ParseTree::new(GreenNode::new(Syn::Root.into(), []), errors),
				|ptree| ptree,
			);

			let ptree = ParseTree::new(
				ptree.root,
				ptree
					.errors
					.into_iter()
					.map(|err| err.into_owned())
					.collect(),
			);

			let fptree = FileParseTree {
				inner: ptree,
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
	pub fn new_par<F>(filesystem: F, path: impl AsRef<Path>) -> Result<Self, Self>
	where
		F: Send + Sync + Fn(&Path) -> Option<Cow<str>>,
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

					let tbuf = crate::_scan(source.as_ref(), zdoom::Version::V1_0_0);

					let ptree: ParseTree = crate::_parse(file(), source.as_ref(), &tbuf)
						.map_or_else(
							|errors| ParseTree::new(GreenNode::new(Syn::Root.into(), []), errors),
							|ptree| ptree,
						);

					let ptree = ParseTree::new(
						ptree.root,
						ptree
							.errors
							.into_iter()
							.map(|err| err.into_owned())
							.collect(),
					);

					let fptree = FileParseTree {
						inner: ptree,
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
#[cfg(any())]
mod test {
	use crate::testing::*;

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
		let _ = IncludeTree::new(lookup, "file/a.dec").unwrap();
	}

	#[test]
	fn smoke_include_tree_par() {
		let _ = IncludeTree::new_par(lookup, "file/a.dec").unwrap();
	}

	#[test]
	fn with_sample_data() {
		let (_, sample) = match read_sample_data("DOOMFRONT_DECORATE_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Skipping DECORATE sample data-based unit test. Reason: {err}");
				return;
			}
		};

		let tbuf = crate::_scan(&sample, zdoom::Version::default());
		let result = crate::_parse(file(), &sample, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
	}

	#[test]
	fn include_tree() {
		let (root_path, _) = match read_sample_data("DOOMFRONT_DECORATE_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!(
					"Skipping DECORATE sample data-based include tree unit test. Reason: {err}"
				);
				return;
			}
		};

		let Some(root_parent_path) = root_path.parent() else {
			eprintln!(
				"Skipping DECORATE sample data-based include tree unit test. Reason: `{}` has no parent.",
				root_path.display()
			);
			return;
		};

		let inctree = IncludeTree::new(
			|path: &Path| -> Option<Cow<str>> {
				let p = root_parent_path.join(path);

				if !p.exists() {
					return None;
				}

				let bytes = std::fs::read(p)
					.map_err(|err| panic!("file I/O failure: {err}"))
					.unwrap();
				let source = String::from_utf8_lossy(&bytes);
				Some(Cow::Owned(source.as_ref().to_owned()))
			},
			&root_path,
		)
		.unwrap();

		for ptree in inctree.files {
			eprintln!("Checking `{}`...", ptree.path().display());
			assert_no_errors(&ptree);
		}
	}
}
