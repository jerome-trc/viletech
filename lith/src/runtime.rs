use crate::{rti, FxDashView};

/// Context for Lithica execution.
///
/// Fully re-entrant; Lith has no global state.
#[derive(Debug)]
pub struct Runtime {
	pub(crate) rtinfo: FxDashView<String, rti::Record>,
}

/// A pointer to a structure of this type gets weaved through all Lith calls.
#[derive(Debug)]
#[repr(C)]
pub struct Context<'r, 'u, U: Sized> {
	pub rt: &'r mut Runtime,
	pub user: &'u mut U,
}
