use cranelift::{
	codegen::{data_value::DataValue, ir::LibCall},
	prelude::{types, AbiParam, FloatCC, IntCC, TrapCode, Type},
};
use cranelift_interpreter::{
	interpreter::LibCallValues,
	value::{DataValueExt, ValueConversionKind, ValueResult},
};
use smallvec::smallvec;

use crate::ValVec;

use super::SimdVec;

pub(super) fn handle_libcall(
	libcall: LibCall,
	values: LibCallValues,
) -> Result<LibCallValues, TrapCode> {
	const TOINT_32: f32 = 1.0 / f32::EPSILON;
	const TOINT_64: f64 = 1.0 / f64::EPSILON;

	match libcall {
		LibCall::CeilF32 => {
			let DataValue::F32(f) = values.last().unwrap() else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F32(f.as_f32().ceil().into())])
		}
		LibCall::CeilF64 => {
			let DataValue::F64(f) = values.last().unwrap() else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F64(f.as_f64().ceil().into())])
		}
		LibCall::FloorF32 => {
			let DataValue::F32(f) = values.last().unwrap() else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F32(f.as_f32().floor().into())])
		}
		LibCall::FloorF64 => {
			let DataValue::F64(f) = values.last().unwrap() else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F64(f.as_f64().floor().into())])
		}
		LibCall::TruncF32 => {
			let DataValue::F32(f) = values.last().unwrap() else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F32(f.as_f32().trunc().into())])
		}
		LibCall::TruncF64 => {
			let DataValue::F64(f) = values.last().unwrap() else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F64(f.as_f64().trunc().into())])
		}
		LibCall::NearestF32 => {
			let DataValue::F32(f) = values.last().unwrap() else {
				unreachable!()
			};

			let x = f.as_f32();

			// Rust doesn't have a nearest function; there's nearbyint, but it's not
			// stabilized, so do it manually.
			// Nearest is either ceil or floor depending on which is nearest or even.
			// This approach exploited round half to even default mode.
			let i = x.to_bits();
			let e = i >> 23 & 0xff;

			let ret = if e >= 0x7f_u32 + 23 {
				// Check for NaNs.
				if e == 0xff {
					// Read the 23-bits significand.
					if i & 0x7fffff != 0 {
						// Ensure it's arithmetic by setting the significand's most
						// significant bit to 1; it also works for canonical NaNs.
						f32::from_bits(i | (1 << 22))
					} else {
						x
					}
				} else {
					x
				}
			} else {
				(x.abs() + TOINT_32 - TOINT_32).copysign(x)
			};

			Ok(smallvec![DataValue::F32(ret.into())])
		}
		LibCall::NearestF64 => {
			let DataValue::F64(f) = values.last().unwrap() else {
				unreachable!()
			};

			let x = f.as_f64();

			let i = x.to_bits();
			let e = i >> 52 & 0x7ff;

			let ret = if e >= 0x3ff_u64 + 52 {
				// Check for NaNs.
				if e == 0x7ff {
					// Read the 52-bits significand.
					if i & 0xfffffffffffff != 0 {
						// Ensure it's arithmetic by setting the significand's most
						// significant bit to 1; it also works for canonical NaNs.
						f64::from_bits(i | (1 << 51))
					} else {
						x
					}
				} else {
					x
				}
			} else {
				(x.abs() + TOINT_64 - TOINT_64).copysign(x)
			};

			Ok(smallvec![DataValue::F64(ret.into())])
		}
		LibCall::FmaF32 => {
			let &[DataValue::F32(a), DataValue::F32(b), DataValue::F32(c)] = &values[0..3] else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F32(
				a.as_f32().mul_add(b.as_f32(), c.as_f32()).into()
			)])
		}
		LibCall::FmaF64 => {
			let &[DataValue::F64(a), DataValue::F64(b), DataValue::F64(c)] = &values[0..3] else {
				unreachable!()
			};

			Ok(smallvec![DataValue::F64(
				a.as_f64().mul_add(b.as_f64(), c.as_f64()).into()
			)])
		}
		LibCall::Memcpy | LibCall::Memset | LibCall::Memmove | LibCall::Memcmp => todo!(),
		LibCall::ElfTlsGetAddr
		| LibCall::ElfTlsGetOffset
		| LibCall::X86Pshufb
		| LibCall::Probestack => unimplemented!(),
	}
}

