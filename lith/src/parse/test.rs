use doomfront::testing::{assert_no_errors, prettyprint_maybe};

use crate::ParseTree;

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn smoke_expr_bin_userop() {
	const SAMPLE: &str = "a @dot b";

	let ptree: ParseTree = doomfront::parse(
		SAMPLE,
		|p| {
			super::expr(p);
		},
		(),
	);

	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}
