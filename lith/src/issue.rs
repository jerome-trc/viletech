//! Types for reporting compiler-emitted diagnostics not related to parsing.

use std::borrow::Cow;

use ariadne::ReportKind;
use doomfront::rowan::TextRange;
use smallvec::SmallVec;

#[derive(Debug)]
pub struct Issue {
	pub id: FileSpan,
	pub level: Level,
	pub message: Cow<'static, str>,
	pub labels: SmallVec<[Label; 1]>,
	pub notes: SmallVec<[Cow<'static, str>; 1]>,
}

impl Issue {
	#[must_use]
	pub fn new(path: impl AsRef<str>, span: TextRange, level: Level) -> Self {
		Self {
			id: FileSpan {
				path: path.as_ref().to_owned(),
				span,
			},
			level,
			message: Cow::Borrowed(""),
			labels: smallvec::smallvec![],
			notes: smallvec::smallvec![],
		}
	}

	#[must_use]
	pub fn with_message(mut self, message: String) -> Self {
		self.message = Cow::Owned(message);
		self
	}

	#[must_use]
	pub fn with_message_static(mut self, message: &'static str) -> Self {
		self.message = Cow::Borrowed(message);
		self
	}

	#[must_use]
	pub fn with_label(mut self, path: impl AsRef<str>, span: TextRange, message: String) -> Self {
		self.labels.push(Label {
			id: FileSpan {
				path: path.as_ref().to_owned(),
				span,
			},
			message: Cow::Owned(message),
		});

		self
	}

	#[must_use]
	pub fn with_label_static(
		mut self,
		path: impl AsRef<str>,
		span: TextRange,
		message: &'static str,
	) -> Self {
		self.labels.push(Label {
			id: FileSpan {
				path: path.as_ref().to_owned(),
				span,
			},
			message: Cow::Borrowed(message),
		});

		self
	}

	#[must_use]
	pub fn with_note(mut self, message: String) -> Self {
		self.notes.push(Cow::Owned(message));
		self
	}

	#[must_use]
	pub fn with_note_static(mut self, message: &'static str) -> Self {
		self.notes.push(Cow::Borrowed(message));
		self
	}

	#[must_use]
	pub fn is_error(&self) -> bool {
		matches!(self.level, Level::Error(_) | Level::Deny(_))
	}

	#[must_use]
	pub fn report(self) -> Report {
		let mut colorgen = ariadne::ColorGenerator::default();

		let (kind, code) = match self.level {
			Level::Error(err) => (ReportKind::Error, err as u16),
			Level::Deny(lint) => (ReportKind::Error, lint as u16),
			Level::Warn(lint) => (ReportKind::Warning, lint as u16),
			Level::Suggest(lint) => (ReportKind::Advice, lint as u16),
		};

		let mut builder = Report::build(kind, self.id.path, 12)
			.with_code(code)
			.with_message(self.message);

		for label in self.labels {
			builder = builder.with_label(
				ariadne::Label::new(label.id)
					.with_color(colorgen.next())
					.with_message(label.message),
			);
		}

		for note in self.notes {
			builder = builder.with_note(note)
		}

		builder.finish()
	}
}

impl std::error::Error for Issue {}

impl std::fmt::Display for Issue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.level {
			Level::Error(iss) => iss.fmt(f),
			Level::Deny(iss) => iss.fmt(f),
			Level::Warn(iss) => iss.fmt(f),
			Level::Suggest(iss) => iss.fmt(f),
		}
	}
}

/// See [`Issue`].
#[derive(Debug)]
pub struct Label {
	pub id: FileSpan,
	pub message: Cow<'static, str>,
}

/// See [`Issue`].
#[derive(Debug)]
pub enum Level {
	Error(Error),
	Deny(Lint),
	Warn(Lint),
	Suggest(Lint),
}

/// Code numbers for diagnostics on Lithica that the compiler can never accept.
///
/// Also see [`Issue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Error {
	/// An annotation expected one value from a specific set of possible values
	/// for one of its arguments, but received something else.
	AnnotationArg,
	/// An annotation was used in an invalid position, e.g. `#[inline]`
	/// on an AST element that it is not a function declaration.
	AnnotationUsage,
	/// Wrong number of arguments passed to a function.
	ArgCount,
	/// Mismatch between argument and parameter types.
	ArgType,
	BinExprTypeMismatch,
	BuiltinMisuse,
	CEvalRecursion,
	ConstEval,
	/// Declared a symbolic constant or static variable with the type specifier `any_t`.
	ContainerValAnyType,
	FlagDefBitOverflow,
	FolderImport,
	/// A named argument was passed to an annotation that cannot accept names
	/// on any of its arguments.
	IllegalArgName,
	IllegalClassQual,
	IllegalConstInit,
	IllegalFnQual,
	/// A function marked `native` has a body block.
	IllegalFnBody,
	IllegalStructQual,
	/// A non-native library attempted to declare a symbol starting or ending
	/// with `__`, which is reserved for internal/native use.
	IllegalSymbolName,
	ImportPath,
	/// e.g. attempt to implicitly narrow,
	/// or to use a literal suffix which would narrow the literal's value.
	IntConvert,
	/// Something went wrong with the compiler itself. The problem was either
	/// in Rust, or an ill-formed native declaration in a script.
	Internal,
	/// An annotation was passed an anonymous argument that it expected to be named.
	MissingArgName,
	/// A function not marked `native` or `builtin` has no body block.
	MissingFnBody,
	/// An import entry using a name literal is missing its required rename identifier.
	MissingImportRename,
	MissingNative,
	/// A non-native library tried to use the `builtin` or `native` annotation.
	NonNative,
	ParseFloat,
	ParseInt,
	QualifierOverlap,
	Redeclare,
	SelfImport,
	SymbolKindMismatch,
	SymbolNotFound,
	UnknownAnnotation,
	UnknownExprType,
}

impl std::fmt::Display for Error {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

/// Code numbers for diagnostics which are not fatal unless user requests that they be.
///
/// For example, warnings about potentially incorrect or sub-optimal code go here,
/// but so do suggestions about conformance to the official style guidelines.
///
/// Also see [`Issue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Lint {
	/// i.e. code like `if x == true {}` or `if x == false {}`.
	BoolCompare,
	UnusedReturnValue,
}

impl std::fmt::Display for Lint {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

/// See [`Issue`].
#[derive(Debug)]
pub struct FileSpan {
	pub path: String,
	pub span: TextRange,
}

impl ariadne::Span for FileSpan {
	type SourceId = str;

	fn source(&self) -> &Self::SourceId {
		&self.path
	}

	fn start(&self) -> usize {
		self.span.start().into()
	}

	fn end(&self) -> usize {
		self.span.end().into()
	}
}

/// See [`Issue`].
pub type Report = ariadne::Report<'static, FileSpan>;
