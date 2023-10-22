use cranelift::{
	codegen::{
		data_value::DataValue,
		ir::{self, ArgumentPurpose, Endianness, FuncRef, GlobalValue, LibCall, StackSlot},
	},
	prelude::{ExternalName, GlobalValueData, MemFlags, Type},
};
use cranelift_interpreter::{
	address::{Address, AddressFunctionEntry, AddressRegion, AddressSize},
	frame::Frame,
	interpreter::LibCallHandler,
	state::{InterpreterFunctionRef, MemoryError},
	value::DataValueExt,
};

use crate::Compiler;

/// Adapted from [`cranelift_interpreter::interpreter::InterpreterState`].
#[derive(Debug)]
pub(crate) struct Interpreter<'c> {
	pub(crate) compiler: &'c Compiler,
	pub(crate) native_endianness: Endianness,

	pub(crate) frame_stack: Vec<Frame<'c>>,
	/// Number of bytes from the bottom of the stack where the current frame's stack space is
	pub(crate) frame_offset: usize,
	pub(crate) stack: Vec<u8>,
	pub(crate) pinned_reg: DataValue,
}

impl<'c> Interpreter<'c> {
	#[must_use]
	pub(crate) fn new(compiler: &'c Compiler) -> Self {
		let native_endianness = if cfg!(target_endian = "little") {
			Endianness::Little
		} else {
			Endianness::Big
		};

		Self {
			compiler,
			native_endianness,

			frame_stack: vec![],
			frame_offset: 0,
			stack: Vec::with_capacity(1024),
			pinned_reg: DataValue::I64(0),
		}
	}
}

