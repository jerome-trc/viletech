//! Semantic mid-section for ZScript.

use std::sync::Arc;

use crossbeam::utils::Backoff;
use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::{
		ast::LitToken,
		zscript::{ast, ParseTree},
	},
};

use crate::{
	compile::Scope,
	front::{Location, Symbol},
	issue::{self, FileSpan, Issue, IssueLevel},
	vir,
};

use super::{ConstEval, SemaContext};

pub(super) fn sema(ctx: &mut SemaContext, ptree: &ParseTree) {
	let ast = ptree
		.cursor()
		.children()
		.map(|node| ast::TopLevel::cast(node).unwrap());

	for top in ast {
		match top {
			ast::TopLevel::ClassDef(classdef) => {
				define_class(ctx, classdef);
			}
			ast::TopLevel::ConstDef(constdef) => {
				define_constant(ctx, constdef);
			}
			ast::TopLevel::EnumDef(_) => todo!(),
			ast::TopLevel::StructDef(_) => todo!(),
			ast::TopLevel::ClassExtend(_)
			| ast::TopLevel::Include(_)
			| ast::TopLevel::MixinClassDef(_)
			| ast::TopLevel::StructExtend(_)
			| ast::TopLevel::Version(_) => {}
		}
	}
}

fn define_class(ctx: &mut SemaContext, classdef: ast::ClassDef) {
	let name_tok = classdef.name().unwrap();
	let iname = ctx.intern_name(name_tok.text());
	let symptr = ctx.scopes.lookup(&iname).unwrap().clone();
	let symbol = symptr.load();

	if !symbol.is_undefined() {
		// Native-defined or a definition is in progress. Nothing to do here.
		return;
	}

	drop(symbol);

	for qual in classdef.qualifiers() {
		match qual {
			ast::ClassQual::Replaces(_) => todo!(),
			ast::ClassQual::Abstract(_) => todo!(),
			ast::ClassQual::Play(_) => todo!(),
			ast::ClassQual::Ui(_) => todo!(),
			ast::ClassQual::Native(token) => {
				let r_start = classdef.syntax().text_range().start();
				let r_end = name_tok.text_range().start();

				ctx.raise([Issue {
					id: FileSpan::new(ctx.ipath.as_str(), TextRange::new(r_start, r_end)),
					level: IssueLevel::Error(issue::Error::IllegalClassQual),
					message: "`native` ZScript symbols cannot be transpiled".to_string(),
					label: None,
				}]);

				return;
			}
			ast::ClassQual::Version(_) => {}
		}
	}

	symptr.rcu(|undef| {
		let Symbol::Undefined { location, kind, scope } = undef.as_ref() else {
			unreachable!()
		};

		let mut guard = scope.write();
		let scope: &mut Scope = &mut guard;
		let scope = std::mem::take(scope);

		for innard in classdef.innards() {
			match innard {
				ast::ClassInnard::Const(constdef) => {
					define_constant(ctx, constdef);
				}
				ast::ClassInnard::Enum(_) => todo!(),
				ast::ClassInnard::Struct(_) => todo!(),
				ast::ClassInnard::StaticConst(_) => todo!(),
				ast::ClassInnard::Function(fndecl) => {}
				ast::ClassInnard::Field(_) => todo!(),
				ast::ClassInnard::Default(_) => todo!(),
				ast::ClassInnard::States(_) => todo!(),
				ast::ClassInnard::Property(_) => todo!(),
				ast::ClassInnard::Flag(flagdef) => {
					let backing_tok = flagdef.backing_field().unwrap();
					let backing_iname = ctx.intern_name(backing_tok.text());
					let backing = scope.get(&backing_iname).unwrap();

					let bit_tok = flagdef.bit().unwrap();

					let bit = match bit_tok.int().unwrap() {
						Ok(b) => b,
						Err(err) => {
							ctx.raise([Issue {
								id: FileSpan::new(
									ctx.ipath.as_str(),
									bit_tok.syntax().text_range(),
								),
								level: IssueLevel::Error(issue::Error::ParseInt),
								message: format!("invalid integer: {err}"),
								label: None,
							}]);

							return undef.clone();
						}
					};

					if bit >= 32 {
						ctx.raise([Issue {
							id: FileSpan::new(ctx.ipath.as_str(), bit_tok.syntax().text_range()),
							level: IssueLevel::Error(issue::Error::FlagDefBitOverflow),
							message: format!("bit {bit} is out of range"),
							label: None,
						}]);

						return undef.clone();
					}
				}
				ast::ClassInnard::Mixin(_) => continue,
			}
		}

		Arc::new(todo!())
	});
}

