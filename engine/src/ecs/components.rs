/// Component for data which is baked into a newly-spawned entity and never changes.
#[derive(Debug)]
pub struct Constant {
	/// The sim tic on which this entity was spawned.
	spawned_tic: u32,
}

/// Primarily for use by ACS behaviours. An entity won't have this component unless
/// the map specifies one of the fields within, or it gets added at runtime.
#[derive(Default, Debug)]
pub struct SpecialVars {
	pub tid: i64,
	pub special_i: [i64; 3],
	pub special_f: [f64; 2],
	pub args: [i64; 5],
}
