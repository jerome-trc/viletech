use std::fmt;

#[derive(Debug)]
pub enum Error {
	AdapterRequest,
	NoSurfaceFormat,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::AdapterRequest => write!(f, "Failed to retrieve an adapter."),
			Error::NoSurfaceFormat => write!(
				f,
				"None of this system's supported texture formats are suitable for surface use."
			),
		}
	}
}
