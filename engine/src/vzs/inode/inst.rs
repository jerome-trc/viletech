//! Concrete implementations of individual instructions and the symbols involved.

use crate::vzs::abi::QWord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::vzs) enum BinOp {
	AddI8,
	SubI8,
	MulI8,
	DivI8,
	RemI8,

	AddI16,
	SubI16,
	MulI16,
	DivI16,
	RemI16,

	AddI32,
	SubI32,
	MulI32,
	DivI32,
	RemI32,

	AddI64,
	SubI64,
	MulI64,
	DivI64,
	RemI64,

	AddF,
	SubF,
	MulF,
	DivF,
	RemF,
}

impl BinOp {
	#[must_use]
	pub(super) unsafe fn eval(&self, l: &QWord, r: &QWord) -> QWord {
		// TODO: Divide-by-zero exceptions.
		match self {
			BinOp::AddI32 => (l.i_32.wrapping_add(r.i_32)).into(),
			BinOp::SubI32 => (l.i_32.wrapping_sub(r.i_32)).into(),
			BinOp::MulI32 => (l.i_32.wrapping_mul(r.i_32)).into(),
			BinOp::DivI32 => {
				if r.i_32 == 0 {
					panic!("Division by zero.");
				}

				(l.i_32 / r.i_32).into()
			}
			BinOp::RemI32 => (l.i_32 % r.i_32).into(),

			BinOp::AddI8 => (l.i_8.wrapping_add(r.i_8)).into(),
			BinOp::SubI8 => (l.i_8.wrapping_sub(r.i_8)).into(),
			BinOp::MulI8 => (l.i_8.wrapping_mul(r.i_8)).into(),
			BinOp::DivI8 => {
				if r.i_8 == 0 {
					panic!("Division by zero.");
				}

				(l.i_8 / r.i_8).into()
			}
			BinOp::RemI8 => (l.i_8 % r.i_8).into(),

			BinOp::AddI64 => (l.i_64.wrapping_add(r.i_64)).into(),
			BinOp::SubI64 => (l.i_64.wrapping_sub(r.i_64)).into(),
			BinOp::MulI64 => (l.i_64.wrapping_mul(r.i_64)).into(),
			BinOp::DivI64 => {
				if r.i_64 == 0 {
					panic!("Division by zero.");
				}

				(l.i_64 / r.i_64).into()
			}
			BinOp::RemI64 => (l.i_64 % r.i_64).into(),

			BinOp::AddI16 => (l.i_16.wrapping_add(r.i_16)).into(),
			BinOp::SubI16 => (l.i_16.wrapping_sub(r.i_16)).into(),
			BinOp::MulI16 => (l.i_16.wrapping_mul(r.i_16)).into(),
			BinOp::DivI16 => {
				if r.i_16 == 0 {
					panic!("Division by zero.");
				}

				(l.i_16 / r.i_16).into()
			}
			BinOp::RemI16 => (l.i_16 % r.i_16).into(),

			BinOp::AddF => (l.f_64 + r.f_64).into(),
			BinOp::SubF => (l.f_64 - r.f_64).into(),
			BinOp::MulF => (l.f_64 * r.f_64).into(),
			BinOp::DivF => (l.f_64 / r.f_64).into(),
			BinOp::RemF => (l.f_64 % r.f_64).into(),
		}
	}
}