impl<'c> cranelift_interpreter::state::State<'c> for Interpreter<'c> {
	fn get_function(&self, func_ref: FuncRef) -> Option<&'c ir::Function> {
		let curr_ir = self.get_current_function();
		let ext_data = curr_ir.stencil.dfg.ext_funcs.get(func_ref).unwrap();

		let ExternalName::User(uenr) = ext_data.name else {
			unimplemented!()
		};

		let uen = curr_ir.params.user_named_funcs().get(uenr).unwrap();

		Some(&self.compiler.ir[uen.index as usize].1)
	}

	fn get_current_function(&self) -> &'c ir::Function {
		self.current_frame().function()
	}

	fn get_libcall_handler(&self) -> LibCallHandler {
		super::help::handle_libcall
	}

	fn push_frame(&mut self, function: &'c ir::Function) {
		if let Some(frame) = self.frame_stack.iter().last() {
			self.frame_offset += frame.function().fixed_stack_size() as usize;
		}

		// Grow the stack by the space necessary for this frame
		self.stack
			.extend(std::iter::repeat(0).take(function.fixed_stack_size() as usize));

		self.frame_stack.push(Frame::new(function));
	}

	fn pop_frame(&mut self) {
		if let Some(frame) = self.frame_stack.pop() {
			// Shorten the stack after exiting the frame
			self.stack
				.truncate(self.stack.len() - frame.function().fixed_stack_size() as usize);

			// Reset frame_offset to the start of this function
			if let Some(frame) = self.frame_stack.iter().last() {
				self.frame_offset -= frame.function().fixed_stack_size() as usize;
			}
		}
	}

	fn current_frame(&self) -> &Frame<'c> {
		let num_frames = self.frame_stack.len();
		match num_frames {
			0 => panic!("unable to retrieve the current frame because no frames were pushed"),
			_ => &self.frame_stack[num_frames - 1],
		}
	}

	fn current_frame_mut(&mut self) -> &mut Frame<'c> {
		let num_frames = self.frame_stack.len();
		match num_frames {
			0 => panic!("unable to retrieve the current frame because no frames were pushed"),
			_ => &mut self.frame_stack[num_frames - 1],
		}
	}

	fn stack_address(
		&self,
		size: AddressSize,
		slot: StackSlot,
		offset: u64,
	) -> Result<Address, MemoryError> {
		let stack_slots = &self.get_current_function().sized_stack_slots;
		let stack_slot = &stack_slots[slot];

		// offset must be `0 <= Offset < sizeof(SS)`
		if offset >= stack_slot.size as u64 {
			return Err(MemoryError::InvalidOffset {
				offset,
				max: stack_slot.size as u64,
			});
		}

		// Calculate the offset from the current frame to the requested stack slot
		let slot_offset: u64 = stack_slots
			.keys()
			.filter(|k| k < &slot)
			.map(|k| stack_slots[k].size as u64)
			.sum();

		let final_offset = self.frame_offset as u64 + slot_offset + offset;
		Address::from_parts(size, AddressRegion::Stack, 0, final_offset)
	}

	fn checked_load(
		&self,
		addr: Address,
		ty: Type,
		mem_flags: MemFlags,
	) -> Result<DataValue, MemoryError> {
		let load_size = ty.bytes() as usize;
		let addr_start = addr.offset as usize;
		let addr_end = addr_start + load_size;

		let src = match addr.region {
			AddressRegion::Stack => {
				if addr_end > self.stack.len() {
					return Err(MemoryError::OutOfBoundsLoad { addr, load_size });
				}

				&self.stack[addr_start..addr_end]
			}
			_ => unimplemented!(),
		};

		// Aligned flag is set and address is not aligned for the given type
		if mem_flags.aligned() && addr_start % load_size != 0 {
			return Err(MemoryError::MisalignedLoad { addr, load_size });
		}

		Ok(match mem_flags.endianness(self.native_endianness) {
			Endianness::Big => DataValue::read_from_slice_be(src, ty),
			Endianness::Little => DataValue::read_from_slice_le(src, ty),
		})
	}

	fn checked_store(
		&mut self,
		addr: Address,
		v: DataValue,
		mem_flags: MemFlags,
	) -> Result<(), MemoryError> {
		let store_size = v.ty().bytes() as usize;
		let addr_start = addr.offset as usize;
		let addr_end = addr_start + store_size;

		let dst = match addr.region {
			AddressRegion::Stack => {
				if addr_end > self.stack.len() {
					return Err(MemoryError::OutOfBoundsStore { addr, store_size });
				}

				&mut self.stack[addr_start..addr_end]
			}
			_ => unimplemented!(),
		};

		// Aligned flag is set and address is not aligned for the given type
		if mem_flags.aligned() && addr_start % store_size != 0 {
			return Err(MemoryError::MisalignedStore { addr, store_size });
		}

		match mem_flags.endianness(self.native_endianness) {
			Endianness::Big => v.write_to_slice_be(dst),
			Endianness::Little => v.write_to_slice_le(dst),
		}

		Ok(())
	}

	fn function_address(
		&self,
		size: AddressSize,
		name: &ExternalName,
	) -> Result<Address, MemoryError> {
		let curr_func = self.get_current_function();

		let (entry, index) = match name {
			ExternalName::User(username) => {
				let uen = &curr_func.params.user_named_funcs()[*username];
				(AddressFunctionEntry::UserFunction, uen.index)
			}
			ExternalName::LibCall(libcall) => {
				// We don't properly have a "libcall" store, but we can use `LibCall::all()`
				// and index into that.
				let index = LibCall::all_libcalls()
					.iter()
					.position(|lc| lc == libcall)
					.unwrap();

				(AddressFunctionEntry::LibCall, index as u32)
			}
			_ => unimplemented!("function_address: {:?}", name),
		};

		Address::from_parts(size, AddressRegion::Function, entry as u64, index as u64)
	}

	fn get_function_from_address(&self, address: Address) -> Option<InterpreterFunctionRef<'c>> {
		Some(InterpreterFunctionRef::Function(
			&self.compiler.ir[address.offset as usize].1,
		))
	}

	/// Non-recursively resolves a global value until its address is found
	fn resolve_global_value(&self, gv: GlobalValue) -> Result<DataValue, MemoryError> {
		// Resolving a Global Value is a "pointer" chasing operation that lends itself to
		// using a recursive solution. However, resolving this in a recursive manner
		// is a bad idea because it's very easy to add a bunch of global values and
		// blow up the call stack.
		//
		// Adding to the challenges of this, is that the operations possible with GlobalValues
		// mean that we cannot use a simple loop to resolve each global value, we must keep
		// a pending list of operations.

		// These are the possible actions that we can perform
		#[derive(Debug)]
		enum ResolveAction {
			Resolve(GlobalValue),
			/// Perform an add on the current address
			Add(DataValue),
			/// Load From the current address and replace it with the loaded value
			Load {
				/// Offset added to the base pointer before doing the load.
				offset: i32,

				/// Type of the loaded value.
				global_type: Type,
			},
		}

		let func = self.get_current_function();

		// We start with a sentinel value that will fail if we try to load / add to it
		// without resolving the base GV First.
		let mut current_val = DataValue::I8(0);
		let mut action_stack = vec![ResolveAction::Resolve(gv)];

		loop {
			match action_stack.pop() {
				Some(ResolveAction::Resolve(gv)) => match func.global_values[gv] {
					GlobalValueData::VMContext => {
						// Fetch the VMContext value from the values of the first block in the function
						let index = func
							.signature
							.params
							.iter()
							.enumerate()
							.find(|(_, p)| p.purpose == ArgumentPurpose::VMContext)
							.map(|(i, _)| i)
							// This should be validated by the verifier
							.expect("No VMCtx argument was found, but one is referenced");

						let first_block =
							func.layout.blocks().next().expect("to have a first block");
						let vmctx_value = func.dfg.block_params(first_block)[index];
						current_val = self.current_frame().get(vmctx_value).clone();
					}
					GlobalValueData::Load {
						base,
						offset,
						global_type,
						..
					} => {
						action_stack.push(ResolveAction::Load {
							offset: offset.into(),
							global_type,
						});
						action_stack.push(ResolveAction::Resolve(base));
					}
					GlobalValueData::IAddImm {
						base,
						offset,
						global_type,
					} => {
						let offset: i64 = offset.into();
						let dv = DataValue::int(offset as i128, global_type)
							.map_err(|_| MemoryError::InvalidAddressType(global_type))?;
						action_stack.push(ResolveAction::Add(dv));
						action_stack.push(ResolveAction::Resolve(base));
					}
					GlobalValueData::Symbol { .. } => unimplemented!(),
					GlobalValueData::DynScaleTargetConst { .. } => unimplemented!(),
				},
				Some(ResolveAction::Add(dv)) => {
					current_val = current_val
						.add(dv.clone())
						.map_err(|_| MemoryError::InvalidAddress(dv))?;
				}
				Some(ResolveAction::Load {
					offset,
					global_type,
				}) => {
					let mut addr = Address::try_from(current_val)?;
					let mem_flags = MemFlags::trusted();
					// We can forego bounds checking here since its performed in `checked_load`
					addr.offset += offset as u64;
					current_val = self.checked_load(addr, global_type, mem_flags)?;
				}

				// We are done resolving this, return the current value
				None => return Ok(current_val),
			}
		}
	}

	fn get_pinned_reg(&self) -> DataValue {
		self.pinned_reg.clone()
	}

	fn set_pinned_reg(&mut self, v: DataValue) {
		self.pinned_reg = v;
	}
}