fn define_constant(ctx: &mut SemaContext, constdef: ast::ConstDef) {
	let iname = ctx.intern_name(constdef.name().unwrap().text());
	let symptr = ctx.scopes.lookup(&iname).unwrap();
	let symbol = symptr.load();

	if !symbol.is_undefined() {
		// Native-defined or a definition is in progress. Nothing to do here.
		return;
	}

	drop(symbol);

	symptr.rcu(|undef| {
		let Symbol::Undefined { location, kind, scope } = undef.as_ref() else {
			unreachable!()
		};

		let expr = constdef.initializer().unwrap();

		let consteval = match expr {
			ast::Expr::Binary(e_bin) => match consteval_bin(ctx, e_bin) {
				Ok(eval) => eval,
				Err(()) => return undef.clone(),
			},
			ast::Expr::Call(e_call) => todo!(),
			ast::Expr::ClassCast(e_cast) => {
				ctx.raise([Issue {
					id: FileSpan::new(ctx.ipath.as_str(), e_cast.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::IllegalConstInit),
					message: "class cast expressions cannot be used to initialize constants"
						.to_string(),
					label: None,
				}]);

				return undef.clone();
			}
			ast::Expr::Group(e_grp) => todo!(),
			ast::Expr::Ident(e_ident) => todo!(),
			ast::Expr::Index(e_index) => todo!(),
			ast::Expr::Literal(e_lit) => match consteval_lit(ctx, e_lit) {
				Ok(eval) => eval,
				Err(()) => return undef.clone(),
			},
			ast::Expr::Member(e_mem) => todo!(),
			ast::Expr::Postfix(e_post) => todo!(),
			ast::Expr::Prefix(e_pre) => todo!(),
			ast::Expr::Super(e_super) => {
				ctx.raise([Issue {
					id: FileSpan::new(ctx.ipath.as_str(), e_super.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::IllegalConstInit),
					message: "`super` expressions cannot be used to initialize constants"
						.to_string(),
					label: None,
				}]);

				return undef.clone();
			}
			ast::Expr::Ternary(e_tern) => todo!(),
			ast::Expr::Vector(e_vector) => todo!(),
		};

		let Some(typedef) = consteval.typedef else {
			ctx.raise([Issue {
				id: FileSpan::new(ctx.ipath.as_str(), constdef.syntax().text_range()),
				level: IssueLevel::Error(issue::Error::UnknownExprType),
				message: "type of expression could not be inferred".to_string(),
				label: None,
			}]);

			return undef.clone();
		};

		Arc::new(Symbol::Value {
			location: location.clone(),
			typedef,
			mutable: false,
		})
	});
}

fn consteval_bin(ctx: &SemaContext, bin: ast::BinExpr) -> Result<ConstEval, ()> {
	let lhs = bin.left();
	let rhs = bin.right().unwrap();
	todo!()
}

fn consteval_lit(ctx: &SemaContext, literal: ast::Literal) -> Result<ConstEval, ()> {
	let token = literal.token();

	if token.null() {
		Ok(ConstEval {
			typedef: None,
			ir: vir::Node::Immediate(vir::Immediate::pointer(0)),
		})
	} else if let Some(boolean) = token.bool() {
		Ok(ConstEval {
			typedef: Some(ctx.builtins.bool_t.clone()),
			ir: vir::Node::Immediate(vir::Immediate::I8(boolean as i8)),
		})
	} else if let Some(result) = token.int() {
		match result {
			Ok(int) => Ok(ConstEval {
				typedef: Some(ctx.builtins.int32_t.clone()),
				ir: vir::Node::Immediate(vir::Immediate::I32(int)),
			}),
			Err(err) => {
				ctx.raise([Issue {
					id: FileSpan::new(ctx.ipath.as_str(), literal.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::ParseInt),
					message: format!("invalid integer: {err}"),
					label: None,
				}]);

				Err(())
			}
		}
	} else if let Some(result) = token.float() {
		match result {
			Ok(float) => Ok(ConstEval {
				typedef: Some(ctx.builtins.float64_t.clone()),
				ir: vir::Node::Immediate(vir::Immediate::F64(float)),
			}),
			Err(err) => {
				ctx.raise([Issue {
					id: FileSpan::new(ctx.ipath.as_str(), literal.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::ParseFloat),
					message: format!("invalid floating-point number: {err}"),
					label: None,
				}]);

				Err(())
			}
		}
	} else if let Some(string) = token.string() {
		let istring = ctx.intern_string(string);
		let addr = istring.as_thin_ptr() as usize;

		Ok(ConstEval {
			typedef: Some(ctx.builtins.string_t.clone()),
			ir: todo!(),
		})
	} else if let Some(name) = token.name() {
		let iname = ctx.intern_name(name);
		let addr = iname.as_thin_ptr() as usize;

		Ok(ConstEval {
			typedef: Some(ctx.builtins.iname_t.clone()),
			ir: todo!(),
		})
	} else {
		unreachable!()
	}
}
