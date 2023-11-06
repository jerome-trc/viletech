//! Compiler intrinsic functions.
//!
//! Some are runtime-only, some are const-eval only, some support both.

use cranelift::{
	codegen::ir::{ArgumentExtension, ArgumentPurpose},
	prelude::AbiParam,
};
use doomfront::rowan::ast::AstNode;

use crate::{
	ast,
	compile::RunTimeNativeFunc,
	front::{
		sema::{CEval, SemaContext},
		sym::{self, Location, Symbol, SymbolId},
		tsys::{self, TypeDef},
	},
	interop,
	issue::{self, Issue},
	runtime::Runtime,
	types::{SymPtr, TypeNPtr, TypePtr},
};

pub(crate) fn primitive_type(
	ctx: &SemaContext,
	arg_list: ast::ArgList,
	_: &Symbol,
	_: &sym::Function,
) -> CEval {
	#[must_use]
	fn lazy_init(
		ctx: &SemaContext,
		ptr: &TypeNPtr,
		arg_list: ast::ArgList,
		datum: tsys::Primitive,
	) -> TypePtr {
		let Some(sym_const) = arg_list.syntax().ancestors().find_map(ast::SymConst::cast) else {
			panic!("primitive type must be assigned to a symbolic constant")
		};

		let location = Location {
			file_ix: ctx.file_ix,
			span: sym_const.syntax().text_range(),
		};

		let sym_ptr = ctx.symbols.get(&SymbolId::new(location)).unwrap();

		let tdef = TypeDef {
			symbol: Some(SymPtr::from(sym_ptr.value())),
			datum: tsys::TypeDatum::Primitive(datum),
		};

		let p = ctx.intern_type(tdef);
		ptr.store(p);
		p
	}

	let mut args = arg_list.iter();

	let Some(arg0) = args.next() else {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg_list.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message_static("`primitiveType` requires exactly one argument"),
		);

		return CEval::Err;
	};

	let ast::Expr::Literal(lit) = arg0.expr().unwrap() else {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg0.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgType),
			)
			.with_message_static("`primitiveType` argument 1 must be a name literal"),
		);

		return CEval::Err;
	};

	let token = lit.token();

	let Some(iname) = token.name() else {
		ctx.raise(
			Issue::new(
				ctx.path,
				token.text_range(),
				issue::Level::Error(issue::Error::ArgType),
			)
			.with_message(format!(
				"`primitiveType` argument 1 expected a name literal, found: {token}"
			)),
		);

		return CEval::Err;
	};

	let type_ptr = match iname {
		"void" => ctx
			.sym_cache
			.void_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.void_t, arg_list, tsys::Primitive::Void)
			}),
		"bool" => ctx
			.sym_cache
			.bool_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.bool_t, arg_list, tsys::Primitive::Bool)
			}),
		"i8" => ctx
			.sym_cache
			.i8_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.i8_t, arg_list, tsys::Primitive::I8)),
		"u8" => ctx
			.sym_cache
			.u8_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.u8_t, arg_list, tsys::Primitive::U8)),
		"i16" => ctx
			.sym_cache
			.i16_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i16_t, arg_list, tsys::Primitive::I16)
			}),
		"u16" => ctx
			.sym_cache
			.u16_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u16_t, arg_list, tsys::Primitive::U16)
			}),
		"i32" => ctx
			.sym_cache
			.i32_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i32_t, arg_list, tsys::Primitive::I32)
			}),
		"u32" => ctx
			.sym_cache
			.u32_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u32_t, arg_list, tsys::Primitive::U32)
			}),
		"i64" => ctx
			.sym_cache
			.i64_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i64_t, arg_list, tsys::Primitive::I64)
			}),
		"u64" => ctx
			.sym_cache
			.u64_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u64_t, arg_list, tsys::Primitive::U64)
			}),
		"i128" => ctx
			.sym_cache
			.i128_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i128_t, arg_list, tsys::Primitive::I128)
			}),
		"u128" => ctx
			.sym_cache
			.u128_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u128_t, arg_list, tsys::Primitive::U128)
			}),
		"f32" => ctx
			.sym_cache
			.f32_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.f32_t, arg_list, tsys::Primitive::F32)
			}),
		"f64" => ctx
			.sym_cache
			.f64_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.f64_t, arg_list, tsys::Primitive::F64)
			}),
		"iname_t" => ctx
			.sym_cache
			.iname_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(
					ctx,
					&ctx.sym_cache.iname_t,
					arg_list,
					tsys::Primitive::IName,
				)
			}),
		"never_t" => ctx
			.sym_cache
			.never_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| {
				lazy_init(
					ctx,
					&ctx.sym_cache.never_t,
					arg_list,
					tsys::Primitive::Never,
				)
			}),
		other => {
			ctx.raise(
				Issue::new(
					ctx.path,
					token.text_range(),
					issue::Level::Error(issue::Error::Builtin),
				)
				.with_message(format!("invalid primitive type name: `{other}`")),
			);

			return CEval::Err;
		}
	};

	CEval::Type(type_ptr)
}

pub(crate) fn type_of(_: &SemaContext, _: ast::ArgList, _: &Symbol, _: &sym::Function) -> CEval {
	todo!()
}

pub(crate) fn rtti_of(_: &SemaContext, _: ast::ArgList, _: &Symbol, _: &sym::Function) -> CEval {
	unimplemented!()
}

/// Returns the total memory used by the garbage collector.
pub(crate) unsafe extern "C" fn gc_usage(_: *mut Runtime) -> usize {
	// TODO: just a dummy function for proof-of-concept purposes at the moment.
	unimplemented!()
}

pub(crate) const GC_USAGE: RunTimeNativeFunc = RunTimeNativeFunc::Static {
	ptr: gc_usage as *const u8,
	params: &[AbiParam {
		value_type: interop::PTR_T,
		purpose: ArgumentPurpose::Normal,
		extension: ArgumentExtension::Uext,
	}],
	returns: &[AbiParam {
		value_type: <usize as interop::Native>::REPR,
		purpose: ArgumentPurpose::Normal,
		extension: ArgumentExtension::Uext,
	}],
};

/// Constants fed to [`cranelift::codegen::ir::UserExternalName::index`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum Index {
	PrimitiveType,
	TypeOf,
	RttiOf,
	GcUsage,
	__Last,
}

impl From<u32> for Index {
	fn from(value: u32) -> Self {
		assert!(value < Self::__Last as u32);
		unsafe { std::mem::transmute(value) }
	}
}
