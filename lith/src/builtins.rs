//! Compiler intrinsic functions.
//!
//! Some are runtime-only, some are const-eval only, some support both.

use doomfront::rowan::ast::AstNode;

use crate::{
	ast,
	issue::{self, Issue},
	runtime, tsys,
	types::{TypeNPtr, TypePtr},
	CEval, SemaContext,
};

pub(crate) fn primitive_type(ctx: &SemaContext, arg_list: ast::ArgList) -> CEval {
	#[must_use]
	fn lazy_init(ctx: &SemaContext, ptr: &TypeNPtr, datum: tsys::Primitive) -> TypePtr {
		let p = ctx.intern_type(tsys::TypeDef::Primitive(datum));
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
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.void_t, tsys::Primitive::Void)),
		"bool" => ctx
			.sym_cache
			.bool_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.bool_t, tsys::Primitive::Bool)),
		"i8" => ctx
			.sym_cache
			.i8_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.i8_t, tsys::Primitive::I8)),
		"u8" => ctx
			.sym_cache
			.u8_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.u8_t, tsys::Primitive::U8)),
		"i16" => ctx
			.sym_cache
			.i16_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.i16_t, tsys::Primitive::I16)),
		"u16" => ctx
			.sym_cache
			.u16_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.u16_t, tsys::Primitive::U16)),
		"i32" => ctx
			.sym_cache
			.i32_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.i32_t, tsys::Primitive::I32)),
		"u32" => ctx
			.sym_cache
			.u32_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.u32_t, tsys::Primitive::U32)),
		"i64" => ctx
			.sym_cache
			.i64_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.i64_t, tsys::Primitive::I64)),
		"u64" => ctx
			.sym_cache
			.u64_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.u64_t, tsys::Primitive::U64)),
		"i128" => ctx
			.sym_cache
			.i128_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.i128_t, tsys::Primitive::I128)),
		"u128" => ctx
			.sym_cache
			.u128_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.u128_t, tsys::Primitive::U128)),
		"f32" => ctx
			.sym_cache
			.f32_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.f32_t, tsys::Primitive::F32)),
		"f64" => ctx
			.sym_cache
			.f64_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.f64_t, tsys::Primitive::F64)),
		"iname_t" => ctx
			.sym_cache
			.iname_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.iname_t, tsys::Primitive::IName)),
		"never_t" => ctx
			.sym_cache
			.never_t
			.as_ptr()
			.map(TypePtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.never_t, tsys::Primitive::Never)),
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

pub(crate) fn type_of(_: &SemaContext, _: ast::ArgList) -> CEval {
	todo!()
}

pub(crate) fn rtti_of(_: &SemaContext, _: ast::ArgList) -> CEval {
	unimplemented!()
}

/// Returns the total memory used by the garbage collector.
pub(crate) unsafe extern "C" fn gc_usage(_: *mut runtime::InContext) -> usize {
	// TODO: just a dummy function for proof-of-concept purposes at the moment.
	unimplemented!()
}

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
