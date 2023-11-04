//! Lowering routines from Lith ASTs to Cranelift Intermediate Format (CLIF).

use doomfront::rowan::ast::AstNode;
use smallvec::smallvec;

use crate::{
	ast,
	compile::LutSym,
	issue::{self, Issue},
	types::{Scope, SymOPtr, SymPtr, TypePtr},
};

use super::{
	ceval,
	func::Translator,
	sema::{CEval, SemaContext},
	sym::{LocalVar, Location, SymDatum, Symbol, SymbolId},
};

fn process_type_expr(ctx: &SemaContext, ast: ast::Expr) -> Result<TypePtr, ()> {
	let ast::Expr::Ident(e_id) = ast else {
		ctx.raise(
			Issue::new(
				ctx.path,
				ast.syntax().text_range(),
				issue::Level::Error(issue::Error::Unimplemented),
			)
			.with_message_static("only identifiers are currently supported in type expressions"),
		);

		return Err(());
	};

	todo!()
}

pub(super) fn statement(tlat: &mut Translator, scope: &mut Scope, ast: ast::Statement) {
	match ast {
		ast::Statement::Bind(s_bind) => {
			lower_stmt_bind(tlat, scope, s_bind);
		}
		ast::Statement::Break(_)
		| ast::Statement::Expr(_)
		| ast::Statement::Continue(_)
		| ast::Statement::Return(_) => todo!(),
	}
}

fn lower_stmt_bind(tlat: &mut Translator, scope: &mut Scope, ast: ast::StmtBind) {
	let pat = ast.pattern().unwrap();

	let ident = match &pat {
		ast::Pattern::Ident(id) => id.token(),
		ast::Pattern::Grouped(_)
		| ast::Pattern::Literal(_)
		| ast::Pattern::Slice(_)
		| ast::Pattern::Wildcard(_) => {
			tlat.ctx.raise(
				Issue::new(
					tlat.ctx.path,
					pat.syntax().text_range(),
					issue::Level::Error(issue::Error::Unimplemented),
				)
				.with_message_static("only identifier pattern bindings are currently supported"),
			);

			tlat.failed = true;
			return;
		}
	};

	let Some(ast_tspec) = ast.type_spec() else {
		tlat.ctx.raise(
			Issue::new(
				tlat.ctx.path,
				ast.syntax().text_range(),
				issue::Level::Error(issue::Error::Unimplemented),
			)
			.with_message_static("binding statement is missing a type specifier")
			.with_note_static("type inference for binding statements is not yet implemented"),
		);

		tlat.failed = true;
		return;
	};

	let Some(texpr) = ast_tspec.into_expr() else {
		tlat.ctx.raise(todo!());
		return;
	};

	let tspec = match ceval::expr(tlat.ctx, 0, todo!(), texpr) {
		CEval::Type(t_ptr) => t_ptr,
		CEval::Container(_) => {
			tlat.ctx.raise(todo!());
			return;
		}
		CEval::Function(_) => {
			tlat.ctx.raise(todo!());
			return;
		}
		CEval::Value(_) => {
			tlat.ctx.raise(todo!());
			return;
		}
		CEval::Err => return,
	};

	let mut local = LocalVar {
		abi_vars: smallvec![],
		mutable: match ast.keyword() {
			ast::BindKeyword::Let(_) => false,
			ast::BindKeyword::Var(_) => true,
		},
		tspec,
	};

	let location = Location {
		file_ix: tlat.ctx.file_ix,
		span: ast.syntax().text_range(),
	};

	let sym = Symbol {
		location,
		datum: SymDatum::Local(local),
	};

	scope.insert(
		tlat.ctx.names.intern(&ident),
		LutSym::Owned {
			ptr: SymOPtr::alloc(tlat.ctx.arena, sym),
			imported: false,
		},
	);
}
