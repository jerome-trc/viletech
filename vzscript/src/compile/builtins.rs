use std::any::Any;

use crossbeam::atomic::AtomicCell;
use doomfront::rowan::{ast::AstNode, TextRange, TextSize};
use triomphe::Arc;
use util::rstring::RString;

use crate::{
	ast,
	issue::{self, Issue},
	rti,
	sema::{self, CEval, CEvalVec},
	tsys::TypeDef,
	zname::ZName,
	ArcSwap,
};

use super::{
	intern::NameIx,
	symbol::{DefKind, DefStatus, Definition, Location, Symbol},
	Compiler,
};

#[must_use]
fn define_primitive(compiler: &Compiler, qname: &'static str, typedef: TypeDef) -> CEval {
	let store = rti::Store::new(ZName(RString::new(qname)), typedef);
	let record = rti::Record::new_type(store);
	CEval::TypeDef { record }
}

fn validate_int_t_args(
	compiler: &Compiler,
	fn_name: &'static str,
	path: &str,
	arglist: &ast::ArgList,
) -> Result<u64, Issue> {
	let mut args = arglist.iter();

	let Some(arg0) = args.next() else {
		return Err(Issue::new(
			path,
			arglist.syntax().text_range(),
			format!("`{fn_name}` requires at least one argument"),
			issue::Level::Error(issue::Error::ArgCount),
		));
	};

	if let Some(arg1) = args.next() {
		return Err(Issue::new(
			path,
			arg1.syntax().text_range(),
			format!("`{fn_name}` takes only one argument"),
			issue::Level::Error(issue::Error::ArgCount),
		));
	};

	let ast::Expr::Literal(lit) = arg0.expr().unwrap() else {
		return Err(Issue::new(
			path,
			arg0.syntax().text_range(),
			format!("`{fn_name}` currently only supports literal arguments"),
			issue::Level::Error(issue::Error::ArgType),
		));
	};

	let token = lit.token();

	let Some(result) = token.int() else {
		return Err(Issue::new(
			path,
			token.text_range(),
			format!("expected integer literal, found: {token}"),
			issue::Level::Error(issue::Error::ArgType),
		));
	};

	let num_bits = match result {
		Ok((num_bits, suffix)) => num_bits,
		Err(err) => {
			return Err(Issue::new(
				path,
				token.text_range(),
				format!("failed to parse integer literal: {err}"),
				issue::Level::Error(issue::Error::ParseInt),
			));
		}
	};

	Ok(num_bits)
}

#[must_use]
fn int_t_bit_width_error(path: &str, arglist: &ast::ArgList) -> Issue {
	Issue::new(
		path,
		arglist
			.iter()
			.next()
			.unwrap()
			.expr()
			.unwrap()
			.syntax()
			.text_range(),
		"VZScript only supports integers of 8, 16, 32, 64, and 128 bits".to_string(),
		issue::Level::Error(issue::Error::Builtin),
	)
}

pub(crate) fn int_t(compiler: &Compiler, path: &str, arglist: ast::ArgList) -> Result<CEval, ()> {
	let num_bits = match validate_int_t_args(compiler, "int_t", path, &arglist) {
		Ok(num_bits) => num_bits,
		Err(issue) => {
			compiler.raise(issue);
			return Err(());
		}
	};

	let ce = match num_bits {
		8 => define_primitive(compiler, "vzs.int8", TypeDef::PRIMITIVE_INT8.clone()),
		16 => define_primitive(compiler, "vzs.int16", TypeDef::PRIMITIVE_INT16.clone()),
		32 => define_primitive(compiler, "vzs.int32", TypeDef::PRIMITIVE_INT32.clone()),
		64 => define_primitive(compiler, "vzs.int64", TypeDef::PRIMITIVE_INT64.clone()),
		128 => define_primitive(compiler, "vzs.int128", TypeDef::PRIMITIVE_INT128.clone()),
		_ => {
			compiler.raise(int_t_bit_width_error(path, &arglist));
			return Err(());
		}
	};

	Ok(ce)
}

pub(crate) fn uint_t(compiler: &Compiler, path: &str, arglist: ast::ArgList) -> Result<CEval, ()> {
	let num_bits = match validate_int_t_args(compiler, "uint_t", path, &arglist) {
		Ok(num_bits) => num_bits,
		Err(issue) => {
			compiler.raise(issue);
			return Err(());
		}
	};

	let ce = match num_bits {
		8 => define_primitive(compiler, "vzs.uint8", TypeDef::PRIMITIVE_UINT8.clone()),
		16 => define_primitive(compiler, "vzs.uint16", TypeDef::PRIMITIVE_UINT16.clone()),
		32 => define_primitive(compiler, "vzs.uint32", TypeDef::PRIMITIVE_UINT32.clone()),
		64 => define_primitive(compiler, "vzs.uint64", TypeDef::PRIMITIVE_UINT64.clone()),
		128 => define_primitive(compiler, "vzs.uint128", TypeDef::PRIMITIVE_UINT128.clone()),
		_ => {
			compiler.raise(int_t_bit_width_error(path, &arglist));
			return Err(());
		}
	};

	Ok(ce)
}

pub(crate) fn type_of(compiler: &Compiler, path: &str, arglist: ast::ArgList) -> Result<CEval, ()> {
	Err(())
}

pub(crate) fn rtti_of(compiler: &Compiler, path: &str, arglist: ast::ArgList) -> Result<CEval, ()> {
	Err(())
}
