//! "Compile-time function execution".
//!
//! An interpreter for Cranelift's CLIF IR adapted from [`cranelift_interpreter`].
//!
//! For licensing information, see the repository's ATTRIB.md file.

mod help;
mod state;

use std::{
	convert::{TryFrom, TryInto},
	ops::RangeFrom,
};

use cranelift::codegen::{
	data_value::DataValue,
	ir::{
		condcodes::IntCC, types, AtomicRmwOp, BlockCall, Endianness, ExternalName, Function,
		InstructionData, LibCall, MemFlags, Opcode, TrapCode, Type,
	},
};
use cranelift_interpreter::{
	address::{Address, AddressSize},
	instruction::{DfgInstructionContext, InstructionContext},
	state::{InterpreterFunctionRef, MemoryError, State},
	step::{ControlFlow, CraneliftTrap, StepError},
	value::{DataValueExt, ValueConversionKind, ValueError, ValueResult},
};
use smallvec::{smallvec, SmallVec};

use crate::{builtins, ValVec};

pub(crate) use self::state::Interpreter;

/// Raised when the intepreter attempts to call a function which does not support
/// compile-time execution, such as allocation via the garbage collector.
pub(crate) const TRAP_UNSUPPORTED: TrapCode = TrapCode::User(0451);

/// Interpret a single Cranelift instruction. Note that program traps and interpreter errors are
/// distinct: a program trap results in `Ok(Flow::Trap(...))` whereas an interpretation error (e.g.
/// the types of two values are incompatible) results in `Err(...)`.
#[allow(unused_variables)]
pub(crate) fn step<'c>(
	state: &'c mut Interpreter,
	ictx: DfgInstructionContext<'c>,
) -> Result<ControlFlow<'c>, StepError> {
	let inst = ictx.data();
	let ctrl_ty = ictx.controlling_type().unwrap();

	// The following closures make the `step` implementation much easier to express.
	// Note that they frequently close over the `state` or `ictx` for brevity.

	// Retrieve the current value for an instruction argument.
	let arg = |index: usize| -> DataValue {
		let value_ref = ictx.args()[index];
		state.current_frame().get(value_ref).clone()
	};

	// Retrieve the current values for all of an instruction's arguments.
	let args = || -> ValVec { state.collect_values(ictx.args()) };

	// Retrieve the current values for a range of an instruction's arguments.
	let args_range = |indexes: RangeFrom<usize>| -> Result<ValVec, StepError> {
		Ok(SmallVec::<[DataValue; 1]>::from(&args()[indexes]))
	};

	// Retrieve the immediate value for an instruction, expecting it to exist.
	let imm = || -> DataValue {
		match inst {
			InstructionData::UnaryConst {
				constant_handle, ..
			} => {
				let buffer = state
					.get_current_function()
					.dfg
					.constants
					.get(constant_handle)
					.as_slice();
				match ctrl_ty.bytes() {
					16 => DataValue::V128(buffer.try_into().expect("a 16-byte data buffer")),
					8 => DataValue::V64(buffer.try_into().expect("an 8-byte data buffer")),
					length => panic!("unexpected UnaryConst buffer length {}", length),
				}
			}
			InstructionData::Shuffle { imm, .. } => {
				let mask = state
					.get_current_function()
					.dfg
					.immediates
					.get(imm)
					.unwrap()
					.as_slice();
				match mask.len() {
					16 => DataValue::V128(mask.try_into().expect("a 16-byte vector mask")),
					8 => DataValue::V64(mask.try_into().expect("an 8-byte vector mask")),
					length => panic!("unexpected Shuffle mask length {}", mask.len()),
				}
			}
			// 8-bit.
			InstructionData::BinaryImm8 { imm, .. } | InstructionData::TernaryImm8 { imm, .. } => {
				DataValue::from(imm as i8) // Note the switch from unsigned to signed.
			}
			// 32-bit
			InstructionData::UnaryIeee32 { imm, .. } => DataValue::from(imm),
			InstructionData::Load { offset, .. }
			| InstructionData::Store { offset, .. }
			| InstructionData::StackLoad { offset, .. }
			| InstructionData::StackStore { offset, .. }
			| InstructionData::TableAddr { offset, .. } => DataValue::from(offset),
			// 64-bit.
			InstructionData::UnaryImm { imm, .. }
			| InstructionData::BinaryImm64 { imm, .. }
			| InstructionData::IntCompareImm { imm, .. } => DataValue::from(imm.bits()),
			InstructionData::UnaryIeee64 { imm, .. } => DataValue::from(imm),
			_ => unreachable!(),
		}
	};

	// Retrieve the immediate value for an instruction and convert it to the controlling type of the
	// instruction. For example, since `InstructionData` stores all integer immediates in a 64-bit
	// size, this will attempt to convert `iconst.i8 ...` to an 8-bit size.
	let imm_as_ctrl_ty = || -> Result<DataValue, ValueError> {
		DataValue::convert(imm(), ValueConversionKind::Exact(ctrl_ty))
	};

	// Indicate that the result of a step is to assign a single value to an instruction's results.
	let assign = |value: DataValue| ControlFlow::Assign(smallvec![value]);

	// Indicate that the result of a step is to assign multiple values to an instruction's results.
	let assign_multiple = |values: &[DataValue]| ControlFlow::Assign(SmallVec::from(values));

	// Similar to `assign` but converts some errors into traps
	let assign_or_trap = |value: ValueResult<DataValue>| match value {
		Ok(v) => Ok(assign(v)),
		Err(ValueError::IntegerDivisionByZero) => Ok(ControlFlow::Trap(CraneliftTrap::User(
			TrapCode::IntegerDivisionByZero,
		))),
		Err(ValueError::IntegerOverflow) => Ok(ControlFlow::Trap(CraneliftTrap::User(
			TrapCode::IntegerOverflow,
		))),
		Err(e) => Err(e),
	};

	let memerror_to_trap = |e: MemoryError| match e {
		MemoryError::InvalidAddress(_) => TrapCode::HeapOutOfBounds,
		MemoryError::InvalidAddressType(_) => TrapCode::HeapOutOfBounds,
		MemoryError::InvalidOffset { .. } => TrapCode::HeapOutOfBounds,
		MemoryError::InvalidEntry { .. } => TrapCode::HeapOutOfBounds,
		MemoryError::OutOfBoundsStore { .. } => TrapCode::HeapOutOfBounds,
		MemoryError::OutOfBoundsLoad { .. } => TrapCode::HeapOutOfBounds,
		MemoryError::MisalignedLoad { .. } => TrapCode::HeapMisaligned,
		MemoryError::MisalignedStore { .. } => TrapCode::HeapMisaligned,
	};

	// Assigns or traps depending on the value of the result
	let assign_or_memtrap = |res| match res {
		Ok(v) => assign(v),
		Err(e) => ControlFlow::Trap(CraneliftTrap::User(memerror_to_trap(e))),
	};

	// Continues or traps depending on the value of the result
	let continue_or_memtrap = |res| match res {
		Ok(_) => ControlFlow::Continue,
		Err(e) => ControlFlow::Trap(CraneliftTrap::User(memerror_to_trap(e))),
	};

	let calculate_addr = |addr_ty: Type, imm: DataValue, args: ValVec| -> ValueResult<u64> {
		let imm = imm.convert(ValueConversionKind::ZeroExtend(addr_ty))?;
		let args = args
			.into_iter()
			.map(|v| v.convert(ValueConversionKind::ZeroExtend(addr_ty)))
			.collect::<ValueResult<ValVec>>()?;

		Ok(help::sum_unsigned(imm, args)? as u64)
	};

	// Interpret a unary instruction with the given `op`, assigning the resulting value to the
	// instruction's results.
	let unary =
		|op: fn(DataValue) -> ValueResult<DataValue>, arg: DataValue| -> ValueResult<ControlFlow> {
			let ctrl_ty = ictx.controlling_type().unwrap();
			let res = help::unary_arith(arg, ctrl_ty, op)?;
			Ok(assign(res))
		};

	// Interpret a binary instruction with the given `op`, assigning the resulting value to the
	// instruction's results.
	let binary = |op: fn(DataValue, DataValue) -> ValueResult<DataValue>,
	              left: DataValue,
	              right: DataValue|
	 -> ValueResult<ControlFlow> {
		let ctrl_ty = ictx.controlling_type().unwrap();
		let res = help::binary_arith(left, right, ctrl_ty, op)?;
		Ok(assign(res))
	};

	// Similar to `binary` but converts select `ValueError`'s into trap `ControlFlow`'s
	let binary_can_trap = |op: fn(DataValue, DataValue) -> ValueResult<DataValue>,
	                       left: DataValue,
	                       right: DataValue|
	 -> ValueResult<ControlFlow> {
		let ctrl_ty = ictx.controlling_type().unwrap();
		let res = help::binary_arith(left, right, ctrl_ty, op);
		assign_or_trap(res)
	};

	// Choose whether to assign `left` or `right` to the instruction's result based on a `condition`.
	let choose = |condition: bool, left: DataValue, right: DataValue| -> ControlFlow {
		assign(if condition { left } else { right })
	};

	// Retrieve an instruction's branch destination; expects the instruction to be a branch.

	let continue_at = |block: BlockCall| {
		let branch_args =
			state.collect_values(block.args_slice(&state.get_current_function().dfg.value_lists));
		Ok(ControlFlow::ContinueAt(
			block.block(&state.get_current_function().dfg.value_lists),
			branch_args,
		))
	};

	// Based on `condition`, indicate where to continue the control flow.
	let branch_when = |condition: bool, block| -> Result<ControlFlow, StepError> {
		if condition {
			continue_at(block)
		} else {
			Ok(ControlFlow::Continue)
		}
	};

	// Retrieve an instruction's trap code; expects the instruction to be a trap.
	let trap_code = || -> TrapCode { inst.trap_code().unwrap() };

	// Based on `condition`, either trap or not.
	let trap_when = |condition: bool, trap: CraneliftTrap| -> ControlFlow {
		if condition {
			ControlFlow::Trap(trap)
		} else {
			ControlFlow::Continue
		}
	};

	// Calls a function reference with the given arguments.
	let call_func = |func_ref: InterpreterFunctionRef<'c>,
	                 args: ValVec,
	                 make_ctrl_flow: fn(&'c Function, ValVec) -> ControlFlow|
	 -> Result<ControlFlow, StepError> {
		let signature = func_ref.signature();

		// Check the types of the arguments. This is usually done by the verifier,
		// but nothing guarantees that the user has ran that.
		//
		// TODO: (Lithica) run the verifier in sema. instead of the backend
		// so that this call can be elided.
		let args_match = help::validate_signature_params(&signature.params[..], &args[..]);

		if !args_match {
			return Ok(ControlFlow::Trap(CraneliftTrap::User(
				TrapCode::BadSignature,
			)));
		}

		Ok(match func_ref {
			InterpreterFunctionRef::Function(func) => make_ctrl_flow(func, args),
			InterpreterFunctionRef::LibCall(libcall) => {
				debug_assert!(
					!matches!(
						inst.opcode(),
						Opcode::ReturnCall | Opcode::ReturnCallIndirect,
					),
					"Cannot tail call to libcalls"
				);

				let libcall_handler = state.get_libcall_handler();

				// We don't transfer control to a libcall, we just execute it and return the results
				let res = libcall_handler(libcall, args);

				let res = match res {
					Err(trap) => return Ok(ControlFlow::Trap(CraneliftTrap::User(trap))),
					Ok(rets) => rets,
				};

				// Check that what the handler returned is what we expect.
				if help::validate_signature_params(&signature.returns[..], &res[..]) {
					ControlFlow::Assign(res)
				} else {
					ControlFlow::Trap(CraneliftTrap::User(TrapCode::BadSignature))
				}
			}
		})
	};

	// Interpret a Cranelift instruction.
	Ok(match inst.opcode() {
		Opcode::Jump => {
			if let InstructionData::Jump { destination, .. } = inst {
				continue_at(destination)?
			} else {
				unreachable!()
			}
		}
		Opcode::Brif => {
			if let InstructionData::Brif {
				arg,
				blocks: [block_then, block_else],
				..
			} = inst
			{
				let arg = state.current_frame().get(arg).clone();

				let condition = arg.convert(ValueConversionKind::ToBoolean)?.into_bool()?;

				if condition {
					continue_at(block_then)?
				} else {
					continue_at(block_else)?
				}
			} else {
				unreachable!()
			}
		}
		Opcode::BrTable => {
			let InstructionData::BranchTable { table, .. } = inst else {
				unreachable!()
			};

			let jt_data = &state.get_current_function().stencil.dfg.jump_tables[table];

			// Convert to usize to remove negative indexes from the following operations
			let jump_target = usize::try_from(arg(0).into_int_unsigned()?)
				.ok()
				.and_then(|i| jt_data.as_slice().get(i))
				.copied()
				.unwrap_or(jt_data.default_block());

			continue_at(jump_target)?
		}
		Opcode::Trap => ControlFlow::Trap(CraneliftTrap::User(trap_code())),
		Opcode::Debugtrap => ControlFlow::Trap(CraneliftTrap::Debug),
		Opcode::ResumableTrap => ControlFlow::Trap(CraneliftTrap::Resumable),
		Opcode::Trapz => trap_when(!arg(0).into_bool()?, CraneliftTrap::User(trap_code())),
		Opcode::Trapnz => trap_when(arg(0).into_bool()?, CraneliftTrap::User(trap_code())),
		Opcode::ResumableTrapnz => trap_when(arg(0).into_bool()?, CraneliftTrap::Resumable),
		Opcode::Return => ControlFlow::Return(args()),
		Opcode::Call | Opcode::ReturnCall => {
			let InstructionData::Call { func_ref, .. } = inst else {
				unreachable!()
			};

			let curr_func = state.get_current_function();

			let ext_data = curr_func
				.dfg
				.ext_funcs
				.get(func_ref)
				.ok_or(StepError::UnknownFunction(func_ref))?;

			let args = args();

			match ext_data.name {
				ExternalName::User(uenr) => {
					let u_map = curr_func.params.user_named_funcs().get(uenr).unwrap();

					match u_map.namespace {
						crate::CLNS_BUILTIN => match builtins::Index::from(u_map.index) {
							builtins::Index::__Last => unreachable!(),
							_ => unimplemented!(),
						},
						crate::CLNS_NATIVE => {
							let func = state
								.compiler
								.native
								.functions
								.get_index(u_map.index as usize)
								.unwrap();

							unimplemented!()
						}
						_ => {
							let curr_ir = state.get_current_function();
							let ext_data = curr_ir.stencil.dfg.ext_funcs.get(func_ref).unwrap();

							let ExternalName::User(uenr) = ext_data.name else {
								unimplemented!()
							};

							let uen = curr_ir.params.user_named_funcs().get(uenr).unwrap();

							let ir_ref = state.compiler.ir.get(uen).unwrap();

							let make_control_flow = match inst.opcode() {
								Opcode::Call => ControlFlow::Call,
								Opcode::ReturnCall => ControlFlow::ReturnCall,
								_ => unreachable!(),
							};

							state.ir_ptr.store((&ir_ref.ptr).into());
							let ir = state.ir_ptr.as_ref();
							let ifnref = InterpreterFunctionRef::Function(ir);
							call_func(ifnref, args, make_control_flow)?
						}
					}
				}
				ExternalName::LibCall(libcall) => {
					let make_control_flow = match inst.opcode() {
						Opcode::Call => ControlFlow::Call,
						Opcode::ReturnCall => ControlFlow::ReturnCall,
						_ => unreachable!(),
					};

					call_func(
						InterpreterFunctionRef::LibCall(libcall),
						args,
						make_control_flow,
					)?
				}
				ExternalName::TestCase(_) | ExternalName::KnownSymbol(_) => unimplemented!(),
			}
		}
		Opcode::CallIndirect | Opcode::ReturnCallIndirect => {
			let args = args();
			let addr_dv = DataValue::I64(arg(0).into_int_unsigned()? as i64);
			let addr = Address::try_from(addr_dv.clone()).map_err(StepError::MemoryError)?;

			match addr.entry as u32 {
				crate::CLNS_LIBCALL => {
					let libcall = LibCall::all_libcalls()
						.get(addr.offset as usize)
						.copied()
						.map(InterpreterFunctionRef::from);
				}
				crate::CLNS_BUILTIN => {}
				crate::CLNS_NATIVE => {}
				user => {}
			}

			let func = state
				.get_function_from_address(addr)
				.ok_or(StepError::MemoryError(MemoryError::InvalidAddress(addr_dv)))?;

			let call_args: ValVec = SmallVec::from(&args[1..]);

			let make_control_flow = match inst.opcode() {
				Opcode::CallIndirect => ControlFlow::Call,
				Opcode::ReturnCallIndirect => ControlFlow::ReturnCall,
				_ => unreachable!(),
			};

			call_func(func, call_args, make_control_flow)?
		}
		Opcode::FuncAddr => {
			let InstructionData::FuncAddr { func_ref, .. } = inst else {
				unreachable!()
			};

			let ext_data = state
				.get_current_function()
				.dfg
				.ext_funcs
				.get(func_ref)
				.ok_or(StepError::UnknownFunction(func_ref))?;

			let addr_ty = ictx.controlling_type().unwrap();

			assign_or_memtrap({
				AddressSize::try_from(addr_ty).and_then(|addr_size| {
					let addr = state.function_address(addr_size, &ext_data.name)?;
					let dv = DataValue::try_from(addr)?;
					Ok(dv)
				})
			})
		}
		Opcode::Load
		| Opcode::Uload8
		| Opcode::Sload8
		| Opcode::Uload16
		| Opcode::Sload16
		| Opcode::Uload32
		| Opcode::Sload32
		| Opcode::Uload8x8
		| Opcode::Sload8x8
		| Opcode::Uload16x4
		| Opcode::Sload16x4
		| Opcode::Uload32x2
		| Opcode::Sload32x2 => {
			let ctrl_ty = ictx.controlling_type().unwrap();
			let (load_ty, kind) = match inst.opcode() {
				Opcode::Load => (ctrl_ty, None),
				Opcode::Uload8 => (types::I8, Some(ValueConversionKind::ZeroExtend(ctrl_ty))),
				Opcode::Sload8 => (types::I8, Some(ValueConversionKind::SignExtend(ctrl_ty))),
				Opcode::Uload16 => (types::I16, Some(ValueConversionKind::ZeroExtend(ctrl_ty))),
				Opcode::Sload16 => (types::I16, Some(ValueConversionKind::SignExtend(ctrl_ty))),
				Opcode::Uload32 => (types::I32, Some(ValueConversionKind::ZeroExtend(ctrl_ty))),
				Opcode::Sload32 => (types::I32, Some(ValueConversionKind::SignExtend(ctrl_ty))),
				Opcode::Uload8x8
				| Opcode::Sload8x8
				| Opcode::Uload16x4
				| Opcode::Sload16x4
				| Opcode::Uload32x2
				| Opcode::Sload32x2 => unimplemented!(),
				_ => unreachable!(),
			};

			let addr_value = calculate_addr(types::I64, imm(), args())?;
			let mem_flags = inst.memflags().expect("instruction to have memory flags");
			let loaded = assign_or_memtrap(
				Address::try_from(addr_value)
					.and_then(|addr| state.checked_load(addr, load_ty, mem_flags)),
			);

			match (loaded, kind) {
				(ControlFlow::Assign(ret), Some(c)) => ControlFlow::Assign(
					ret.into_iter()
						.map(|loaded| loaded.convert(c.clone()))
						.collect::<ValueResult<ValVec>>()?,
				),
				(cf, _) => cf,
			}
		}
		Opcode::Store | Opcode::Istore8 | Opcode::Istore16 | Opcode::Istore32 => {
			let kind = match inst.opcode() {
				Opcode::Store => None,
				Opcode::Istore8 => Some(ValueConversionKind::Truncate(types::I8)),
				Opcode::Istore16 => Some(ValueConversionKind::Truncate(types::I16)),
				Opcode::Istore32 => Some(ValueConversionKind::Truncate(types::I32)),
				_ => unreachable!(),
			};

			let addr_value = calculate_addr(types::I64, imm(), args_range(1..)?)?;
			let mem_flags = inst.memflags().expect("instruction to have memory flags");
			let reduced = if let Some(c) = kind {
				arg(0).convert(c)?
			} else {
				arg(0)
			};
			continue_or_memtrap(
				Address::try_from(addr_value)
					.and_then(|addr| state.checked_store(addr, reduced, mem_flags)),
			)
		}
		Opcode::StackLoad => {
			let load_ty = ictx.controlling_type().unwrap();
			let slot = inst.stack_slot().unwrap();
			let offset = help::sum_unsigned(imm(), args())? as u64;
			let mem_flags = MemFlags::new();
			assign_or_memtrap({
				state
					.stack_address(AddressSize::_64, slot, offset)
					.and_then(|addr| state.checked_load(addr, load_ty, mem_flags))
			})
		}
		Opcode::StackStore => {
			let arg = arg(0);
			let slot = inst.stack_slot().unwrap();
			let offset = help::sum_unsigned(imm(), args_range(1..)?)? as u64;
			let mem_flags = MemFlags::new();
			continue_or_memtrap({
				state
					.stack_address(AddressSize::_64, slot, offset)
					.and_then(|addr| state.checked_store(addr, arg, mem_flags))
			})
		}
		Opcode::StackAddr => {
			let load_ty = ictx.controlling_type().unwrap();
			let slot = inst.stack_slot().unwrap();
			let offset = help::sum_unsigned(imm(), args())? as u64;
			assign_or_memtrap({
				AddressSize::try_from(load_ty).and_then(|addr_size| {
					let addr = state.stack_address(addr_size, slot, offset)?;
					let dv = DataValue::try_from(addr)?;
					Ok(dv)
				})
			})
		}
		Opcode::GlobalValue | Opcode::SymbolValue | Opcode::TlsValue => {
			if let InstructionData::UnaryGlobalValue { global_value, .. } = inst {
				assign_or_memtrap(state.resolve_global_value(global_value))
			} else {
				unreachable!()
			}
		}
		Opcode::GetPinnedReg => assign(state.get_pinned_reg()),
		Opcode::SetPinnedReg => {
			let arg0 = arg(0);
			state.set_pinned_reg(arg0);
			ControlFlow::Continue
		}
		Opcode::TableAddr => {
			if let InstructionData::TableAddr { table, offset, .. } = inst {
				let table = &state.get_current_function().tables[table];
				let base = state.resolve_global_value(table.base_gv)?;
				let bound = state.resolve_global_value(table.bound_gv)?;
				let index_ty = table.index_type;
				let element_size = DataValue::int(u64::from(table.element_size) as i128, index_ty)?;
				let inst_offset = DataValue::int(i32::from(offset) as i128, index_ty)?;

				let byte_offset = arg(0).mul(element_size.clone())?.add(inst_offset)?;
				let bound_bytes = bound.mul(element_size)?;
				if byte_offset > bound_bytes {
					return Ok(ControlFlow::Trap(CraneliftTrap::User(
						TrapCode::HeapOutOfBounds,
					)));
				}

				assign(base.add(byte_offset)?)
			} else {
				unreachable!()
			}
		}
		Opcode::Iconst => assign(DataValueExt::int(imm().into_int_signed()?, ctrl_ty)?),
		Opcode::F32const => assign(imm()),
		Opcode::F64const => assign(imm()),
		Opcode::Vconst => assign(imm()),
		Opcode::Nop => ControlFlow::Continue,
		Opcode::Select | Opcode::SelectSpectreGuard => choose(arg(0).into_bool()?, arg(1), arg(2)),
		Opcode::Bitselect => assign(help::bitselect(arg(0), arg(1), arg(2))?),
		Opcode::Icmp => assign(help::icmp(
			ctrl_ty,
			inst.cond_code().unwrap(),
			&arg(0),
			&arg(1),
		)?),
		Opcode::IcmpImm => assign(help::icmp(
			ctrl_ty,
			inst.cond_code().unwrap(),
			&arg(0),
			&imm_as_ctrl_ty()?,
		)?),
		Opcode::Smin => {
			if ctrl_ty.is_vector() {
				let icmp = help::icmp(ctrl_ty, IntCC::SignedGreaterThan, &arg(1), &arg(0))?;
				assign(help::bitselect(icmp, arg(0), arg(1))?)
			} else {
				assign(arg(0).smin(arg(1))?)
			}
		}
		Opcode::Umin => {
			if ctrl_ty.is_vector() {
				let icmp = help::icmp(ctrl_ty, IntCC::UnsignedGreaterThan, &arg(1), &arg(0))?;
				assign(help::bitselect(icmp, arg(0), arg(1))?)
			} else {
				assign(arg(0).umin(arg(1))?)
			}
		}
		Opcode::Smax => {
			if ctrl_ty.is_vector() {
				let icmp = help::icmp(ctrl_ty, IntCC::SignedGreaterThan, &arg(0), &arg(1))?;
				assign(help::bitselect(icmp, arg(0), arg(1))?)
			} else {
				assign(arg(0).smax(arg(1))?)
			}
		}
		Opcode::Umax => {
			if ctrl_ty.is_vector() {
				let icmp = help::icmp(ctrl_ty, IntCC::UnsignedGreaterThan, &arg(0), &arg(1))?;
				assign(help::bitselect(icmp, arg(0), arg(1))?)
			} else {
				assign(arg(0).umax(arg(1))?)
			}
		}
		Opcode::AvgRound => {
			let sum = DataValueExt::add(arg(0), arg(1))?;
			let one = DataValueExt::int(1, arg(0).ty())?;
			let inc = DataValueExt::add(sum, one)?;
			let two = DataValueExt::int(2, arg(0).ty())?;
			binary(DataValueExt::udiv, inc, two)?
		}
		Opcode::Iadd => binary(DataValueExt::add, arg(0), arg(1))?,
		Opcode::UaddSat => assign(help::binary_arith(
			arg(0),
			arg(1),
			ctrl_ty,
			DataValueExt::uadd_sat,
		)?),
		Opcode::SaddSat => assign(help::binary_arith(
			arg(0),
			arg(1),
			ctrl_ty,
			DataValueExt::sadd_sat,
		)?),
		Opcode::Isub => binary(DataValueExt::sub, arg(0), arg(1))?,
		Opcode::UsubSat => assign(help::binary_arith(
			arg(0),
			arg(1),
			ctrl_ty,
			DataValueExt::usub_sat,
		)?),
		Opcode::SsubSat => assign(help::binary_arith(
			arg(0),
			arg(1),
			ctrl_ty,
			DataValueExt::ssub_sat,
		)?),
		Opcode::Ineg => binary(DataValueExt::sub, DataValueExt::int(0, ctrl_ty)?, arg(0))?,
		Opcode::Iabs => {
			let (min_val, _) = ctrl_ty.lane_type().bounds(true);
			let min_val: DataValue = DataValueExt::int(min_val as i128, ctrl_ty.lane_type())?;
			let arg0 = help::extractlanes(&arg(0), ctrl_ty)?;
			let new_vec = arg0
				.into_iter()
				.map(|lane| {
					if lane == min_val {
						Ok(min_val.clone())
					} else {
						DataValueExt::int(lane.into_int_signed()?.abs(), ctrl_ty.lane_type())
					}
				})
				.collect::<ValueResult<SimdVec<DataValue>>>()?;
			assign(help::vectorizelanes(&new_vec, ctrl_ty)?)
		}
		Opcode::Imul => binary(DataValueExt::mul, arg(0), arg(1))?,
		Opcode::Umulhi | Opcode::Smulhi => {
			let double_length = match ctrl_ty.lane_bits() {
				8 => types::I16,
				16 => types::I32,
				32 => types::I64,
				64 => types::I128,
				_ => unimplemented!("Unsupported integer length {}", ctrl_ty.bits()),
			};
			let conv_type = if inst.opcode() == Opcode::Umulhi {
				ValueConversionKind::ZeroExtend(double_length)
			} else {
				ValueConversionKind::SignExtend(double_length)
			};
			let arg0 = help::extractlanes(&arg(0), ctrl_ty)?;
			let arg1 = help::extractlanes(&arg(1), ctrl_ty)?;

			let res = arg0
				.into_iter()
				.zip(arg1)
				.map(|(x, y)| {
					let x = x.convert(conv_type.clone())?;
					let y = y.convert(conv_type.clone())?;

					DataValueExt::mul(x, y)?
						.convert(ValueConversionKind::ExtractUpper(ctrl_ty.lane_type()))
				})
				.collect::<ValueResult<SimdVec<DataValue>>>()?;

			assign(help::vectorizelanes(&res, ctrl_ty)?)
		}
		Opcode::Udiv => binary_can_trap(DataValueExt::udiv, arg(0), arg(1))?,
		Opcode::Sdiv => binary_can_trap(DataValueExt::sdiv, arg(0), arg(1))?,
		Opcode::Urem => binary_can_trap(DataValueExt::urem, arg(0), arg(1))?,
		Opcode::Srem => binary_can_trap(DataValueExt::srem, arg(0), arg(1))?,
		Opcode::IaddImm => binary(DataValueExt::add, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::ImulImm => binary(DataValueExt::mul, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::UdivImm => binary_can_trap(DataValueExt::udiv, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::SdivImm => binary_can_trap(DataValueExt::sdiv, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::UremImm => binary_can_trap(DataValueExt::urem, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::SremImm => binary_can_trap(DataValueExt::srem, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::IrsubImm => binary(DataValueExt::sub, imm_as_ctrl_ty()?, arg(0))?,
		Opcode::UaddOverflow => {
			let (sum, carry) = arg(0).uadd_overflow(arg(1))?;
			assign_multiple(&[sum, DataValueExt::bool(carry, false, types::I8)?])
		}
		Opcode::SaddOverflow => {
			let (sum, carry) = arg(0).sadd_overflow(arg(1))?;
			assign_multiple(&[sum, DataValueExt::bool(carry, false, types::I8)?])
		}
		Opcode::UsubOverflow => {
			let (sum, carry) = arg(0).usub_overflow(arg(1))?;
			assign_multiple(&[sum, DataValueExt::bool(carry, false, types::I8)?])
		}
		Opcode::SsubOverflow => {
			let (sum, carry) = arg(0).ssub_overflow(arg(1))?;
			assign_multiple(&[sum, DataValueExt::bool(carry, false, types::I8)?])
		}
		Opcode::UmulOverflow => {
			let (sum, carry) = arg(0).umul_overflow(arg(1))?;
			assign_multiple(&[sum, DataValueExt::bool(carry, false, types::I8)?])
		}
		Opcode::SmulOverflow => {
			let (sum, carry) = arg(0).smul_overflow(arg(1))?;
			assign_multiple(&[sum, DataValueExt::bool(carry, false, types::I8)?])
		}
		Opcode::IaddCin => choose(
			DataValueExt::into_bool(arg(2))?,
			DataValueExt::add(
				DataValueExt::add(arg(0), arg(1))?,
				DataValueExt::int(1, ctrl_ty)?,
			)?,
			DataValueExt::add(arg(0), arg(1))?,
		),
		Opcode::IaddCarry => {
			let mut sum = DataValueExt::add(arg(0), arg(1))?;
			let mut carry = arg(0).sadd_checked(arg(1))?.is_none();

			if DataValueExt::into_bool(arg(2))? {
				carry |= sum
					.clone()
					.sadd_checked(DataValueExt::int(1, ctrl_ty)?)?
					.is_none();
				sum = DataValueExt::add(sum, DataValueExt::int(1, ctrl_ty)?)?;
			}

			assign_multiple(&[sum, DataValueExt::bool(carry, false, types::I8)?])
		}
		Opcode::UaddOverflowTrap => {
			let sum = DataValueExt::add(arg(0), arg(1))?;
			let carry = sum < arg(0) && sum < arg(1);
			if carry {
				ControlFlow::Trap(CraneliftTrap::User(trap_code()))
			} else {
				assign(sum)
			}
		}
		Opcode::IsubBin => choose(
			DataValueExt::into_bool(arg(2))?,
			DataValueExt::sub(
				arg(0),
				DataValueExt::add(arg(1), DataValueExt::int(1, ctrl_ty)?)?,
			)?,
			DataValueExt::sub(arg(0), arg(1))?,
		),
		Opcode::IsubBorrow => {
			let rhs = if DataValueExt::into_bool(arg(2))? {
				DataValueExt::add(arg(1), DataValueExt::int(1, ctrl_ty)?)?
			} else {
				arg(1)
			};
			let borrow = arg(0) < rhs;
			let sum = DataValueExt::sub(arg(0), rhs)?;
			assign_multiple(&[sum, DataValueExt::bool(borrow, false, types::I8)?])
		}
		Opcode::Band => binary(DataValueExt::and, arg(0), arg(1))?,
		Opcode::Bor => binary(DataValueExt::or, arg(0), arg(1))?,
		Opcode::Bxor => binary(DataValueExt::xor, arg(0), arg(1))?,
		Opcode::Bnot => unary(DataValueExt::not, arg(0))?,
		Opcode::BandNot => binary(DataValueExt::and, arg(0), DataValueExt::not(arg(1))?)?,
		Opcode::BorNot => binary(DataValueExt::or, arg(0), DataValueExt::not(arg(1))?)?,
		Opcode::BxorNot => binary(DataValueExt::xor, arg(0), DataValueExt::not(arg(1))?)?,
		Opcode::BandImm => binary(DataValueExt::and, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::BorImm => binary(DataValueExt::or, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::BxorImm => binary(DataValueExt::xor, arg(0), imm_as_ctrl_ty()?)?,
		Opcode::Rotl => binary(
			DataValueExt::rotl,
			arg(0),
			help::shift_amt(ctrl_ty, arg(1))?,
		)?,
		Opcode::Rotr => binary(
			DataValueExt::rotr,
			arg(0),
			help::shift_amt(ctrl_ty, arg(1))?,
		)?,
		Opcode::RotlImm => binary(DataValueExt::rotl, arg(0), help::shift_amt(ctrl_ty, imm())?)?,
		Opcode::RotrImm => binary(DataValueExt::rotr, arg(0), help::shift_amt(ctrl_ty, imm())?)?,
		Opcode::Ishl => binary(DataValueExt::shl, arg(0), help::shift_amt(ctrl_ty, arg(1))?)?,
		Opcode::Ushr => binary(
			DataValueExt::ushr,
			arg(0),
			help::shift_amt(ctrl_ty, arg(1))?,
		)?,
		Opcode::Sshr => binary(
			DataValueExt::sshr,
			arg(0),
			help::shift_amt(ctrl_ty, arg(1))?,
		)?,
		Opcode::IshlImm => binary(DataValueExt::shl, arg(0), help::shift_amt(ctrl_ty, imm())?)?,
		Opcode::UshrImm => binary(DataValueExt::ushr, arg(0), help::shift_amt(ctrl_ty, imm())?)?,
		Opcode::SshrImm => binary(DataValueExt::sshr, arg(0), help::shift_amt(ctrl_ty, imm())?)?,
		Opcode::Bitrev => unary(DataValueExt::reverse_bits, arg(0))?,
		Opcode::Bswap => unary(DataValueExt::swap_bytes, arg(0))?,
		Opcode::Clz => unary(DataValueExt::leading_zeros, arg(0))?,
		Opcode::Cls => {
			let count = if arg(0) < DataValueExt::int(0, ctrl_ty)? {
				arg(0).leading_ones()?
			} else {
				arg(0).leading_zeros()?
			};
			assign(DataValueExt::sub(count, DataValueExt::int(1, ctrl_ty)?)?)
		}
		Opcode::Ctz => unary(DataValueExt::trailing_zeros, arg(0))?,
		Opcode::Popcnt => {
			let count = if arg(0).ty().is_int() {
				arg(0).count_ones()?
			} else {
				let lanes = help::extractlanes(&arg(0), ctrl_ty)?
					.into_iter()
					.map(|lane| lane.count_ones())
					.collect::<ValueResult<SimdVec<DataValue>>>()?;
				help::vectorizelanes(&lanes, ctrl_ty)?
			};
			assign(count)
		}

		Opcode::Fcmp => {
			let arg0 = help::extractlanes(&arg(0), ctrl_ty)?;
			let arg1 = help::extractlanes(&arg(1), ctrl_ty)?;

			assign(help::vectorizelanes(
				&(arg0
					.into_iter()
					.zip(arg1.into_iter())
					.map(|(x, y)| {
						DataValue::bool(
							help::fcmp(inst.fp_cond_code().unwrap(), &x, &y).unwrap(),
							ctrl_ty.is_vector(),
							ctrl_ty.lane_type().as_truthy(),
						)
					})
					.collect::<ValueResult<SimdVec<DataValue>>>()?),
				ctrl_ty,
			)?)
		}
		Opcode::Fadd => binary(DataValueExt::add, arg(0), arg(1))?,
		Opcode::Fsub => binary(DataValueExt::sub, arg(0), arg(1))?,
		Opcode::Fmul => binary(DataValueExt::mul, arg(0), arg(1))?,
		Opcode::Fdiv => binary(DataValueExt::sdiv, arg(0), arg(1))?,
		Opcode::Sqrt => unary(DataValueExt::sqrt, arg(0))?,
		Opcode::Fma => {
			let arg0 = help::extractlanes(&arg(0), ctrl_ty)?;
			let arg1 = help::extractlanes(&arg(1), ctrl_ty)?;
			let arg2 = help::extractlanes(&arg(2), ctrl_ty)?;

			assign(help::vectorizelanes(
				&(arg0
					.into_iter()
					.zip(arg1.into_iter())
					.zip(arg2.into_iter())
					.map(|((x, y), z)| DataValueExt::fma(x, y, z))
					.collect::<ValueResult<SimdVec<DataValue>>>()?),
				ctrl_ty,
			)?)
		}
		Opcode::Fneg => unary(DataValueExt::neg, arg(0))?,
		Opcode::Fabs => unary(DataValueExt::abs, arg(0))?,
		Opcode::Fcopysign => binary(DataValueExt::copysign, arg(0), arg(1))?,
		Opcode::Fmin => assign(match (arg(0), arg(1)) {
			(a, _) if a.is_nan()? => a,
			(_, b) if b.is_nan()? => b,
			(a, b) if a.is_zero()? && b.is_zero()? && a.is_negative()? => a,
			(a, b) if a.is_zero()? && b.is_zero()? && b.is_negative()? => b,
			(a, b) => a.smin(b)?,
		}),
		Opcode::Fmax => assign(match (arg(0), arg(1)) {
			(a, _) if a.is_nan()? => a,
			(_, b) if b.is_nan()? => b,
			(a, b) if a.is_zero()? && b.is_zero()? && a.is_negative()? => b,
			(a, b) if a.is_zero()? && b.is_zero()? && b.is_negative()? => a,
			(a, b) => a.smax(b)?,
		}),
		Opcode::Ceil => unary(DataValueExt::ceil, arg(0))?,
		Opcode::Floor => unary(DataValueExt::floor, arg(0))?,
		Opcode::Trunc => unary(DataValueExt::trunc, arg(0))?,
		Opcode::Nearest => unary(DataValueExt::nearest, arg(0))?,

		Opcode::Bitcast | Opcode::ScalarToVector => {
			let input_ty = ictx.type_of(ictx.args()[0]).unwrap();
			let lanes = &if input_ty.is_vector() {
				assert_eq!(
					inst.memflags()
						.expect("byte order flag to be set")
						.endianness(Endianness::Little),
					Endianness::Little,
					"Only little endian bitcasts on vectors are supported"
				);
				help::extractlanes(&arg(0), ctrl_ty)?
			} else {
				help::extractlanes(&arg(0), input_ty)?
					.into_iter()
					.map(|x| DataValue::convert(x, ValueConversionKind::Exact(ctrl_ty.lane_type())))
					.collect::<ValueResult<SimdVec<DataValue>>>()?
			};
			assign(match inst.opcode() {
				Opcode::Bitcast => help::vectorizelanes(lanes, ctrl_ty)?,
				Opcode::ScalarToVector => help::vectorizelanes_all(lanes, ctrl_ty)?,
				_ => unreachable!(),
			})
		}
		Opcode::Ireduce => assign(DataValueExt::convert(
			arg(0),
			ValueConversionKind::Truncate(ctrl_ty),
		)?),
		Opcode::Snarrow | Opcode::Unarrow | Opcode::Uunarrow => {
			let arg0 = help::extractlanes(&arg(0), ctrl_ty)?;
			let arg1 = help::extractlanes(&arg(1), ctrl_ty)?;
			let new_type = ctrl_ty.split_lanes().unwrap();
			let (min, max) = new_type.bounds(inst.opcode() == Opcode::Snarrow);
			let min: DataValue = DataValueExt::int(min as i128, ctrl_ty.lane_type())?;
			let max: DataValue = DataValueExt::int(max as i128, ctrl_ty.lane_type())?;
			let narrow = |mut lane: DataValue| -> ValueResult<DataValue> {
				if inst.opcode() == Opcode::Uunarrow {
					lane = DataValueExt::umax(lane, min.clone())?;
					lane = DataValueExt::umin(lane, max.clone())?;
				} else {
					lane = DataValueExt::smax(lane, min.clone())?;
					lane = DataValueExt::smin(lane, max.clone())?;
				}
				lane = lane.convert(ValueConversionKind::Truncate(new_type.lane_type()))?;
				Ok(lane)
			};
			let new_vec = arg0
				.into_iter()
				.chain(arg1)
				.map(narrow)
				.collect::<ValueResult<Vec<_>>>()?;
			assign(help::vectorizelanes(&new_vec, new_type)?)
		}
		Opcode::Bmask => assign({
			let bool = arg(0);
			let bool_ty = ctrl_ty.as_truthy_pedantic();
			let lanes = help::extractlanes(&bool, bool_ty)?
				.into_iter()
				.map(|lane| lane.convert(ValueConversionKind::Mask(ctrl_ty.lane_type())))
				.collect::<ValueResult<SimdVec<DataValue>>>()?;
			help::vectorizelanes(&lanes, ctrl_ty)?
		}),
		Opcode::Sextend => assign(DataValueExt::convert(
			arg(0),
			ValueConversionKind::SignExtend(ctrl_ty),
		)?),
		Opcode::Uextend => assign(DataValueExt::convert(
			arg(0),
			ValueConversionKind::ZeroExtend(ctrl_ty),
		)?),
		Opcode::Fpromote => assign(DataValueExt::convert(
			arg(0),
			ValueConversionKind::Exact(ctrl_ty),
		)?),
		Opcode::Fdemote => assign(DataValueExt::convert(
			arg(0),
			ValueConversionKind::RoundNearestEven(ctrl_ty),
		)?),
		Opcode::Shuffle => {
			let mask = imm().into_array()?;
			let a = DataValueExt::into_array(&arg(0))?;
			let b = DataValueExt::into_array(&arg(1))?;
			let mut new = [0u8; 16];
			for i in 0..mask.len() {
				if (mask[i] as usize) < a.len() {
					new[i] = a[mask[i] as usize];
				} else if (mask[i] as usize - a.len()) < b.len() {
					new[i] = b[mask[i] as usize - a.len()];
				} // else leave as 0.
			}
			assign(DataValueExt::vector(new, types::I8X16)?)
		}
		Opcode::Swizzle => {
			let x = DataValueExt::into_array(&arg(0))?;
			let s = DataValueExt::into_array(&arg(1))?;
			let mut new = [0u8; 16];
			for i in 0..new.len() {
				if (s[i] as usize) < new.len() {
					new[i] = x[s[i] as usize];
				} // else leave as 0
			}
			assign(DataValueExt::vector(new, types::I8X16)?)
		}
		Opcode::Splat => assign(help::splat(ctrl_ty, arg(0))?),
		Opcode::Insertlane => {
			let idx = imm().into_int_unsigned()? as usize;
			let mut vector = help::extractlanes(&arg(0), ctrl_ty)?;
			vector[idx] = arg(1);
			assign(help::vectorizelanes(&vector, ctrl_ty)?)
		}
		Opcode::Extractlane => {
			let idx = imm().into_int_unsigned()? as usize;
			let lanes = help::extractlanes(&arg(0), ctrl_ty)?;
			assign(lanes[idx].clone())
		}
		Opcode::VhighBits => {
			// `ctrl_ty` controls the return type for this, so the input type
			// must be retrieved via `ictx`.
			let vector_type = ictx.type_of(ictx.args()[0]).unwrap();
			let a = help::extractlanes(&arg(0), vector_type)?;
			let mut result: u128 = 0;
			for (i, val) in a.into_iter().enumerate() {
				let val = val.reverse_bits()?.into_int_unsigned()?; // MSB -> LSB
				result |= (val & 1) << i;
			}
			assign(DataValueExt::int(result as i128, ctrl_ty)?)
		}
		Opcode::VanyTrue => {
			let lane_ty = ctrl_ty.lane_type();
			let init = DataValue::bool(false, true, lane_ty)?;
			let any = help::fold_vector(arg(0), ctrl_ty, init.clone(), |acc, lane| acc.or(lane))?;
			assign(DataValue::bool(any != init, false, types::I8)?)
		}
		Opcode::VallTrue => assign(DataValue::bool(
			!(arg(0).iter_lanes(ctrl_ty)?.try_fold(false, |acc, lane| {
				Ok::<bool, ValueError>(acc | lane.is_zero()?)
			})?),
			false,
			types::I8,
		)?),
		Opcode::SwidenLow | Opcode::SwidenHigh | Opcode::UwidenLow | Opcode::UwidenHigh => {
			let new_type = ctrl_ty.merge_lanes().unwrap();
			let conv_type = match inst.opcode() {
				Opcode::SwidenLow | Opcode::SwidenHigh => {
					ValueConversionKind::SignExtend(new_type.lane_type())
				}
				Opcode::UwidenLow | Opcode::UwidenHigh => {
					ValueConversionKind::ZeroExtend(new_type.lane_type())
				}
				_ => unreachable!(),
			};
			let vec_iter = help::extractlanes(&arg(0), ctrl_ty)?.into_iter();
			let new_vec = match inst.opcode() {
				Opcode::SwidenLow | Opcode::UwidenLow => vec_iter
					.take(new_type.lane_count() as usize)
					.map(|lane| lane.convert(conv_type.clone()))
					.collect::<ValueResult<Vec<_>>>()?,
				Opcode::SwidenHigh | Opcode::UwidenHigh => vec_iter
					.skip(new_type.lane_count() as usize)
					.map(|lane| lane.convert(conv_type.clone()))
					.collect::<ValueResult<Vec<_>>>()?,
				_ => unreachable!(),
			};
			assign(help::vectorizelanes(&new_vec, new_type)?)
		}
		Opcode::FcvtToUint | Opcode::FcvtToSint => {
			// NaN check
			if arg(0).is_nan()? {
				return Ok(ControlFlow::Trap(CraneliftTrap::User(
					TrapCode::BadConversionToInteger,
				)));
			}
			let x = arg(0).into_float()? as i128;
			let is_signed = inst.opcode() == Opcode::FcvtToSint;
			let (min, max) = ctrl_ty.bounds(is_signed);

			let overflow = if is_signed {
				x < (min as i128) || x > (max as i128)
			} else {
				x < 0 || (x as u128) > max
			};

			// bounds check
			if overflow {
				return Ok(ControlFlow::Trap(CraneliftTrap::User(
					TrapCode::IntegerOverflow,
				)));
			}
			// perform the conversion.
			assign(DataValueExt::int(x, ctrl_ty)?)
		}
		Opcode::FcvtToUintSat | Opcode::FcvtToSintSat => {
			let in_ty = ictx.type_of(ictx.args()[0]).unwrap();
			let cvt = |x: DataValue| -> ValueResult<DataValue> {
				// NaN check
				if x.is_nan()? {
					DataValue::int(0, ctrl_ty.lane_type())
				} else {
					let is_signed = inst.opcode() == Opcode::FcvtToSintSat;
					let (min, max) = ctrl_ty.bounds(is_signed);
					let x = x.into_float()? as i128;

					let x = if is_signed {
						let x = i128::max(x, min as i128);
						i128::min(x, max as i128)
					} else {
						let x = if x < 0 { 0 } else { x };
						let x = u128::min(x as u128, max);
						x as i128
					};

					DataValue::int(x, ctrl_ty.lane_type())
				}
			};

			let x = help::extractlanes(&arg(0), in_ty)?;

			assign(help::vectorizelanes(
				&x.into_iter()
					.map(cvt)
					.collect::<ValueResult<SimdVec<DataValue>>>()?,
				ctrl_ty,
			)?)
		}
		Opcode::FcvtFromUint | Opcode::FcvtFromSint => {
			let x = help::extractlanes(&arg(0), ictx.type_of(ictx.args()[0]).unwrap())?;
			let bits = |x: DataValue| -> ValueResult<u64> {
				Ok(match ctrl_ty.lane_type() {
					types::F32 => (if inst.opcode() == Opcode::FcvtFromUint {
						x.into_int_unsigned()? as f32
					} else {
						x.into_int_signed()? as f32
					})
					.to_bits() as u64,
					types::F64 => (if inst.opcode() == Opcode::FcvtFromUint {
						x.into_int_unsigned()? as f64
					} else {
						x.into_int_signed()? as f64
					})
					.to_bits(),
					_ => unimplemented!("unexpected conversion to {:?}", ctrl_ty.lane_type()),
				})
			};
			assign(help::vectorizelanes(
				&x.into_iter()
					.map(|x| DataValue::float(bits(x)?, ctrl_ty.lane_type()))
					.collect::<ValueResult<SimdVec<DataValue>>>()?,
				ctrl_ty,
			)?)
		}
		Opcode::FvpromoteLow => {
			let in_ty = ictx.type_of(ictx.args()[0]).unwrap();
			assert_eq!(in_ty, types::F32X4);
			let out_ty = types::F64X2;
			let x = help::extractlanes(&arg(0), in_ty)?;
			assign(help::vectorizelanes(
				&x[..(out_ty.lane_count() as usize)]
					.iter()
					.map(|x| {
						DataValue::convert(
							x.to_owned(),
							ValueConversionKind::Exact(out_ty.lane_type()),
						)
					})
					.collect::<ValueResult<SimdVec<DataValue>>>()?,
				out_ty,
			)?)
		}
		Opcode::Fvdemote => {
			let in_ty = ictx.type_of(ictx.args()[0]).unwrap();
			assert_eq!(in_ty, types::F64X2);
			let out_ty = types::F32X4;
			let x = help::extractlanes(&arg(0), in_ty)?;
			let x = &mut x
				.into_iter()
				.map(|x| {
					DataValue::convert(x, ValueConversionKind::RoundNearestEven(out_ty.lane_type()))
				})
				.collect::<ValueResult<SimdVec<DataValue>>>()?;
			// zero the high bits.
			for _ in 0..(out_ty.lane_count() as usize - x.len()) {
				x.push(DataValue::float(0, out_ty.lane_type())?);
			}
			assign(help::vectorizelanes(x, out_ty)?)
		}
		Opcode::Isplit => assign_multiple(&[
			DataValueExt::convert(arg(0), ValueConversionKind::Truncate(types::I64))?,
			DataValueExt::convert(arg(0), ValueConversionKind::ExtractUpper(types::I64))?,
		]),
		Opcode::Iconcat => assign(DataValueExt::concat(arg(0), arg(1))?),
		Opcode::AtomicRmw => {
			let op = inst.atomic_rmw_op().unwrap();
			let val = arg(1);
			let addr = arg(0).into_int_unsigned()? as u64;
			let mem_flags = inst.memflags().expect("instruction to have memory flags");
			let loaded = Address::try_from(addr)
				.and_then(|addr| state.checked_load(addr, ctrl_ty, mem_flags));
			let prev_val = match loaded {
				Ok(v) => v,
				Err(e) => return Ok(ControlFlow::Trap(CraneliftTrap::User(memerror_to_trap(e)))),
			};
			let prev_val_to_assign = prev_val.clone();
			let replace = match op {
				AtomicRmwOp::Xchg => Ok(val),
				AtomicRmwOp::Add => DataValueExt::add(prev_val, val),
				AtomicRmwOp::Sub => DataValueExt::sub(prev_val, val),
				AtomicRmwOp::And => DataValueExt::and(prev_val, val),
				AtomicRmwOp::Or => DataValueExt::or(prev_val, val),
				AtomicRmwOp::Xor => DataValueExt::xor(prev_val, val),
				AtomicRmwOp::Nand => DataValueExt::and(prev_val, val).and_then(DataValue::not),
				AtomicRmwOp::Smax => DataValueExt::smax(prev_val, val),
				AtomicRmwOp::Smin => DataValueExt::smin(prev_val, val),
				AtomicRmwOp::Umax => DataValueExt::umax(val, prev_val),
				AtomicRmwOp::Umin => DataValueExt::umin(val, prev_val),
			}?;
			let stored = Address::try_from(addr)
				.and_then(|addr| state.checked_store(addr, replace, mem_flags));
			assign_or_memtrap(stored.map(|_| prev_val_to_assign))
		}
		Opcode::AtomicCas => {
			let addr = arg(0).into_int_unsigned()? as u64;
			let mem_flags = inst.memflags().expect("instruction to have memory flags");
			let loaded = Address::try_from(addr)
				.and_then(|addr| state.checked_load(addr, ctrl_ty, mem_flags));
			let loaded_val = match loaded {
				Ok(v) => v,
				Err(e) => return Ok(ControlFlow::Trap(CraneliftTrap::User(memerror_to_trap(e)))),
			};
			let expected_val = arg(1);
			let val_to_assign = if loaded_val == expected_val {
				let val_to_store = arg(2);
				Address::try_from(addr)
					.and_then(|addr| state.checked_store(addr, val_to_store, mem_flags))
					.map(|_| loaded_val)
			} else {
				Ok(loaded_val)
			};
			assign_or_memtrap(val_to_assign)
		}
		Opcode::AtomicLoad => {
			let load_ty = ictx.controlling_type().unwrap();
			let addr = arg(0).into_int_unsigned()? as u64;
			let mem_flags = inst.memflags().expect("instruction to have memory flags");
			// We are doing a regular load here, this isn't actually thread safe.
			assign_or_memtrap(
				Address::try_from(addr)
					.and_then(|addr| state.checked_load(addr, load_ty, mem_flags)),
			)
		}
		Opcode::AtomicStore => {
			let val = arg(0);
			let addr = arg(1).into_int_unsigned()? as u64;
			let mem_flags = inst.memflags().expect("instruction to have memory flags");
			// We are doing a regular store here, this isn't actually thread safe.
			continue_or_memtrap(
				Address::try_from(addr).and_then(|addr| state.checked_store(addr, val, mem_flags)),
			)
		}
		Opcode::Fence => {
			// The interpreter always runs in a single threaded context, so we don't
			// actually need to emit a fence here.
			ControlFlow::Continue
		}
		Opcode::SqmulRoundSat => {
			let lane_type = ctrl_ty.lane_type();
			let double_width = ctrl_ty.double_width().unwrap().lane_type();
			let arg0 = help::extractlanes(&arg(0), ctrl_ty)?;
			let arg1 = help::extractlanes(&arg(1), ctrl_ty)?;
			let (min, max) = lane_type.bounds(true);
			let min: DataValue = DataValueExt::int(min as i128, double_width)?;
			let max: DataValue = DataValueExt::int(max as i128, double_width)?;
			let new_vec = arg0
				.into_iter()
				.zip(arg1.into_iter())
				.map(|(x, y)| {
					let x = x.into_int_signed()?;
					let y = y.into_int_signed()?;
					// temporarily double width of the value to avoid overflow.
					let z: DataValue = DataValueExt::int(
						(x * y + (1 << (lane_type.bits() - 2))) >> (lane_type.bits() - 1),
						double_width,
					)?;
					// check bounds, saturate, and truncate to correct width.
					let z = DataValueExt::smin(z, max.clone())?;
					let z = DataValueExt::smax(z, min.clone())?;
					let z = z.convert(ValueConversionKind::Truncate(lane_type))?;
					Ok(z)
				})
				.collect::<ValueResult<SimdVec<_>>>()?;
			assign(help::vectorizelanes(&new_vec, ctrl_ty)?)
		}
		Opcode::IaddPairwise => assign(help::binary_pairwise(
			arg(0),
			arg(1),
			ctrl_ty,
			DataValueExt::add,
		)?),
		// Unimplemented ///////////////////////////////////////////////////////
		Opcode::DynamicStackAddr => unimplemented!("`OpCode::DynamicStackSlot` not supported"),
		Opcode::DynamicStackLoad => unimplemented!("`OpCode::DynamicStackLoad` not supported"),
		Opcode::DynamicStackStore => unimplemented!("`OpCode::DynamicStackStore` not supported"),
		Opcode::ExtractVector => unimplemented!("`OpCode::ExtractVector` not supported"),
		Opcode::GetFramePointer => unimplemented!("`OpCode::GetFramePointer` not supported"),
		Opcode::GetStackPointer => unimplemented!("`OpCode::GetStackPointer` not supported"),
		Opcode::GetReturnAddress => unimplemented!("`OpCode::GetReturnAddress` not supported"),
		Opcode::IsNull => unimplemented!("`OpCode::IsNull` not supported"),
		Opcode::IsInvalid => unimplemented!("`OpCode::IsInvalid` not supported"),
		Opcode::Null => unimplemented!("`OpCode:Null` not supported"),
		Opcode::X86Pshufb => unimplemented!("`OpCode::X86Pshufb` not supported"),
		Opcode::X86Blendv => unimplemented!("`OpCode::X86Blendv` not supported"),
		Opcode::X86Pmulhrsw => unimplemented!("`OpCode::X86Pmulhrsw` not supported"),
		Opcode::X86Pmaddubsw => unimplemented!("`OpCode::X86Pmaddubsw` not supported"),
		Opcode::X86Cvtt2dq => unimplemented!("`OpCode::X86Cvtt2dq` not supported"),
	})
}

type SimdVec<DataValue> = SmallVec<[DataValue; 4]>;