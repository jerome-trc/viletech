//! Transpilation, semantic checking, and LIR lowering for GZDoom's ZScript.

use std::sync::Arc;

use doomfront::{
	rowan::ast::AstNode,
	zdoom::zscript::{ast, Syn, SyntaxNode},
};
use rayon::prelude::*;
use util::rstring::RString;

use crate::{
	compile::QName,
	issue::{self, FileSpan, Issue, IssueLevel},
	lir, rti,
	tsys::{ClassType, TypeDef, TypeInfo},
};

use super::{ContainerSource, Pass1, Pass2};

pub(super) fn pass1(pass: Pass1) {
	pass.src.zscript.par_iter().for_each(|tu| {
		let ast = SyntaxNode::new_root(tu.root.clone());

		for child in ast.children() {
			match ast::TopLevel::cast(child) {
				Some(ast::TopLevel::ClassDef(classdef)) => {
					declare_class(&pass, tu, classdef);
				}
				Some(ast::TopLevel::EnumDef(enumdef)) => {
					declare_enum(&pass, tu, enumdef);
				}
				Some(ast::TopLevel::StructDef(structdef)) => {
					declare_struct(&pass, tu, structdef);
				}
				Some(ast::TopLevel::ConstDef(constdef)) => {
					declare_const(&pass, tu, constdef);
				}
				Some(ast::TopLevel::MixinClassDef(mixindef)) => {
					declare_mixin_class(&pass, tu, mixindef);
				}
				_ => continue,
			}
		}
	});
}

fn declare_class(pass: &Pass1, tu: &ContainerSource<Syn>, classdef: ast::ClassDef) {
	pass.declare(&tu.path, [classdef.name().unwrap().text()]);

	for innard in classdef.innards() {
		match innard {
			ast::ClassInnard::Const(constdef) => declare_const(pass, tu, constdef),
			ast::ClassInnard::Enum(enumdef) => declare_enum(pass, tu, enumdef),
			ast::ClassInnard::StaticConst(sconst) => declare_static_const(pass, tu, sconst),
			ast::ClassInnard::Function(_) => todo!(),
			ast::ClassInnard::Default(_)
			| ast::ClassInnard::Field(_)
			| ast::ClassInnard::Flag(_)
			| ast::ClassInnard::Mixin(_)
			| ast::ClassInnard::Property(_)
			| ast::ClassInnard::States(_) => {}
		}
	}
}

fn declare_const(pass: &Pass1, tu: &ContainerSource<Syn>, constdef: ast::ConstDef) {
	pass.declare(&tu.path, [constdef.name().unwrap().text()]);
}

fn declare_static_const(pass: &Pass1, tu: &ContainerSource<Syn>, sconst: ast::StaticConstStat) {
	pass.declare(&tu.path, [sconst.name().unwrap().text()]);
}

fn declare_enum(pass: &Pass1, tu: &ContainerSource<Syn>, enumdef: ast::EnumDef) {
	pass.declare(&tu.path, [enumdef.name().unwrap().text()]);

	for variant in enumdef.variants() {
		pass.declare(&tu.path, [variant.name().text()]);
	}
}

fn declare_mixin_class(pass: &Pass1, tu: &ContainerSource<Syn>, mixindef: ast::MixinClassDef) {
	pass.declare(&tu.path, [mixindef.name().unwrap().text()]);
}

fn declare_struct(pass: &Pass1, tu: &ContainerSource<Syn>, structdef: ast::StructDef) {
	pass.declare(&tu.path, [structdef.name().unwrap().text()]);

	for innard in structdef.innards() {
		match innard {
			ast::StructInnard::Const(constdef) => declare_const(pass, tu, constdef),
			ast::StructInnard::Enum(enumdef) => declare_enum(pass, tu, enumdef),
			ast::StructInnard::StaticConst(sconst) => declare_static_const(pass, tu, sconst),
			ast::StructInnard::Function(_) => todo!(),
			ast::StructInnard::Field(_) => {}
		}
	}
}

