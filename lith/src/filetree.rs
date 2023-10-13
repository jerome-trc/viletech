use petgraph::prelude::DiGraph;

use crate::ParseTree;

pub type FileIx = petgraph::graph::DefaultIx;

#[derive(Debug)]
pub struct FileTree {
	/// Edges run from parents ([`Node::Folder`]) to children ([`Node::File`]).
	/// An invalid graph is always safe but will cause unexpected compiler errors.
	pub files: DiGraph<Node, (), FileIx>,
}

#[derive(Debug)]
pub enum Node {
	File { ptree: ParseTree, path: String },
	Folder { path: String },
}

impl FileTree {
	#[must_use]
	pub fn valid(&self) -> bool {
		for node in self.files.node_weights() {
			let Node::File { ptree, .. } = node else {
				continue;
			};

			if ptree.any_errors() {
				return false;
			}
		}

		true
	}
}
