use std::path::Path;

use parking_lot::Mutex;
use petgraph::{
	graph::{DefaultIx, NodeIndex},
	prelude::DiGraph,
};
use rayon::prelude::*;

use crate::{compile::baselib, parse, Error, LexContext, ParseTree};

pub type FileIx = petgraph::graph::NodeIndex<DefaultIx>;

#[derive(Debug)]
pub struct FileTree {
	/// Edges run from parents ([`Node::Folder`]) to children ([`Node::File`]).
	/// An invalid graph is always safe but will cause unexpected compiler errors
	/// during import resolution.
	pub(crate) graph: DiGraph<Node, (), DefaultIx>,
}

#[derive(Debug)]
pub enum Node {
	File { ptree: ParseTree, path: String },
	Folder { path: String },
	Root,
}

impl Node {
	#[must_use]
	pub fn path(&self) -> &str {
		match self {
			Self::File { path, .. } => path.as_str(),
			Self::Folder { path } => path.as_str(),
			Self::Root => "/",
		}
	}

	#[must_use]
	pub fn is_file(&self) -> bool {
		matches!(self, Self::File { .. })
	}
}

impl FileTree {
	/// Returns `true` if none of the [parse trees](ParseTree) in this file tree
	/// have any errors associated with them.
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

	#[must_use]
	pub fn get(&self, ix: FileIx) -> Option<&Node> {
		self.graph.node_weight(ix)
	}

	#[must_use]
	pub fn root(&self) -> FileIx {
		let ret = FileIx::new(0);
		debug_assert!(matches!(&self.graph[ret], &Node::Root));
		ret
	}

	#[must_use]
	pub fn parent_of(&self, index: NodeIndex) -> Option<NodeIndex> {
		let mut neighbors = self
			.graph
			.neighbors_directed(index, petgraph::Direction::Incoming);

		let ret = neighbors.next();

		debug_assert!(
			neighbors.next().is_none(),
			"`FileTree` node has more than one parent"
		);

		ret
	}

	#[must_use]
	pub fn find_child(&self, index: NodeIndex, name: &str) -> Option<NodeIndex> {
		for child in self
			.graph
			.neighbors_directed(index, petgraph::Direction::Outgoing)
		{
			let ftn = &self.graph[child];

			match ftn {
				Node::File { path, .. } => {
					let path = Path::new(path);

					if path.file_stem().is_some_and(|fstem| fstem == name) {
						return Some(child);
					}
				}
				Node::Folder { path } => {
					let path = Path::new(path);

					if path.file_name().is_some_and(|fname| fname == name) {
						return Some(child);
					}
				}
				Node::Root => unreachable!(),
			};
		}

		None
	}

	pub fn reset(&mut self) {
		#[must_use]
		fn parse_baselib_file(text: &'static str) -> ParseTree {
			let ptree = doomfront::parse(text, parse::file, LexContext::default());
			debug_assert!(!ptree.any_errors());
			ptree
		}

		self.graph.clear();
		self.graph.add_node(Node::Root);

		let folder_ix = self.graph.add_node(Node::Folder {
			path: "lith".to_string(),
		});
		self.graph.add_edge(self.root(), folder_ix, ());

		let builtins_ix = self.graph.add_node(Node::File {
			ptree: parse_baselib_file(baselib::BUILTINS),
			path: "builtins.lith".to_string(),
		});
		self.graph.add_edge(folder_ix, builtins_ix, ());

		let primitive_ix = self.graph.add_node(Node::File {
			ptree: parse_baselib_file(baselib::PRIMITIVE),
			path: "primitive.lith".to_string(),
		});
		self.graph.add_edge(folder_ix, primitive_ix, ());
	}

	#[must_use]
	pub fn add_folder(&mut self, parent: FileIx, name: &str) -> FileIx {
		assert!(!name.eq_ignore_ascii_case("lith") && !name.eq_ignore_ascii_case("lithica"));

		let parent_node = &self.graph[parent];

		let base_path = match parent_node {
			Node::Folder { path } => path,
			Node::Root => "/",
			Node::File { .. } => panic!(),
		};

		let full_path = base_path.to_owned() + "/" + name;
		let ix = self.graph.add_node(Node::Folder { path: full_path });
		self.graph.add_edge(parent, ix, ());
		ix
	}

	#[must_use]
	pub fn add_file(&mut self, parent: FileIx, name: &str, text: &str) -> FileIx {
		assert!(name
			.split('.')
			.last()
			.is_some_and(|ext| ext.eq_ignore_ascii_case("lith")));

		let ptree = doomfront::parse(text, parse::file, LexContext::default());

		let parent_node = &self.graph[parent];

		let Node::Folder { path } = parent_node else {
			panic!()
		};

		let full_path = path.to_owned() + "/" + name;
		let ix = self.graph.add_node(Node::File {
			path: full_path,
			ptree,
		});
		self.graph.add_edge(parent, ix, ());
		ix
	}

	/// `base` can be a directory or file.
	/// Blocks on the [`rayon`] global thread pool for parallelized parsing.
	pub fn add_from_fs(&mut self, base: &Path) -> Result<FileIx, Vec<Error>> {
		if !base.is_dir() {
			unimplemented!();
		}

		let parent = self.root();
		let files = Mutex::new(self);
		let errors = Mutex::default();
		let option = Self::add_from_fs_recur(&files, &errors, base, parent);
		let errors = errors.into_inner();

		match option {
			Some(base_ix) => {
				if errors.is_empty() {
					Ok(base_ix)
				} else {
					Err(errors)
				}
			}
			None => Err(errors),
		}
	}

	#[must_use]
	fn add_from_fs_recur(
		this: &Mutex<&mut Self>,
		errors: &Mutex<Vec<Error>>,
		dir: &Path,
		parent: FileIx,
	) -> Option<FileIx> {
		let dir_iter = match std::fs::read_dir(dir) {
			Ok(d_i) => d_i,
			Err(err) => {
				errors.lock().push(Error::ReadDir(err));
				return None;
			}
		};

		let dir_ix = {
			let mut guard = this.lock();

			let dir_ix = guard.graph.add_node(Node::Folder {
				path: dir.to_string_lossy().into_owned(),
			});

			guard.graph.add_edge(parent, dir_ix, ());

			dir_ix
		};

		dir_iter.par_bridge().for_each(|result| {
			let dir_entry = match result {
				Ok(d_e) => d_e,
				Err(err) => {
					errors.lock().push(Error::ReadFile(err));
					return;
				}
			};

			let path = dir_entry.path();

			if path.is_dir() {
				let _ = Self::add_from_fs_recur(this, errors, &path, dir_ix);
				return;
			}

			if !path
				.extension()
				.is_some_and(|ext| ext.eq_ignore_ascii_case("lith"))
			{
				return;
			}

			let bytes = match std::fs::read(&path) {
				Ok(b) => b,
				Err(err) => {
					errors.lock().push(Error::ReadFile(err));
					return;
				}
			};

			let text = match String::from_utf8(bytes) {
				Ok(t) => t,
				Err(err) => {
					errors.lock().push(Error::FromUtf8(err));
					return;
				}
			};

			let ptree = doomfront::parse(&text, parse::file, LexContext::default());

			let mut guard = this.lock();

			let file_ix = guard.graph.add_node(Node::File {
				ptree,
				path: path.to_string_lossy().into_owned(),
			});

			guard.graph.add_edge(dir_ix, file_ix, ());
		});

		Some(dir_ix)
	}
}

impl Default for FileTree {
	fn default() -> Self {
		let mut ret = Self {
			graph: DiGraph::default(),
		};

		ret.reset();

		ret
	}
}
