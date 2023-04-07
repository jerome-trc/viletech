use super::{abi::QWord, *};

#[test]
fn smoke_interpreter() {
	let mut builder = Builder::default();
	let mut runtime = Runtime::default();

	let left = builder.pop(LineInfo::new(69, 69));
	let right = builder.imm(LineInfo::new(420, 420), QWord { i_32: 32 });
	let bin = builder.bin_op(LineInfo::new(2, 2), BinOp::AddI32, left, right);
	builder.push(LineInfo::new(0, 0), bin);
	builder.ret(LineInfo::new(u32::MAX, u32::MAX));

	let tree = builder.build();

	let ret: i32 = tree.eval(&mut runtime, 64_i32);
	assert_eq!(ret, (64 + 32));
}
