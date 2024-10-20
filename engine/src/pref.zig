//! Data structures for storing user preferences. 

pub const Pref = union(enum) {
	pub const Bool = struct {
		val: bool,
		default: bool,
	};

	pub const Float = struct {
		val: f64,
		default: f64,
	};

	pub const Int = struct {
		val: i64,
		default: i64,
	};

	pub const String = struct {
		val: []const u8,
		default: []const u8,
	};

	pub const Data = union(enum) {
		bool: Bool,
		int: Int,
		float: Float,
		string: String,
	};

	name: []const u8,
	data: Data,
};
