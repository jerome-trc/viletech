use std::{
	borrow::Cow,
	io::Read,
	path::{Path, PathBuf},
	str::FromStr,
	string::FromUtf8Error,
	sync::OnceLock,
};

use crossbeam::queue::SegQueue;
use doomfront::{
	rowan::{ast::AstNode, cursor::SyntaxNode, GreenNodeData, Language, NodeOrToken, TextRange},
	zdoom::{self, zscript},
	ParseTree,
};
use parking_lot::Mutex;
use rayon::prelude::*;
use regex::Regex;
use util::{lazy_regex, path::PathExt};
use vfs::VirtualFs;

use crate::{ast, FxDashSet, Syn, Version};

#[derive(Debug, Default)]
pub struct IncludeTree {
	pub(crate) files: Vec<ParsedFile>,
	pub(crate) errors: Vec<Error>,
}

impl IncludeTree {
	/// Traverses an include tree, starting from a VFS root path.
	/// Note that this blocks the [`rayon`] global thread pool.
	#[must_use]
	pub fn new(fref: vfs::FileRef, vzs_version: Option<Version>) -> Self {
		let vfs = fref.vfs();
		let pfiles = Mutex::new(vec![]);
		let errors = Mutex::new(vec![]);

		let walker = VfsWalker {
			vfs,
			pfiles: &pfiles,
			errors: &errors,
			vzs_version,
			zs_version: None,
		};

		if fref
			.path_extension()
			.is_some_and(|s| s.eq_ignore_ascii_case("vzs"))
		{
			walker.recur_vzs(fref);
		} else {
			walker.zs_root(fref);
		};

		Self {
			files: pfiles.into_inner(),
			errors: errors.into_inner(),
		}
	}

	/// `base_path` should be something like `/home/janedoe/Data/viletech/assets`.
	/// `root_path` should be something like `/vzscript/main.vzs`.
	#[must_use]
	pub fn from_fs(base_path: &Path, root_path: &Path, vzs_version: Option<Version>) -> Self {
		let pfiles = Mutex::new(vec![]);
		let errors = Mutex::new(vec![]);
		let full_path = base_path.join(root_path);

		let fd = match std::fs::File::open(&full_path) {
			Ok(f) => f,
			Err(err) => {
				return Self {
					files: vec![],
					errors: vec![Error::Io {
						path: full_path,
						error: err,
					}],
				}
			}
		};

		let walker = FsWalker {
			base_path,
			pfiles: &pfiles,
			errors: &errors,
			vzs_version,
			zs_version: None,
		};

		if root_path.extension_is("vzs") {
			walker.recur_vzs(fd, full_path);
		} else {
			walker.recur_zs(fd, full_path);
		}

		Self {
			files: pfiles.into_inner(),
			errors: errors.into_inner(),
		}
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		!self.errors.is_empty() || self.files.iter().any(|fptree| fptree.any_errors())
	}
}

#[derive(Debug)]
pub struct ParsedFile {
	pub(crate) inner: SourceKind,
	pub(crate) path: String,
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

#[derive(Debug)]
pub enum Error {
	/// Include tree walking found `#include("")` (VZS) or `#include ""` (ZS).
	EmptyPath { file: String, span: TextRange },
	/// An `#include` annotation has no argument list, or its argument list
	/// contains something other than exactly one string literal.
	IncludeArgs { file: String, span: TextRange },
	/// A failure to read from the physical file system rather than the VFS.
	Io {
		path: PathBuf,
		error: std::io::Error,
	},
	/// `file` tried to include `inc_path`, but the latter failed to resolve.
	Missing {
		file: String,
		inc_path: String,
		span: TextRange,
	},
	/// The file at `path` is not valid Unicode.
	InvalidUtf8 { path: String },
	/// The file at `path` is a ZSCRIPT lump but has no version directive,
	/// or the version directive is malformed.
	ZsVersion { path: String },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EmptyPath { file, span } => {
				write!(f, "{file}:{span:?} has an empty include path")
			}
			Self::IncludeArgs { file, span } => {
				write!(f, "`#include` annotation does not have exactly one string argument ({file}:{span:?})")
			}
			Self::Io { path, error } => {
				write!(f, "failed to read {p}: {error}", p = path.display())
			}
			Self::Missing {
				file,
				inc_path,
				span,
			} => {
				write!(
					f,
					"included file could not be found: {inc_path} ({file}:{span:?})"
				)
			}
			Self::InvalidUtf8 { path } => {
				write!(f, "file is not valid UTF-8: {path}")
			}
			Self::ZsVersion { path } => {
				write!(f, "ZSCRIPT lump has a missing or malformed version: {path}")
			}
		}
	}
}

