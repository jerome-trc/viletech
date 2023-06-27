//! Errors, warnings, or lints emitted during compilation.

//! An error, warning, or lint emitted during compilation.
#[derive(Debug)]
pub struct Issue {
	level: IssueLevel,
}

impl Issue {
	#[must_use]
	pub fn is_error(&self) -> bool {
		matches!(self.level, IssueLevel::Error(_))
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
pub enum IssueLevel {
	Error(Error),
	Warning(Warning),
	Lint(Lint),
}

#[derive(Debug)]
pub enum Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

#[derive(Debug)]
pub enum Warning {}

impl std::fmt::Display for Warning {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

#[derive(Debug)]
pub enum Lint {}

impl std::fmt::Display for Lint {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}
