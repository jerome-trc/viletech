//! Compiler intrinsic functions.
//!
//! Some are runtime-only, some are const-eval only, some support both.

use doomfront::rowan::ast::AstNode;

use crate::{
	ast,
	issue::{self, Issue},
	runtime,
	tsys::{self},
	types::{TypeNPtr, TypePtr},
	CEval, SemaContext,
};

pub(crate) fn primitive_type(ctx: &SemaContext, arg_list: ast::ArgList) -> CEval {
	#[must_use]
	fn lazy_init(ctx: &SemaContext, ptr: &TypeNPtr, datum: tsys::Primitive) -> TypePtr {
		let p = TypePtr::alloc(ctx.arena, tsys::TypeDef::Primitive(datum));
		let _ = ctx.types.push(p);
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
			.with_message_static("`primitiveType` argument 1 must be an integer literal"),
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
				"`primitiveType` argument 1 expected an integer literal, found: {token}"
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

/// Returns the total memory used by the garbage collector.
pub(crate) extern "C" fn _gc_usage(_: *mut runtime::Context) -> usize {
	// TODO: just a dummy function for proof-of-concept purposes at the moment.
	123_456_789
}

// All constants below are used in `UserExternalName::index`.
pub(crate) const UEXTIX_PRIMITIVETYPE: u32 = 0;
pub(crate) const UEXTIX_GCUSAGE: u32 = 100;
