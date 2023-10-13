use std::path::Path;

use parking_lot::Mutex;
use petgraph::prelude::DiGraph;
use rayon::prelude::*;

use crate::{parse, ParseTree};

pub type FileIx = petgraph::graph::DefaultIx;

#[derive(Debug)]
pub struct FileTree {
	/// Edges run from parents ([`Node::Folder`]) to children ([`Node::File`]).
	/// An invalid graph is always safe but will cause unexpected compiler errors.
	pub graph: DiGraph<Node, (), FileIx>,
}

#[derive(Debug)]
pub enum Node {
	File { ptree: ParseTree, path: String },
	Folder { path: String },
}

impl FileTree {
	#[must_use]
	pub fn valid(&self) -> bool {
		for node in self.graph.node_weights() {
			let Node::File { ptree, .. } = node else {
				continue;
			};

			if ptree.any_errors() {
				return false;
			}
		}

		true
	}

	pub fn files(&self) -> impl Iterator<Item = (&String, &ParseTree)> {
		self.graph.node_weights().filter_map(|ftn| {
			let Node::File { ptree, path } = ftn else {
				return None;
			};

			Some((path, ptree))
		})
	}

	/// `root` can be a directory or file.
	/// Blocks on the [`rayon`] global thread pool for parallelized parsing.
	pub fn from_fs(root: &Path) -> std::io::Result<Self> {
		if !root.is_dir() {
			unimplemented!()
		}

		let files = Mutex::default();
		Self::from_fs_recur(&files, root);

		Ok(Self {
			graph: files.into_inner(),
		})
	}

	fn from_fs_recur(files: &Mutex<DiGraph<Node, (), FileIx>>, dir: &Path) {
		let Ok(dir_iter) = std::fs::read_dir(dir) else {
			// TODO: accumulate and return errors.
			return;
		};

		let dir_ix = files.lock().add_node(Node::Folder {
			path: dir.to_string_lossy().into_owned(),
		});

		dir_iter.par_bridge().for_each(|result| {
			let Ok(dir_entry) = result else {
				// TODO: accumulate and return errors.
				return;
			};

			let path = dir_entry.path();

			if path.is_dir() {
				Self::from_fs_recur(files, &path);
				return;
			}

			if !path
				.extension()
				.is_some_and(|ext| ext.eq_ignore_ascii_case("lith"))
			{
				return;
			}

			let Ok(bytes) = std::fs::read(&path) else {
				// TODO: accumulate and return errors.
				return;
			};

			let Ok(text) = String::from_utf8(bytes) else {
				// TODO: accumulate and return errors.
				return;
			};

			let ptree = doomfront::parse(&text, parse::file, ());

			let mut guard = files.lock();

			let file_ix = guard.add_node(Node::File {
				ptree,
				path: path.to_string_lossy().into_owned(),
			});

			guard.add_edge(dir_ix, file_ix, ());
		});
	}
}
