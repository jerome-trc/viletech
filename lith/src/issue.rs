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
		matches!(self.level, Level::Error(_))
	}

	#[must_use]
	pub fn report(self) -> Report {
		let mut colorgen = ariadne::ColorGenerator::default();

		let (kind, code) = match self.level {
			Level::Error(err) => (ReportKind::Error, err as u16),
			Level::Warning(warn) => (ReportKind::Warning, warn as u16),
			Level::Lint(lint) => (ReportKind::Advice, lint as u16),
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
			Level::Warning(iss) => iss.fmt(f),
			Level::Lint(iss) => iss.fmt(f),
		}
	}
}

#[derive(Debug)]
pub struct Label {
	pub id: FileSpan,
	pub message: Cow<'static, str>,
}

#[derive(Debug)]
pub enum Level {
	Error(Error),
	Warning(Warning),
	Lint(Lint),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Error {
	/// Wrong number of arguments passed to a function.
	ArgCount,
	/// Mismatch between argument and parameter types.
	ArgType,
	BinExprTypeMismatch,
	/// An argument passed to a compiler built-in caused an error that otherwise
	/// falls under no other error code.
	Builtin,
	CEvalRecursion,
	/// e.g., ZScript tried to use `super` at compile time.
	ConstEval,
	FlagDefBitOverflow,
	IllegalClassQual,
	IllegalConstInit,
	IllegalFnQual,
	IllegalStructQual,
	/// e.g. attempt to implicitly narrow,
	/// or to use a literal suffix which would narrow the literal's value.
	IntConvert,
	/// Something went wrong with the compiler itself. The problem was either
	/// in Rust, or an ill-formed native declaration in a script.
	Internal,
	ParseFloat,
	ParseInt,
	/// e.g. script marked a ZS class as `play` twice.
	QualifierOverlap,
	Redeclare,
	/// e.g. script defines a class specifying inheritance from a struct.
	SymbolKindMismatch,
	SymbolNotFound,
	/// e.g. a null literal was provided in an ambiguous context.
	UnknownExprType,
}

impl std::fmt::Display for Error {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Warning {
	UnusedRetVal,
}

impl std::fmt::Display for Warning {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Lint {
	/// i.e. code like `if x == true {}` or `if x == false {}`.
	BoolCompare,
}

impl std::fmt::Display for Lint {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

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

pub type Report = ariadne::Report<'static, FileSpan>;
