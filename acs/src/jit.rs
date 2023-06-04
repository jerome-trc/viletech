use std::collections::HashMap;

/// For best possible results, assume that every single initializing method
/// on this structure needs to be called; JIT compilation will panic if attempting
/// to translate an instruction which depends on a symbol that was not registered.
#[derive(Debug, Default)]
pub struct ExtSymbols(HashMap<&'static str, *mut ()>);

impl ExtSymbols {
	const NAME_FN_RANDOM: &'static str = "__fn_random__";
	const NAME_USERDATA_RANDOM: &'static str = "__userdata_random__";

	/// The given function is expected to a return value within the range
	/// `min + [0, max - min + 1)`.
	/// Note that `func` may be called with a max less than its min; it is
	/// expected to handle that case.
	pub unsafe fn random(
		mut self,
		func: unsafe extern "C" fn(userdata: *mut (), min: i32, max: i32),
		userdata: *mut u8,
	) -> Self {
		self.0.insert(Self::NAME_FN_RANDOM, func as *mut ());
		self.0.insert(Self::NAME_USERDATA_RANDOM, userdata as *mut ());
		self
	}
}