/// Compare two values using the given integer condition `code`.
pub(super) fn icmp(
	ctrl_ty: types::Type,
	code: IntCC,
	left: &DataValue,
	right: &DataValue,
) -> ValueResult<DataValue> {
	let cmp = |bool_ty: types::Type,
	           code: IntCC,
	           left: &DataValue,
	           right: &DataValue|
	 -> ValueResult<DataValue> {
		DataValueExt::bool(
			match code {
				IntCC::Equal => left == right,
				IntCC::NotEqual => left != right,
				IntCC::SignedGreaterThan => left > right,
				IntCC::SignedGreaterThanOrEqual => left >= right,
				IntCC::SignedLessThan => left < right,
				IntCC::SignedLessThanOrEqual => left <= right,
				IntCC::UnsignedGreaterThan => {
					left.clone().into_int_unsigned()? > right.clone().into_int_unsigned()?
				}
				IntCC::UnsignedGreaterThanOrEqual => {
					left.clone().into_int_unsigned()? >= right.clone().into_int_unsigned()?
				}
				IntCC::UnsignedLessThan => {
					left.clone().into_int_unsigned()? < right.clone().into_int_unsigned()?
				}
				IntCC::UnsignedLessThanOrEqual => {
					left.clone().into_int_unsigned()? <= right.clone().into_int_unsigned()?
				}
			},
			ctrl_ty.is_vector(),
			bool_ty,
		)
	};

	let dst_ty = ctrl_ty.as_truthy();
	let left = extractlanes(left, ctrl_ty)?;
	let right = extractlanes(right, ctrl_ty)?;

	let res = left
		.into_iter()
		.zip(right.into_iter())
		.map(|(l, r)| cmp(dst_ty.lane_type(), code, &l, &r))
		.collect::<ValueResult<SimdVec<DataValue>>>()?;

	vectorizelanes(&res, dst_ty)
}

/// Compare two values using the given floating point condition `code`.
pub(super) fn fcmp(code: FloatCC, left: &DataValue, right: &DataValue) -> ValueResult<bool> {
	Ok(match code {
		FloatCC::Ordered => left <= right || left > right,
		FloatCC::Unordered => DataValueExt::uno(left, right)?,
		FloatCC::Equal => left == right,
		FloatCC::NotEqual => left != right || DataValueExt::uno(left, right)?,
		FloatCC::OrderedNotEqual => left != right,
		FloatCC::UnorderedOrEqual => left == right || DataValueExt::uno(left, right)?,
		FloatCC::LessThan => left < right,
		FloatCC::LessThanOrEqual => left <= right,
		FloatCC::GreaterThan => left > right,
		FloatCC::GreaterThanOrEqual => left >= right,
		FloatCC::UnorderedOrLessThan => DataValueExt::uno(left, right)? || left < right,
		FloatCC::UnorderedOrLessThanOrEqual => DataValueExt::uno(left, right)? || left <= right,
		FloatCC::UnorderedOrGreaterThan => DataValueExt::uno(left, right)? || left > right,
		FloatCC::UnorderedOrGreaterThanOrEqual => DataValueExt::uno(left, right)? || left >= right,
	})
}

/// Converts a SIMD vector value into a Rust array of [Value] for processing.
/// If `x` is a scalar, it will be returned as a single-element array.
pub(super) fn extractlanes(
	x: &DataValue,
	vector_type: types::Type,
) -> ValueResult<SimdVec<DataValue>> {
	let lane_type = vector_type.lane_type();
	let mut lanes = SimdVec::new();
	// Wrap scalar values as a single-element vector and return.
	if !x.ty().is_vector() {
		lanes.push(x.clone());
		return Ok(lanes);
	}

	let iterations = match lane_type {
		types::I8 => 1,
		types::I16 => 2,
		types::I32 | types::F32 => 4,
		types::I64 | types::F64 => 8,
		_ => unimplemented!("vectors with lanes wider than 64-bits are currently unsupported."),
	};

	let x = x.into_array()?;

	for i in 0..vector_type.lane_count() {
		let mut lane: i128 = 0;

		for j in 0..iterations {
			lane += (x[((i * iterations) + j) as usize] as i128) << (8 * j);
		}

		let lane_val: DataValue = if lane_type.is_float() {
			DataValueExt::float(lane as u64, lane_type)?
		} else {
			DataValueExt::int(lane, lane_type)?
		};
		lanes.push(lane_val);
	}

	Ok(lanes)
}

/// Convert a Rust array of [Value] back into a `Value::vector`.
/// Supplying a single-element array will simply return its contained value.
pub(super) fn vectorizelanes(x: &[DataValue], vector_type: types::Type) -> ValueResult<DataValue> {
	// If the array is only one element, return it as a scalar.
	if x.len() == 1 {
		Ok(x[0].clone())
	} else {
		vectorizelanes_all(x, vector_type)
	}
}

/// Convert a Rust array of [Value] back into a `Value::vector`.
pub(super) fn vectorizelanes_all(
	x: &[DataValue],
	vector_type: types::Type,
) -> ValueResult<DataValue> {
	let lane_type = vector_type.lane_type();
	let iterations = match lane_type {
		types::I8 => 1,
		types::I16 => 2,
		types::I32 | types::F32 => 4,
		types::I64 | types::F64 => 8,
		_ => unimplemented!("vectors with lanes wider than 64-bits are currently unsupported."),
	};
	let mut result: [u8; 16] = [0; 16];
	for (i, val) in x.iter().enumerate() {
		let lane_val: i128 = val
			.clone()
			.convert(ValueConversionKind::Exact(lane_type.as_int()))?
			.into_int_unsigned()? as i128;

		for j in 0..iterations {
			result[(i * iterations) + j] = (lane_val >> (8 * j)) as u8;
		}
	}
	DataValueExt::vector(result, vector_type)
}

