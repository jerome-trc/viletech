//! A slightly-modified [Cranelift Interpreter].
//!
//! [Cranelift Interpreter]: cranelift_interpreter::interpreter::Interpreter

use cranelift::{
	codegen::{data_value::DataValue, ir::Function},
	prelude::Block,
};
use cranelift_interpreter::{
	environment::FuncIndex,
	instruction::DfgInstructionContext,
	interpreter::{FuelResult, InterpreterError, InterpreterState},
	state::State,
	step::{step, ControlFlow},
};

/// The Cranelift interpreter; this contains some high-level functions to control the interpreter's
/// flow. The interpreter state is defined separately (see [InterpreterState]) as the execution
/// semantics for each Cranelift instruction (see [step]).
pub struct Interpreter<'a> {
	state: InterpreterState<'a>,
	fuel: Option<u64>,
}

impl<'a> Interpreter<'a> {
	#[must_use]
	pub fn new(state: InterpreterState<'a>) -> Self {
		Self { state, fuel: None }
	}

	#[must_use]
	pub fn into_inner(self) -> InterpreterState<'a> {
		self.state
	}

	/// The `fuel` mechanism sets a number of instructions that
	/// the interpreter can execute before stopping. If this
	/// value is `None` (the default), no limit is imposed.
	#[must_use]
	pub fn with_fuel(self, fuel: Option<u64>) -> Self {
		Self { fuel, ..self }
	}

	/// Call a function by name; this is a helpful proxy for [Interpreter::call_by_index].
	pub fn call_by_name(
		&mut self,
		func_name: &str,
		arguments: &[DataValue],
	) -> Result<ControlFlow<'a>, InterpreterError> {
		let index = self
			.state
			.functions
			.index_of(func_name)
			.ok_or_else(|| InterpreterError::UnknownFunctionName(func_name.to_string()))?;
		self.call_by_index(index, arguments)
	}

	/// Call a function by its index in the [FunctionStore]; this is a proxy for
	/// `Interpreter::call`.
	pub fn call_by_index(
		&mut self,
		index: FuncIndex,
		arguments: &[DataValue],
	) -> Result<ControlFlow<'a>, InterpreterError> {
		match self.state.functions.get_by_index(index) {
			None => Err(InterpreterError::UnknownFunctionIndex(index)),
			Some(func) => self.call(func, arguments),
		}
	}

	/// Interpret a call to a [Function] given its [DataValue] arguments.
	fn call(
		&mut self,
		function: &'a Function,
		arguments: &[DataValue],
	) -> Result<ControlFlow<'a>, InterpreterError> {
		let first_block = function
			.layout
			.blocks()
			.next()
			.expect("to have a first block");
		let parameters = function.dfg.block_params(first_block);
		self.state.push_frame(function);
		self.state
			.current_frame_mut()
			.set_all(parameters, arguments.to_vec());

		self.block(first_block)
	}

	/// Interpret a [Block] in a [Function]. This drives the interpretation over sequences of
	/// instructions, which may continue in other blocks, until the function returns.
	fn block(&mut self, block: Block) -> Result<ControlFlow<'a>, InterpreterError> {
		let function = self.state.current_frame_mut().function();
		let layout = &function.layout;
		let mut maybe_inst = layout.first_inst(block);
		while let Some(inst) = maybe_inst {
			if self.consume_fuel() == FuelResult::Stop {
				return Err(InterpreterError::FuelExhausted);
			}

			let inst_context = DfgInstructionContext::new(inst, &function.dfg);
			match step(&mut self.state, inst_context)? {
				ControlFlow::Assign(values) => {
					self.state
						.current_frame_mut()
						.set_all(function.dfg.inst_results(inst), values.to_vec());
					maybe_inst = layout.next_inst(inst)
				}
				ControlFlow::Continue => maybe_inst = layout.next_inst(inst),
				ControlFlow::ContinueAt(block, block_arguments) => {
					self.state
						.current_frame_mut()
						.set_all(function.dfg.block_params(block), block_arguments.to_vec());
					maybe_inst = layout.first_inst(block)
				}
				ControlFlow::Call(called_function, arguments) => {
					match self.call(called_function, &arguments)? {
						ControlFlow::Return(rets) => {
							self.state
								.current_frame_mut()
								.set_all(function.dfg.inst_results(inst), rets.to_vec());
							maybe_inst = layout.next_inst(inst)
						}
						ControlFlow::Trap(trap) => return Ok(ControlFlow::Trap(trap)),
						cf => {
							panic!("invalid control flow after call: {:?}", cf)
						}
					}
				}
				ControlFlow::ReturnCall(callee, args) => {
					self.state.pop_frame();

					return match self.call(callee, &args)? {
						ControlFlow::Return(rets) => Ok(ControlFlow::Return(rets)),
						ControlFlow::Trap(trap) => Ok(ControlFlow::Trap(trap)),
						cf => {
							panic!("invalid control flow after return_call: {:?}", cf)
						}
					};
				}
				ControlFlow::Return(returned_values) => {
					self.state.pop_frame();
					return Ok(ControlFlow::Return(returned_values));
				}
				ControlFlow::Trap(trap) => return Ok(ControlFlow::Trap(trap)),
			}
		}
		Err(InterpreterError::Unreachable)
	}

	fn consume_fuel(&mut self) -> FuelResult {
		match self.fuel {
			Some(0) => FuelResult::Stop,
			Some(ref mut n) => {
				*n -= 1;
				FuelResult::Continue
			}

			// We do not have fuel enabled, so unconditionally continue
			None => FuelResult::Continue,
		}
	}
}
