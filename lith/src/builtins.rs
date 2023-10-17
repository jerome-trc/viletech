use crate::runtime;

/// Compiler intrinsic functions.
///
/// Some are runtime-only, some are const-eval only, some support both.

/// Returns the total memory used by the garbage collector.
pub(crate) extern "C" fn gc_usage(_: *mut runtime::Context) -> usize {
	// TODO: just a dummy function for proof-of-concept purposes at the moment.
	123_456_789
}

// All constants below are used in `UserExternalName::index`.
pub(crate) const CLIX_GCUSAGE: u32 = 0;
