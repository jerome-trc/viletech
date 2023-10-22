//! Annotation processing helpers used by [`super::decl`].

use cranelift::codegen::ir::UserExternalName;
use doomfront::rowan::ast::AstNode;

use crate::{
	ast::{self, LitToken},
	issue::{self, Issue},
	sym::{self, Confinement, FunctionFlags, FunctionKind, Inlining},
};

use super::FrontendContext;

pub(super) fn unknown_annotation_error(
	ctx: &FrontendContext,
	ast: ast::Annotation,
	names: (&str, Option<&str>),
) {
	let msg = match names.1 {
		Some(n1) => format!("unknown annotation: `{n0}.{n1}`", n0 = names.0),
		None => format!("unknown annotation: `{}`", names.0),
	};

	ctx.raise(
		Issue::new(
			ctx.path,
			ast.syntax().text_range(),
			issue::Level::Error(issue::Error::UnknownAnnotation),
		)
		.with_message(msg),
	);
}

// `#[builtin()]` //////////////////////////////////////////////////////////////

pub(super) fn builtin_fndecl(
	ctx: &FrontendContext,
	anno: ast::Annotation,
	datum: &mut sym::Function,
) {
	if !check_native_lib(ctx, "builtin", &anno) {
		return;
	}

	let Some(arg_list) = check_arg_list(ctx, "builtin", &anno) else {
		return;
	};

	if !check_non_acceding(ctx, "builtin", &anno, &arg_list) {
		return;
	}

	let mut args = arg_list.iter();

	let Some(arg0) = check_arg0_exactly(ctx, "builtin", &arg_list, &mut args) else {
		return;
	};

	if !check_arg_anon(ctx, "builtin", &arg0) {
		return;
	}

	let expr = arg0.expr().unwrap();

	let Some(lit_name) = check_expr_lit_name(ctx, "builtin", expr) else {
		return;
	};

	let string = lit_name.name().unwrap();

	match string {
		"primitiveType" => {
			datum.kind = FunctionKind::Builtin {
				uext_name: UserExternalName {
					namespace: crate::CLNS_BUILTIN,
					index: crate::builtins::UEXTIX_PRIMITIVETYPE,
				},
				rt: None,
				ceval: Some(crate::builtins::primitive_type),
			};
		}
		"typeOf" => {
			// TODO
		}
		"rttiOf" => {
			// TODO
		}
		other => panic!("unknown baselib builtin name: `{other}`"),
	}
}

pub(super) fn builtin_non_fndecl(ctx: &FrontendContext, anno: ast::Annotation) {
	ctx.raise(
		Issue::new(
			ctx.path,
			anno.syntax().text_range(),
			issue::Level::Error(issue::Error::AnnotationUsage),
		)
		.with_message_static("`builtin` annotation can only be used on function declarations"),
	);
}

// `#[cold]` ///////////////////////////////////////////////////////////////////

pub(super) fn cold_fndecl(
	ctx: &FrontendContext,
	anno: ast::Annotation,
	in_out: &mut FunctionFlags,
) {
	if !check_no_arg_list(ctx, "cold", &anno) {
		return;
	}

	in_out.insert(FunctionFlags::COLD);
}

pub(super) fn cold_invalid(ctx: &FrontendContext, anno: ast::Annotation) {
	ctx.raise(
		Issue::new(
			ctx.path,
			anno.syntax().text_range(),
			issue::Level::Error(issue::Error::AnnotationUsage),
		)
		.with_message_static("`cold` annotation can only be used on function declarations"),
	); // TODO: allow branches to support `#[cold]` and mention it here.
}

// `#[confine()]` //////////////////////////////////////////////////////////////

pub(super) fn confine(ctx: &FrontendContext, anno: ast::Annotation, in_out: &mut Confinement) {
	let Some(arg_list) = check_arg_list(ctx, "confine", &anno) else {
		return;
	};

	if !check_non_acceding(ctx, "confine", &anno, &arg_list) {
		return;
	}

	let mut args = arg_list.iter();

	let Some(arg0) = check_arg0_exactly(ctx, "confine", &arg_list, &mut args) else {
		return;
	};

	if !check_arg_anon(ctx, "confine", &arg0) {
		return;
	}

	let Some(e_ident) = check_expr_ident(ctx, "confine", arg0.expr().unwrap()) else {
		return;
	};

	let confine = match e_ident.token().text() {
		"none" => Confinement::None,
		"ui" => Confinement::Ui,
		"sim" => Confinement::Sim,
		_ => {
			const MSG: &str = concat!(
				"`confine` annotation argument must be one of the following:",
				"\r\n- `none`",
				"\r\n- `ui`",
				"\r\n- `sim`"
			);

			ctx.raise(
				Issue::new(
					ctx.path,
					e_ident.syntax().text_range(),
					issue::Level::Error(issue::Error::AnnotationArg),
				)
				.with_message_static(MSG),
			);

			return;
		}
	};

	if !check_no_arg1(ctx, "confine", &mut args) {
		return;
	}

	*in_out = confine;
}

