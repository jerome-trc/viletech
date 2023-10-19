//! Compiler intrinsic functions.
//!
//! Some are runtime-only, some are const-eval only, some support both.

use doomfront::rowan::ast::AstNode;

use crate::{
	ast,
	data::{self, Datum, DatumPtr, Location, SymPtr, Symbol, SymbolId},
	issue::{self, Issue},
	runtime, CEval, SemaContext,
};

pub(crate) fn primitive_type(ctx: &SemaContext, arg_list: ast::ArgList) -> CEval {
	let Some(caller) = arg_list.syntax().ancestors().find_map(ast::SymConst::cast) else {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg_list.syntax().parent().unwrap().text_range(),
				issue::Level::Error(issue::Error::BuiltinMisuse),
			)
			.with_message_static(
				"`primitiveType` can only be used as an initializer for symbolic constants",
			),
		);

		return CEval::Err;
	};

	let location = Location {
		file_ix: ctx.file_ix,
		span: caller.syntax().text_range(),
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

	let datum = match iname {
		"void" => data::Primitive::Void,
		"bool" => data::Primitive::Bool,
		"i8" => data::Primitive::I8,
		"u8" => data::Primitive::U8,
		"i16" => data::Primitive::I16,
		"u16" => data::Primitive::U16,
		"i32" => data::Primitive::I32,
		"u32" => data::Primitive::U32,
		"i64" => data::Primitive::I64,
		"u64" => data::Primitive::U64,
		"i128" => data::Primitive::I128,
		"u128" => data::Primitive::U128,
		"f32" => data::Primitive::F32,
		"f64" => data::Primitive::F64,
		other => {
			ctx.raise(
				Issue::new(
					ctx.path,
					token.text_range(),
					issue::Level::Error(issue::Error::BuiltinMisuse),
				)
				.with_message(format!("invalid primitive type name: `{other}`")),
			);

			return CEval::Err;
		}
	};

	let sym_ptr = SymPtr::alloc(
		ctx.arena,
		Symbol {
			location,
			datum: DatumPtr::alloc(ctx.arena, Datum::Primitive(datum)),
		},
	);

	let overriden = ctx.symbols.insert(SymbolId::new(location), sym_ptr.clone());

	debug_assert!(overriden.is_none());

	CEval::Type(sym_ptr)
}

/// Returns the total memory used by the garbage collector.
pub(crate) extern "C" fn _gc_usage(_: *mut runtime::Context) -> usize {
	// TODO: just a dummy function for proof-of-concept purposes at the moment.
	123_456_789
}

// All constants below are used in `UserExternalName::index`.
pub(crate) const UEXTIX_PRIMITIVETYPE: u32 = 0;
pub(crate) const UEXTIX_GCUSAGE: u32 = 100;