pub(super) fn pass2(pass: Pass2) {
	pass.src.zscript.par_iter().for_each(|tu| {
		let ast = SyntaxNode::new_root(tu.root.clone());

		for child in ast.children() {
			match ast::TopLevel::cast(child) {
				Some(ast::TopLevel::ClassDef(classdef)) => {
					semcheck_class(&pass, tu, classdef);
				}
				Some(ast::TopLevel::EnumDef(enumdef)) => {
					define_enum(&pass, tu, enumdef);
				}
				Some(ast::TopLevel::ConstDef(constdef)) => {
					define_const(&pass, tu, constdef);
				}
				_ => continue,
			}
		}
	});
}

fn semcheck_class(pass: &Pass2, tu: &ContainerSource<Syn>, classdef: ast::ClassDef) {
	let decl = pass.get_z(classdef.name().unwrap().text()).unwrap();

	decl.value().inner.rcu(|_| {
		let class_t = ClassType { parent: None };
		let store = rti::Store::new(
			RString::new(decl.key().as_str()),
			TypeDef::new_class(class_t),
		);
		Arc::new(lir::Symbol::Type(Arc::new(store).into()))
	});

	for qual in classdef.qualifiers() {
		match qual {
			ast::ClassQual::Native(token) => {
				pass.raise(Issue {
					id: FileSpan::new(&tu.path, classdef.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::IllegalStructQual),
					label: Some(issue::Label::new(
						&tu.path,
						token.text_range(),
						"class qualifier `native` is forbidden for transpiled ZScript".to_string(),
					)),
				});

				return;
			}
			_ => unimplemented!(),
		}
	}
}

fn define_const(pass: &Pass2, tu: &ContainerSource<Syn>, constdef: ast::ConstDef) {
	let expr = constdef.initializer().unwrap();

	match expr {
		ast::Expr::ClassCast(e_cc) => {
			pass.raise(Issue {
				id: FileSpan::new(&tu.path, e_cc.syntax().text_range()),
				level: IssueLevel::Error(issue::Error::IllegalConstInit),
				label: Some(issue::Label::new(
					&tu.path,
					e_cc.syntax().text_range(),
					"class cast expressions are never valid in constant definitions".to_string(),
				)),
			});
		}
		ast::Expr::Super(e_super) => {
			pass.raise(Issue {
				id: FileSpan::new(&tu.path, constdef.syntax().text_range()),
				level: IssueLevel::Error(issue::Error::IllegalConstInit),
				label: Some(issue::Label::new(
					&tu.path,
					e_super.syntax().text_range(),
					"`super` expression can only be used in class methods".to_string(),
				)),
			});
		}
		_ => todo!(),
	}
}

/// Defines the pre-declared symbol as an alias to one of Lith's integral built-in types.
fn define_enum(pass: &Pass2, _: &ContainerSource<Syn>, enumdef: ast::EnumDef) {
	let decl = pass.get_z(enumdef.name().unwrap().text()).unwrap();

	let underlying_name = if let Some((_, enum_t)) = enumdef.type_spec() {
		let aliased = match enum_t {
			ast::EnumType::KwInt8 | ast::EnumType::KwSByte => "int8",
			ast::EnumType::KwUInt8 | ast::EnumType::KwByte => "uint8",
			ast::EnumType::KwInt16 | ast::EnumType::KwShort => "int16",
			ast::EnumType::KwUInt16 | ast::EnumType::KwUShort => "uint16",
			ast::EnumType::KwInt => "int",
			ast::EnumType::KwUInt => "uint",
		};

		QName::new_value_name("/lith", [aliased])
	} else {
		QName::new_value_name("/lith", ["int"])
	};

	let underlying = pass.symtab.symbols.get(&underlying_name).unwrap();
	let sym_g = underlying.load();
	let lir::Symbol::Type(typedef) = sym_g.as_ref() else { unreachable!() };
	let TypeInfo::Num(_) = typedef.inner() else { unreachable!() };

	decl.value()
		.inner
		.rcu(|_| Arc::new(lir::Symbol::Type(typedef.clone())));
}