// `#[crucial]` ////////////////////////////////////////////////////////////////

pub(super) fn crucial_nonfndecl(ctx: &FrontendContext, anno: ast::Annotation) {
	ctx.raise(
		Issue::new(
			ctx.path,
			anno.syntax().text_range(),
			issue::Level::Error(issue::Error::AnnotationUsage),
		)
		.with_message_static("`crucial` annotation can only be used on function declarations"),
	);
}

// `#[inline()]` ///////////////////////////////////////////////////////////////

pub(super) fn inline_fndecl(ctx: &FrontendContext, anno: ast::Annotation, in_out: &mut Inlining) {
	let Some(arg_list) = anno.arg_list() else {
		*in_out = Inlining::More;
		return;
	};

	if !check_non_acceding(ctx, "inline", &anno, &arg_list) {
		return;
	}

	let mut args = arg_list.iter();

	let Some(arg0) = args.next() else {
		return;
	};

	if !check_arg_anon(ctx, "inline", &arg0) {
		return;
	}

	let Some(e_ident) = check_expr_ident(ctx, "inline", arg0.expr().unwrap()) else {
		return;
	};

	let policy = match e_ident.token().text() {
		"never" => Inlining::Never,
		"extra" => Inlining::Extra,
		_ => {
			const MSG: &str = concat!(
				"`inline` annotation argument must be one of the following:",
				"\r\n- `never`",
				"\r\n- `extra`",
			);

			ctx.raise(
				Issue::new(
					ctx.path,
					e_ident.syntax().text_range(),
					issue::Level::Error(issue::Error::AnnotationArg),
				)
				.with_message_static(MSG),
			);

			return;
		}
	};

	if !check_no_arg1(ctx, "inline", &mut args) {
		return;
	}

	*in_out = policy;
}

pub(super) fn inline_non_fndecl(ctx: &FrontendContext, anno: ast::Annotation) {
	ctx.raise(
		Issue::new(
			ctx.path,
			anno.syntax().text_range(),
			issue::Level::Error(issue::Error::AnnotationUsage),
		)
		.with_message_static("`inline` annotation can only be used on function declarations"),
	);
}

// `#[native()]` ///////////////////////////////////////////////////////////////

pub(super) fn native_fndecl(
	ctx: &FrontendContext,
	anno: ast::Annotation,
	datum: &mut sym::Function,
) {
	if !check_native_lib(ctx, "native", &anno) {
		return;
	}

	let Some(arg_list) = check_arg_list(ctx, "native", &anno) else {
		return;
	};

	if !check_non_acceding(ctx, "native", &anno, &arg_list) {
		return;
	}

	let mut args = arg_list.iter();

	let Some(arg0) = check_arg0_exactly(ctx, "native", &arg_list, &mut args) else {
		return;
	};

	if !check_arg_anon(ctx, "native", &arg0) {
		return;
	}

	let expr = arg0.expr().unwrap();

	let Some(lit_name) = check_expr_lit_name(ctx, "native", expr) else {
		return;
	};

	let string = lit_name.name().unwrap();

	let Some((ix, _qname, nfn)) = ctx.compiler.native.functions.get_full(string) else {
		ctx.raise(
			Issue::new(
				ctx.path,
				lit_name.text_range(),
				issue::Level::Error(issue::Error::MissingNative),
			)
			.with_message(format!(
				"native function `{string}` has not been registered"
			)),
		);

		return;
	};

	datum.kind = FunctionKind::Native {
		uext_name: UserExternalName {
			namespace: crate::CLNS_NATIVE,
			index: ix as u32,
		},
		rt: nfn.rt.as_ref().map(|rtn| rtn.ptr),
		ceval: nfn.ceval,
	};
}

// Details /////////////////////////////////////////////////////////////////////

#[must_use]
fn check_native_lib(ctx: &FrontendContext, name: &'static str, anno: &ast::Annotation) -> bool {
	if !ctx.lib.native {
		ctx.raise(
			Issue::new(
				ctx.path,
				anno.syntax().text_range(),
				issue::Level::Error(issue::Error::NonNative),
			)
			.with_message(format!(
				"`{name}` annotation can only be used by native libraries"
			)),
		);
	}

	ctx.lib.native
}