#[derive(Debug, Clone, Copy)]
struct VfsWalker<'a> {
	vfs: &'a VirtualFs,
	pfiles: &'a Mutex<Vec<ParsedFile>>,
	errors: &'a Mutex<Vec<Error>>,
	vzs_version: Option<Version>,
	zs_version: Option<zdoom::Version>,
}

impl VfsWalker<'_> {
	fn recur_vzs(self, fref: vfs::FileRef) {
		let Ok(source) = fref.try_read_str() else {
			self.errors.lock().push(Error::InvalidUtf8 { path: fref.path_str().to_owned() });
			return;
		};

		let ptree = parse_vzs(
			source,
			self.vzs_version.unwrap(),
			|node, inc_path| {
				self.converge(fref, node, inc_path);
			},
			|span| {
				self.errors.lock().push(Error::IncludeArgs {
					file: fref.path_str().to_owned(),
					span,
				})
			},
		);

		self.pfiles.lock().push(ParsedFile {
			inner: SourceKind::Vzs(ptree),
			path: fref.path_str().to_owned(),
		});
	}

	fn zs_root(mut self, fref: vfs::FileRef) {
		let Ok(source) = fref.try_read_str() else {
			self.errors.lock().push(Error::InvalidUtf8 { path: fref.path_str().to_owned() });
			return;
		};

		let rgx = ZS_VERSION_RGX.get_or_init(zs_version_regex_init);

		let Some(vers) = rgx.find(source) else {
			self.errors.lock().push(Error::ZsVersion {
				path: fref.path_str().to_owned(),
			});

			return;
		};

		let Ok(vers) = zdoom::Version::from_str(vers.as_str()) else {
			self.errors.lock().push(Error::ZsVersion {
				path: fref.path_str().to_owned(),
			});

			return;
		};

		self.zs_version = Some(vers);

		let ptree = parse_zs(source, self.zs_version.unwrap(), |node, inc_path| {
			self.converge(fref, node, inc_path);
		});

		self.pfiles.lock().push(ParsedFile {
			inner: SourceKind::Zs(ptree),
			path: fref.path_str().to_owned(),
		});
	}

	fn recur_zs(self, fref: vfs::FileRef) {
		let Ok(source) = fref.try_read_str() else {
			self.errors.lock().push(Error::InvalidUtf8 { path: fref.path_str().to_owned() });
			return;
		};

		let ptree = parse_zs(source, self.zs_version.unwrap(), |node, inc_path| {
			self.converge(fref, node, inc_path);
		});

		self.pfiles.lock().push(ParsedFile {
			inner: SourceKind::Zs(ptree),
			path: fref.path_str().to_owned(),
		});
	}

	fn converge(self, fref: vfs::FileRef, node: &GreenNodeData, inc_path: &str) {
		if inc_path.is_empty() {
			let syn_node = SyntaxNode::new_root(node.to_owned());

			self.errors.lock().push(Error::EmptyPath {
				file: fref.path_str().to_owned(),
				span: syn_node.text_range(),
			});

			return;
		}

		let Some(next_fref) = self.vfs.get(inc_path) else {
			let syn_node = SyntaxNode::new_root(node.to_owned());

			self.errors.lock().push(
				Error::Missing {
					file: fref.path_str().to_owned(),
					inc_path: inc_path.to_string(),
					span: syn_node.text_range()
				}
			);

			return;
		};

		if Path::new(inc_path).extension_is("vzs") {
			self.recur_vzs(next_fref);
		} else {
			self.recur_zs(next_fref);
		}
	}
}

#[derive(Debug, Clone, Copy)]
struct FsWalker<'a> {
	base_path: &'a Path,
	pfiles: &'a Mutex<Vec<ParsedFile>>,
	errors: &'a Mutex<Vec<Error>>,
	vzs_version: Option<Version>,
	zs_version: Option<zdoom::Version>,
}

