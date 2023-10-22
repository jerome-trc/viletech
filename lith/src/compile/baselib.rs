pub(crate) const BUILTINS: &str = include_str!(concat!(
	env!("CARGO_WORKSPACE_DIR"),
	"lith/baselib/builtins.lith"
));

pub(crate) const PRIMITIVE: &str = include_str!(concat!(
	env!("CARGO_WORKSPACE_DIR"),
	"lith/baselib/primitive.lith"
));