#[must_use]
fn check_arg_list(
	ctx: &FrontendContext,
	name: &'static str,
	anno: &ast::Annotation,
) -> Option<ast::ArgList> {
	anno.arg_list().or_else(|| {
		ctx.raise(
			Issue::new(
				ctx.path,
				anno.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message(format!("`{name}` annotation requires an argument list")),
		);

		None
	})
}

#[must_use]
fn check_non_acceding(
	ctx: &FrontendContext,
	name: &'static str,
	anno: &ast::Annotation,
	arg_list: &ast::ArgList,
) -> bool {
	let ret = arg_list.acceding();

	if ret {
		ctx.raise(
			Issue::new(
				ctx.path,
				anno.syntax().text_range(),
				issue::Level::Error(issue::Error::IllegalAccede),
			)
			.with_message(format!(
				"`{name}` annotation does not support deference to parameter default arguments"
			)),
		);
	}

	ret
}

#[must_use]
fn check_no_arg_list(ctx: &FrontendContext, name: &'static str, anno: &ast::Annotation) -> bool {
	let has_arg_list = anno.arg_list().is_some();

	if has_arg_list {
		ctx.raise(
			Issue::new(
				ctx.path,
				anno.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message(format!("`{name}` annotation takes no arguments")),
		);
	}

	!has_arg_list
}

#[must_use]
fn check_arg0_exactly(
	ctx: &FrontendContext,
	name: &'static str,
	arg_list: &ast::ArgList,
	args: &mut impl Iterator<Item = ast::Argument>,
) -> Option<ast::Argument> {
	args.next().or_else(|| {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg_list.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message(format!("`{name}` annotation requires exactly one argument")),
		);

		None
	})
}

#[must_use]
fn check_arg_anon(ctx: &FrontendContext, name: &'static str, arg: &ast::Argument) -> bool {
	if let Some(arg_name) = arg.name() {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg_name.inner().text_range(),
				issue::Level::Error(issue::Error::IllegalArgName),
			)
			.with_message(format!(
				"`{name}` annotation does not accept named arguments"
			)),
		);

		return false;
	}

	true
}

#[must_use]
fn check_no_arg1(
	ctx: &FrontendContext,
	name: &'static str,
	args: &mut impl Iterator<Item = ast::Argument>,
) -> bool {
	if let Some(arg1) = args.next() {
		ctx.raise(
			Issue::new(
				ctx.path,
				arg1.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgCount),
			)
			.with_message(format!("`{name}` annotation can only accept one argument")),
		);

		return false;
	}

	true
}

#[must_use]
fn check_expr_ident(
	ctx: &FrontendContext,
	name: &'static str,
	expr: ast::Expr,
) -> Option<ast::ExprIdent> {
	let ast::Expr::Ident(e_ident) = expr else {
		ctx.raise(
			Issue::new(
				ctx.path,
				expr.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgType),
			)
			.with_message(format!(
				"`{name}` annotation argument must be an identifier"
			)),
		);

		return None;
	};

	Some(e_ident)
}

#[must_use]
fn check_expr_lit_name(
	ctx: &FrontendContext,
	name: &'static str,
	expr: ast::Expr,
) -> Option<LitToken> {
	let ast::Expr::Literal(e_lit) = expr.clone() else {
		ctx.raise(
			Issue::new(
				ctx.path,
				expr.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgType),
			)
			.with_message(format!(
				"`{name}` annotation argument must be a name literal"
			)),
		);

		return None;
	};

	let token = e_lit.token();

	token
		.clone()
		.name()
		.or_else(|| {
			ctx.raise(
				Issue::new(
					ctx.path,
					expr.syntax().text_range(),
					issue::Level::Error(issue::Error::ArgType),
				)
				.with_message(format!(
					"`{name}` annotation argument must be a name literal"
				)),
			);

			None
		})
		.map(|_| token)
}

#[must_use]
fn _check_expr_lit_string(
	ctx: &FrontendContext,
	name: &'static str,
	expr: ast::Expr,
) -> Option<LitToken> {
	let ast::Expr::Literal(e_lit) = expr.clone() else {
		ctx.raise(
			Issue::new(
				ctx.path,
				expr.syntax().text_range(),
				issue::Level::Error(issue::Error::ArgType),
			)
			.with_message(format!(
				"`{name}` annotation argument must be a string literal"
			)),
		);

		return None;
	};

	let token = e_lit.token();

	token
		.clone()
		.string()
		.or_else(|| {
			ctx.raise(
				Issue::new(
					ctx.path,
					expr.syntax().text_range(),
					issue::Level::Error(issue::Error::ArgType),
				)
				.with_message(format!(
					"`{name}` annotation argument must be a string literal"
				)),
			);

			None
		})
		.map(|_| token)
}
