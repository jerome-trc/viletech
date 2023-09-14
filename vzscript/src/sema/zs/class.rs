use doomfront::{
	rowan::{ast::AstNode, TextRange},
	zdoom::zscript::{ast, SyntaxToken},
};
use triomphe::Arc;
use util::rstring::RString;

use crate::{
	compile::{
		symbol::{DefKind, DefStatus, Definition, Symbol},
		Scope,
	},
	issue::{self, Issue},
	rti,
	sema::SemaContext,
	tsys::{self, ClassType, TypeDef},
	zname::ZName,
};

pub(super) fn define(
	ctx: &SemaContext,
	qname: ZName,
	scope: Scope,
	classdef: ast::ClassDef,
) -> Result<Arc<Definition>, ()> {
	let mut class_t = ClassType {
		parent: None,
		is_abstract: false,
		restrict: tsys::Restrict::None,
	};

	process_qualifiers(ctx, &classdef, &mut class_t)?;

	let mut valid = true;

	for innard in classdef.innards() {
		match innard {
			ast::ClassInnard::Const(_)
			| ast::ClassInnard::Enum(_)
			| ast::ClassInnard::Function(_)
			| ast::ClassInnard::Struct(_)
			| ast::ClassInnard::StaticConst(_)
			| ast::ClassInnard::Field(_)
			| ast::ClassInnard::Default(_)
			| ast::ClassInnard::States(_)
			| ast::ClassInnard::Property(_)
			| ast::ClassInnard::Flag(_) => todo!(),
			ast::ClassInnard::Mixin(_) => continue,
		}
	}

	if !valid {
		return Err(());
	}

	let store = rti::Store::new(qname, TypeDef::new_class(class_t));
	let record = rti::Record::new_type(store);
	let handle = record.handle_type();

	Ok(Arc::new(Definition {
		kind: DefKind::Class {
			typedef: record,
			handle: handle.downcast().unwrap(),
		},
		scope,
	}))
}

fn process_qualifiers(
	ctx: &SemaContext,
	classdef: &ast::ClassDef,
	typedef: &mut ClassType,
) -> Result<(), ()> {
	fn report_scope_qual_overlap(ctx: &SemaContext, token: SyntaxToken, prev: &SyntaxToken) {
		ctx.raise(
			Issue::new(
				ctx.path,
				token.text_range(),
				"class scope can only be specified once".to_string(),
				issue::Level::Error(issue::Error::QualifierOverlap),
			)
			.with_label(
				ctx.path,
				prev.text_range(),
				"class already qualified with a scope here".to_string(),
			),
		);
	}

	let mut valid = true;
	let mut restrict = None;

	for qual in classdef.qualifiers() {
		match qual {
			ast::ClassQual::Replaces(_) => todo!(),
			ast::ClassQual::Abstract(_) => {
				typedef.is_abstract = true;
			}
			ast::ClassQual::Play(token) => {
				if let Some(prev) = &restrict {
					report_scope_qual_overlap(ctx, token, prev);
					valid = false;
				} else {
					typedef.restrict = tsys::Restrict::Sim;
					restrict = Some(token);
				}
			}
			ast::ClassQual::Ui(token) => {
				if let Some(prev) = &restrict {
					report_scope_qual_overlap(ctx, token, prev);
					valid = false;
				} else {
					typedef.restrict = tsys::Restrict::Ui;
					restrict = Some(token);
				}
			}
			ast::ClassQual::Native(_) => {
				let name_tok = classdef.name().unwrap();
				let r_start = classdef.syntax().text_range().start();
				let r_end = name_tok.text_range().start();

				ctx.raise(Issue::new(
					ctx.path,
					TextRange::new(r_start, r_end),
					"`native` ZScript symbols cannot be transpiled".to_string(),
					issue::Level::Error(issue::Error::IllegalClassQual),
				));

				valid = false;
			}
			ast::ClassQual::Version(_) => {}
		}
	}

	valid.then_some(()).ok_or(())
}