impl FsWalker<'_> {
	fn recur_vzs(self, mut fd: std::fs::File, full_path: PathBuf) {
		let path_cow = full_path.to_string_lossy();
		let mut buf = String::with_capacity(fd.metadata().map(|md| md.len() as usize).unwrap_or(0));

		let Ok(_) = fd.read_to_string(&mut buf) else {
			self.errors.lock().push(Error::InvalidUtf8 { path: path_cow.into_owned() });
			return;
		};

		let ptree = parse_vzs(
			&buf,
			self.vzs_version.unwrap(),
			|node, inc_path| {
				self.converge(path_cow.clone(), node, inc_path);
			},
			|span| {
				self.errors.lock().push(Error::IncludeArgs {
					file: path_cow.clone().into_owned(),
					span,
				})
			},
		);

		self.pfiles.lock().push(ParsedFile {
			inner: SourceKind::Vzs(ptree),
			path: path_cow.into_owned(),
		});
	}

	fn zs_root(mut self, mut fd: std::fs::File, full_path: PathBuf) {
		let path_cow = full_path.to_string_lossy();

		let mut buf = String::with_capacity(fd.metadata().map(|md| md.len() as usize).unwrap_or(0));

		let Ok(_) = fd.read_to_string(&mut buf) else {
			self.errors.lock().push(Error::InvalidUtf8 { path: path_cow.into_owned() });
			return;
		};

		let rgx = ZS_VERSION_RGX.get_or_init(zs_version_regex_init);

		let Some(vers) = rgx.find(&buf) else {
			self.errors.lock().push(Error::ZsVersion {
				path: path_cow.into_owned(),
			});

			return;
		};

		let Ok(vers) = zdoom::Version::from_str(vers.as_str()) else {
			self.errors.lock().push(Error::ZsVersion {
				path: path_cow.into_owned(),
			});

			return;
		};

		self.zs_version = Some(vers);

		let ptree = parse_zs(&buf, self.zs_version.unwrap(), |node, inc_path| {
			self.converge(path_cow.clone(), node, inc_path);
		});

		self.pfiles.lock().push(ParsedFile {
			inner: SourceKind::Zs(ptree),
			path: path_cow.into_owned(),
		});
	}

	fn recur_zs(self, mut fd: std::fs::File, full_path: PathBuf) {
		let path_cow = full_path.to_string_lossy();
		let mut buf = String::with_capacity(fd.metadata().map(|md| md.len() as usize).unwrap_or(0));

		let Ok(_) = fd.read_to_string(&mut buf) else {
			self.errors.lock().push(Error::InvalidUtf8 { path: path_cow.into_owned() });
			return;
		};

		let ptree = parse_zs(&buf, self.zs_version.unwrap(), |node, inc_path| {
			self.converge(path_cow.clone(), node, inc_path);
		});

		self.pfiles.lock().push(ParsedFile {
			inner: SourceKind::Zs(ptree),
			path: path_cow.into_owned(),
		});
	}

	fn converge(self, file_path: Cow<str>, node: &GreenNodeData, inc_path: &str) {
		if inc_path.is_empty() {
			let syn_node = SyntaxNode::new_root(node.to_owned());

			self.errors.lock().push(Error::EmptyPath {
				file: file_path.into_owned(),
				span: syn_node.text_range(),
			});

			return;
		}

		let full_path = self.base_path.join(inc_path);

		let fd = match std::fs::File::open(&full_path) {
			Ok(f) => f,
			Err(err) => {
				let syn_node = SyntaxNode::new_root(node.to_owned());

				self.errors.lock().push(Error::Missing {
					file: file_path.into_owned(),
					inc_path: inc_path.to_string(),
					span: syn_node.text_range(),
				});

				return;
			}
		};

		if Path::new(inc_path).extension_is("vzs") {
			self.recur_vzs(fd, full_path);
		} else {
			self.recur_zs(fd, full_path);
		}
	}
}

#[must_use]
fn parse_vzs<F, E>(source: &str, version: Version, callback: F, err_handler: E) -> ParseTree<Syn>
where
	F: Fn(&GreenNodeData, &str) + Send + Sync,
	E: Fn(TextRange),
{
	let ptree = doomfront::parse(source, crate::parse::file, version);

	for top in ptree.cursor().children().filter_map(ast::TopLevel::cast) {
		let ast::TopLevel::Annotation(anno) = top else {
			continue;
		};

		if !anno.name().is_ok_and(|token| token.text() == "include") {
			continue;
		}

		let Some(arglist) = anno.arg_list() else {
			err_handler(anno.syntax().text_range());
			continue;
		};

		let mut args = arglist.iter();

		let Some(arg0) = args.next() else {
			err_handler(arglist.syntax().text_range());
			continue;
		};

		let Ok(ast::Expr::Literal(lit)) = arg0.expr() else {
			err_handler(anno.syntax().text_range());
			continue;
		};

		if let Some(arg1) = args.next() {
			err_handler(arg1.syntax().text_range());
			continue;
		}

		let token = lit.token();

		let Some(inc_path) = token.string() else {
			err_handler(token.text_range());
			continue;
		};

		callback(anno.syntax().green().as_ref(), inc_path);
	}

	ptree
}

#[must_use]
fn parse_zs<F>(source: &str, version: zdoom::Version, callback: F) -> ParseTree<zscript::Syn>
where
	F: Fn(&GreenNodeData, &str) + Send + Sync,
{
	let ptree = doomfront::parse(
		source,
		zscript::parse::file,
		zdoom::lex::Context {
			version,
			doc_comments: true,
		},
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

		let inc_path = string.text();

		callback(node, inc_path);
	}

	ptree
}

static ZS_VERSION_RGX: OnceLock<Regex> = OnceLock::new();

#[must_use]
fn zs_version_regex_init() -> Regex {
	Regex::new("(?i)version[\0- ]*\"[0-9]+\\.[0-9]+(\\.[0-9]+)?\"").unwrap()
}
