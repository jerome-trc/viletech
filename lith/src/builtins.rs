//! Compiler intrinsic functions.
//!
//! Some are runtime-only, some are const-eval only, some support both.

use doomfront::rowan::ast::AstNode;

use crate::{
	ast,
	data::{self, Datum, Location, Symbol},
	issue::{self, Issue},
	runtime,
	types::{SymNPtr, SymPtr},
	CEval, SemaContext,
};

pub(crate) fn primitive_type(ctx: &SemaContext, arg_list: ast::ArgList) -> CEval {
	#[must_use]
	fn lazy_init(
		ctx: &SemaContext,
		ptr: &SymNPtr,
		datum: data::Primitive,
		location: Location,
	) -> SymPtr {
		let p = SymPtr::alloc(
			ctx.arena,
			Symbol {
				location,
				datum: Datum::Primitive(datum),
			},
		);

		ptr.store(p);

		p
	}

	let Some(sym_const) = arg_list.syntax().ancestors().find_map(ast::SymConst::cast) else {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg_list.syntax().parent().unwrap().text_range(),
				issue::Level::Error(issue::Error::Builtin),
			)
			.with_message_static(
				"`primitiveType` can only be used as an initializer for symbolic constants",
			),
		);

		return CEval::Err;
	};

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

	let location = Location {
		file_ix: ctx.file_ix,
		span: sym_const.syntax().text_range(),
	};

	let type_ptr = match iname {
		"void" => ctx
			.sym_cache
			.void_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.void_t, data::Primitive::Void, location)
			}),
		"bool" => ctx
			.sym_cache
			.bool_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.bool_t, data::Primitive::Bool, location)
			}),
		"i8" => ctx
			.sym_cache
			.i8_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.i8_t, data::Primitive::I8, location)),
		"u8" => ctx
			.sym_cache
			.u8_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| lazy_init(ctx, &ctx.sym_cache.u8_t, data::Primitive::U8, location)),
		"i16" => ctx
			.sym_cache
			.i16_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i16_t, data::Primitive::I16, location)
			}),
		"u16" => ctx
			.sym_cache
			.u16_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u16_t, data::Primitive::U16, location)
			}),
		"i32" => ctx
			.sym_cache
			.i32_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i32_t, data::Primitive::I32, location)
			}),
		"u32" => ctx
			.sym_cache
			.u32_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u32_t, data::Primitive::U32, location)
			}),
		"i64" => ctx
			.sym_cache
			.i64_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i64_t, data::Primitive::I64, location)
			}),
		"u64" => ctx
			.sym_cache
			.u64_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u64_t, data::Primitive::U64, location)
			}),
		"i128" => ctx
			.sym_cache
			.i128_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.i128_t, data::Primitive::I128, location)
			}),
		"u128" => ctx
			.sym_cache
			.u128_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.u128_t, data::Primitive::U128, location)
			}),
		"f32" => ctx
			.sym_cache
			.f32_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.f32_t, data::Primitive::F32, location)
			}),
		"f64" => ctx
			.sym_cache
			.f64_t
			.as_ptr()
			.map(SymPtr::new)
			.unwrap_or_else(|| {
				lazy_init(ctx, &ctx.sym_cache.f64_t, data::Primitive::F64, location)
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

/// Returns the total memory used by the garbage collector.
pub(crate) extern "C" fn _gc_usage(_: *mut runtime::Context) -> usize {
	// TODO: just a dummy function for proof-of-concept purposes at the moment.
	123_456_789
}

// All constants below are used in `UserExternalName::index`.
pub(crate) const UEXTIX_PRIMITIVETYPE: u32 = 0;
pub(crate) const UEXTIX_GCUSAGE: u32 = 100;
