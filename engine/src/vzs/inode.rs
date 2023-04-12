//! "Instruction" nodes are executed by the interpreter.
//!
//! This architecture is inspired by [daScript], which compiles an AST into what it
//! calls "simulation nodes" to interpret them, as opposed to interpreting the AST
//! itself, or bytecode. What differentiates VZS from daScript is the use of
//! Rust enums and recursion schemes, rather than nodes with vtables.
//!
//! This is entirely experimental, to see what merits it has for VZS's purposes.
//!
//! [daScript]: https://dascript.org/

/*

The recursion algorithm used comes from this article:
https://recursion.wtf/posts/rust_schemes/

Q:
- Is dynamic dispatch better?
Existence of the `enum_dispatch` crate indicates that it would likely not be.
- How reliably are words being passed by register between nodes?

*/

#![allow(dead_code)] // TODO: Remove

mod builder;
mod detail;
mod eval;
mod inst;
mod map;

use std::{ffi::c_void, mem::MaybeUninit, sync::Arc};

use crate::vzs::InstPtr;

use super::{
	abi::{Abi, QWord},
	module::JitModule,
	Runtime, TypeInfo, MAX_PARAMS,
};

#[allow(unused)]
pub(super) use self::{builder::*, detail::*, inst::*};

/// All of a [`Function`](super::Function)'s instructions.
#[derive(Debug)]
pub(super) struct Tree {
	code: Code,
}

#[derive(Debug)]
enum Code {
	INodes {
		/// Elements are guaranteed to be in topologically-sorted order.
		nodes: Vec<INodeOwning>,
	},
	Jit {
		/// Never try to de-allocate this.
		code: *const c_void,
		/// Ensure the JIT-compiled code lives long enough.
		#[allow(unused)]
		module: Arc<JitModule>,
	},
}

impl Tree {
	#[must_use]
	pub(self) fn new(nodes: Vec<INodeOwning>) -> Arc<Self> {
		Arc::new(Self {
			code: Code::INodes { nodes },
		})
	}

	pub(super) fn eval<A: Abi, R: Abi>(&self, ctx: &mut Runtime, args: A) -> R {
		match &self.code {
			Code::Jit { code, .. } => {
				let a = args.to_words();
				// SAFETY: For the public interface to have gotten to this point,
				// it must have performed a check to verify that signature types match.
				let func = unsafe { std::mem::transmute::<_, fn(A::Repr) -> R::Repr>(code) };
				let r = func(a);
				R::from_words(r)
			}
			Code::INodes { nodes } => {
				#[must_use]
				fn on_return<R: Abi>(ctx: &mut Runtime, icache_len: usize) -> R {
					ctx.icache.0.truncate(icache_len);
					unsafe { ctx.stack.pop() }
				}

				let icache_len = ctx.icache.0.len();

				if ctx.iptr == InstPtr::Panic {
					return on_return(ctx, icache_len);
				}

				ctx.iptr = InstPtr::Running(Index(0));

				unsafe {
					ctx.stack.push(args);
				}

				debug_assert!(
					ctx.icache.0.capacity() >= (icache_len + nodes.len()),
					"VZS i-cache lacks capacity."
				);

				while let InstPtr::Running(index) = &mut ctx.iptr {
					let node = &nodes[index.0];
					index.0 += 1;

					let alg_res = {
						// SAFETY:
						// Each node is known to be present, and is only referenced once.
						let inst = node.inst.map(|Index(x)| unsafe {
							let r = ctx.icache.0.get_unchecked_mut(x);
							let maybe_uninit = std::mem::replace(r, MaybeUninit::uninit());
							maybe_uninit.assume_init()
						});

						inst.eval(ctx)
					};

					ctx.icache.0.push(MaybeUninit::new(alg_res));
				}

				if ctx.iptr == InstPtr::Panic {
					return on_return(ctx, icache_len);
				}

				ctx.iptr = InstPtr::None;

				// Function calls never directly emit a valid word. Leave a
				// type-annotated discard here so it's clear what's happening.
				let _: QWord = unsafe {
					let maybe_uninit = std::mem::replace(
						ctx.icache.0.get_unchecked_mut(Index::default().0),
						MaybeUninit::uninit(),
					);

					maybe_uninit.assume_init()
				};

				on_return(ctx, icache_len)
			}
		}
	}
}

#[derive(Debug)]
pub(super) struct INode<K: NodeKind> {
	line_info: LineInfo,
	inst: Instruction<K>,
}

#[derive(Debug)]
pub(super) enum Instruction<K: NodeKind> {
	/// Evaluation emits a pointer.
	Allocate(K::HandleT<TypeInfo>),
	/// Evaluation emits a single q-word result.
	BinOp { l: K::Index, r: K::Index, op: BinOp },
	/// Evaluation emits [`QWord::invalid`], regardless of the function's actual
	/// return signature. Return values are left on the stack afterwards.
	Call {
		func: K::ArcT<Tree>,
		args: Box<[K::Index; MAX_PARAMS]>,
		arg_c: u8,
	},
	/// Sets the runtime's instruction pointer to the contained index.
	/// Evaluation emits [`QWord::invalid`].
	Jump(Index),
	/// Evaluation emits the contained value.
	Immediate(QWord),
	/// Evaluation emits [`QWord::invalid`].
	NoOp,
	/// Sets the runtime's instruction pointer to [`InstPtr::Panic`].
	/// Evaluation emits [`QWord::invalid`].
	Panic,
	/// Evaluation emits the q-word at the top of the stack.
	Pop,
	/// Evaluation emits [`QWord::invalid`].
	Push(K::Index),
	/// Sets the runtime's instruction pointer to [`InstPtr::Return`].
	/// Evaluation emits [`QWord::invalid`].
	Return,
}

pub(super) type INodeOwning = INode<OwningNode>;
pub(super) type InstRef<'i> = Instruction<RefNode<'i>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineInfo {
	line: u32,
	col: u32,
}

impl LineInfo {
	#[must_use]
	pub fn new(line: u32, col: u32) -> Self {
		Self { line, col }
	}

	#[must_use]
	pub fn line(&self) -> u32 {
		self.line
	}

	#[must_use]
	pub fn column(&self) -> u32 {
		self.col
	}
}
