use crate::utils::lang::Interner;

use super::*;

#[test]
fn something() {
	const SOURCE: &str = include_str!("test.lith");
	let interner = Interner::new_arc();
	let _ = parse_module(SOURCE, None, &interner).expect("Module parse failed unexpectedly.");
}
