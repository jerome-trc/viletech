//! Errors, warnings, or lints emitted during compilation.

//! An error, warning, or lint emitted during compilation.

use ariadne::ReportKind;
use doomfront::rowan::TextRange;

#[derive(Debug)]
pub struct FileSpan {
	pub path: String,
	pub span: logos::Span,
}

impl FileSpan {
	#[must_use]
	pub fn new(path: impl AsRef<str>, tr: TextRange) -> Self {
		Self {
			path: path.as_ref().to_owned(),
			span: tr.start().into()..tr.end().into(),
		}
	}
}

impl ariadne::Span for FileSpan {
	type SourceId = String;

	fn source(&self) -> &Self::SourceId {
		&self.path
	}

	fn start(&self) -> usize {
		self.span.start
	}

	fn end(&self) -> usize {
		self.span.end
	}
}

#[derive(Debug)]
pub struct Issue {
	pub id: FileSpan,
	pub level: IssueLevel,
	pub label: Option<Label>,
}

impl Issue {
	#[must_use]
	pub fn is_error(&self) -> bool {
		matches!(self.level, IssueLevel::Error(_))
	}

	#[must_use]
	pub fn report(self) -> Report {
		let mut colorgen = ariadne::ColorGenerator::default();

		let (kind, code) = match self.level {
			IssueLevel::Error(err) => (ReportKind::Error, err as u16),
			IssueLevel::Warning(warn) => (ReportKind::Warning, warn as u16),
			IssueLevel::Lint(lint) => (ReportKind::Advice, lint as u16),
		};

		let builder = Report::build(kind, self.id.path, 12).with_code(code);

		let builder = if let Some(label) = self.label {
			builder.with_label(
				ariadne::Label::new(label.id)
					.with_color(colorgen.next())
					.with_message(label.message),
			)
		} else {
			builder
		};

		builder.finish()
	}
}

impl std::error::Error for Issue {}

impl std::fmt::Display for Issue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.level {
			IssueLevel::Error(iss) => iss.fmt(f),
			IssueLevel::Warning(iss) => iss.fmt(f),
			IssueLevel::Lint(iss) => iss.fmt(f),
		}
	}
}

#[derive(Debug)]
pub struct Label {
	pub id: FileSpan,
	pub message: String,
}

impl Label {
	#[must_use]
	pub fn new(path: impl AsRef<str>, tr: TextRange, message: String) -> Self {
		Self {
			id: FileSpan::new(path, tr),
			message,
		}
	}
}

#[derive(Debug)]
pub enum IssueLevel {
	Error(Error),
	Warning(Warning),
	Lint(Lint),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Error {
	IllegalConstInit,
	IllegalFnQual,
	IllegalStructQual,
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

pub type Report = ariadne::Report<'static, FileSpan>;