/// Performs a lanewise fold on a vector type
pub(super) fn fold_vector<F>(
	v: DataValue,
	ty: types::Type,
	init: DataValue,
	op: F,
) -> ValueResult<DataValue>
where
	F: FnMut(DataValue, DataValue) -> ValueResult<DataValue>,
{
	extractlanes(&v, ty)?.into_iter().try_fold(init, op)
}

/// Performs the supplied unary arithmetic `op` on a Value, either Vector or Scalar.
pub(super) fn unary_arith<F>(
	x: DataValue,
	vector_type: types::Type,
	op: F,
) -> ValueResult<DataValue>
where
	F: Fn(DataValue) -> ValueResult<DataValue>,
{
	let arg = extractlanes(&x, vector_type)?;

	let result = arg
		.into_iter()
		.map(op)
		.collect::<ValueResult<SimdVec<DataValue>>>()?;

	vectorizelanes(&result, vector_type)
}

/// Performs the supplied binary arithmetic `op` on two values, either vector or scalar.
pub(super) fn binary_arith<F>(
	x: DataValue,
	y: DataValue,
	vector_type: types::Type,
	op: F,
) -> ValueResult<DataValue>
where
	F: Fn(DataValue, DataValue) -> ValueResult<DataValue>,
{
	let arg0 = extractlanes(&x, vector_type)?;
	let arg1 = extractlanes(&y, vector_type)?;

	let result = arg0
		.into_iter()
		.zip(arg1)
		.map(|(lhs, rhs)| op(lhs, rhs))
		.collect::<ValueResult<SimdVec<DataValue>>>()?;

	vectorizelanes(&result, vector_type)
}

/// Performs the supplied pairwise arithmetic `op` on two SIMD vectors, where
/// pairs are formed from adjacent vector elements and the vectors are
/// concatenated at the end.
pub(super) fn binary_pairwise<F>(
	x: DataValue,
	y: DataValue,
	vector_type: types::Type,
	op: F,
) -> ValueResult<DataValue>
where
	F: Fn(DataValue, DataValue) -> ValueResult<DataValue>,
{
	let arg0 = extractlanes(&x, vector_type)?;
	let arg1 = extractlanes(&y, vector_type)?;

	let result = arg0
		.chunks(2)
		.chain(arg1.chunks(2))
		.map(|pair| op(pair[0].clone(), pair[1].clone()))
		.collect::<ValueResult<SimdVec<DataValue>>>()?;

	vectorizelanes(&result, vector_type)
}

pub(super) fn bitselect(c: DataValue, x: DataValue, y: DataValue) -> ValueResult<DataValue> {
	let mask_x = DataValueExt::and(c.clone(), x)?;
	let mask_y = DataValueExt::and(DataValueExt::not(c)?, y)?;
	DataValueExt::or(mask_x, mask_y)
}

pub(super) fn splat(ty: Type, val: DataValue) -> ValueResult<DataValue> {
	let mut new_vector = SimdVec::new();
	for _ in 0..ty.lane_count() {
		new_vector.push(val.clone());
	}
	vectorizelanes(&new_vector, ty)
}

// Prepares the shift amount for a shift/rotate operation.
// The shift amount must be the same type and have the same number of lanes as the vector.
pub(super) fn shift_amt(ty: Type, val: DataValue) -> ValueResult<DataValue> {
	splat(ty, val.convert(ValueConversionKind::Exact(ty.lane_type()))?)
}

/// Ensures that all types in args are the same as expected by the signature
pub(super) fn validate_signature_params(sig: &[AbiParam], args: &[DataValue]) -> bool {
	args.iter()
		.map(|r| r.ty())
		.zip(sig.iter().map(|r| r.value_type))
		.all(|(a, b)| match (a, b) {
			// For these two cases we don't have precise type information for `a`.
			// We don't distinguish between different bool types, or different vector types
			// The actual error is in `Value::ty` that returns default types for some values
			// but we don't have enough information there either.
			//
			// Ideally the user has run the verifier and caught this properly...
			(a, b) if a.is_vector() && b.is_vector() => true,
			(a, b) => a == b,
		})
}

// Helper for summing a sequence of values.
pub(super) fn sum_unsigned(head: DataValue, tail: ValVec) -> ValueResult<u128> {
	let mut acc = head;

	for t in tail {
		acc = DataValueExt::add(acc, t)?;
	}

	acc.into_int_unsigned()
}
