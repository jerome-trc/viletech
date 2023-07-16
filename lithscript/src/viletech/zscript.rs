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

struct ContainerPass1<'p> {
	inner: &'p Pass1<'p>,
	tu: &'p ContainerSource<Syn>,
}

impl<'p> std::ops::Deref for ContainerPass1<'p> {
	type Target = Pass1<'p>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

pub(super) fn pass1(pass: Pass1) {
	pass.src.zscript.par_iter().for_each(|tu| {
		let ctrpass = ContainerPass1 { inner: &pass, tu };

		let ast = SyntaxNode::new_root(tu.root.clone());

		for child in ast.children() {
			match ast::TopLevel::cast(child) {
				Some(ast::TopLevel::ClassDef(classdef)) => {
					ctrpass.declare_class(classdef);
				}
				Some(ast::TopLevel::EnumDef(enumdef)) => {
					ctrpass.declare_enum(enumdef);
				}
				Some(ast::TopLevel::StructDef(structdef)) => {
					ctrpass.declare_struct(structdef);
				}
				Some(ast::TopLevel::ConstDef(constdef)) => {
					ctrpass.declare_const(constdef);
				}
				Some(ast::TopLevel::MixinClassDef(mixindef)) => {
					ctrpass.declare_mixin_class(mixindef);
				}
				_ => continue,
			}
		}
	});
}

impl ContainerPass1<'_> {
	fn declare_class(&self, classdef: ast::ClassDef) {
		self.declare(&self.tu.path, [classdef.name().unwrap().text()]);

		for innard in classdef.innards() {
			match innard {
				ast::ClassInnard::Const(constdef) => self.declare_const(constdef),
				ast::ClassInnard::Enum(enumdef) => self.declare_enum(enumdef),
				ast::ClassInnard::StaticConst(sconst) => self.declare_static_const(sconst),
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

	fn declare_const(&self, constdef: ast::ConstDef) {
		self.declare(&self.tu.path, [constdef.name().unwrap().text()]);
	}

	fn declare_static_const(&self, sconst: ast::StaticConstStat) {
		self.declare(&self.tu.path, [sconst.name().unwrap().text()]);
	}

	fn declare_enum(&self, enumdef: ast::EnumDef) {
		self.declare(&self.tu.path, [enumdef.name().unwrap().text()]);

		for variant in enumdef.variants() {
			self.declare(&self.tu.path, [variant.name().text()]);
		}
	}

	fn declare_mixin_class(&self, mixindef: ast::MixinClassDef) {
		self.declare(&self.tu.path, [mixindef.name().unwrap().text()]);
	}

	fn declare_struct(&self, structdef: ast::StructDef) {
		self.declare(&self.tu.path, [structdef.name().unwrap().text()]);

		for innard in structdef.innards() {
			match innard {
				ast::StructInnard::Const(constdef) => self.declare_const(constdef),
				ast::StructInnard::Enum(enumdef) => self.declare_enum(enumdef),
				ast::StructInnard::StaticConst(sconst) => self.declare_static_const(sconst),
				ast::StructInnard::Function(_) => todo!(),
				ast::StructInnard::Field(_) => {}
			}
		}
	}
}

// Pass 2 //////////////////////////////////////////////////////////////////////

struct ContainerPass2<'p> {
	inner: &'p Pass2<'p>,
	tu: &'p ContainerSource<Syn>,
}

impl<'p> std::ops::Deref for ContainerPass2<'p> {
	type Target = Pass2<'p>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

pub(super) fn pass2(pass: Pass2) {
	pass.src.zscript.par_iter().for_each(|tu| {
		let ctrpass = ContainerPass2 { inner: &pass, tu };

		let ast = SyntaxNode::new_root(tu.root.clone());

		for child in ast.children() {
			match ast::TopLevel::cast(child) {
				Some(ast::TopLevel::ClassDef(classdef)) => {
					ctrpass.semcheck_class(classdef);
				}
				Some(ast::TopLevel::EnumDef(enumdef)) => {
					ctrpass.define_enum(enumdef);
				}
				Some(ast::TopLevel::ConstDef(constdef)) => {
					ctrpass.define_const(constdef);
				}
				_ => continue,
			}
		}
	});
}

impl ContainerPass2<'_> {
	fn semcheck_class(&self, classdef: ast::ClassDef) {
		let decl = self.get_z(classdef.name().unwrap().text()).unwrap();

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
					self.raise(Issue {
						id: FileSpan::new(&self.tu.path, classdef.syntax().text_range()),
						level: IssueLevel::Error(issue::Error::IllegalStructQual),
						label: Some(issue::Label::new(
							&self.tu.path,
							token.text_range(),
							"class qualifier `native` is forbidden for transpiled ZScript"
								.to_string(),
						)),
					});

					return;
				}
				_ => unimplemented!(),
			}
		}
	}

	fn define_const(&self, constdef: ast::ConstDef) {
		let expr = constdef.initializer().unwrap();

		match expr {
			ast::Expr::Binary(e_bin) => {
				self.lower_bin_expr(e_bin);
			}
			ast::Expr::ClassCast(e_cc) => {
				self.raise(Issue {
					id: FileSpan::new(&self.tu.path, e_cc.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::IllegalConstInit),
					label: Some(issue::Label::new(
						&self.tu.path,
						e_cc.syntax().text_range(),
						"class cast expressions are never valid in constant definitions"
							.to_string(),
					)),
				});
			}
			ast::Expr::Super(e_super) => {
				self.raise(Issue {
					id: FileSpan::new(&self.tu.path, constdef.syntax().text_range()),
					level: IssueLevel::Error(issue::Error::IllegalConstInit),
					label: Some(issue::Label::new(
						&self.tu.path,
						e_super.syntax().text_range(),
						"`super` expression can only be used in class methods".to_string(),
					)),
				});
			}
			_ => todo!(),
		}
	}

	/// Defines the pre-declared symbol as an alias
	/// to one of Lith's integral built-in types.
	fn define_enum(&self, enumdef: ast::EnumDef) {
		let decl = self.get_z(enumdef.name().unwrap().text()).unwrap();

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

		let underlying = self.symtab.symbols.get(&underlying_name).unwrap();
		let sym_g = underlying.load();
		let lir::Symbol::Type(typedef) = sym_g.as_ref() else { unreachable!() };
		let TypeInfo::Num(_) = typedef.inner() else { unreachable!() };

		decl.value()
			.inner
			.rcu(|_| Arc::new(lir::Symbol::Type(typedef.clone())));
	}

	fn lower_bin_expr(&self, expr: ast::BinExpr) {
		let lhs_t = self.expr_type(expr.lhs());
		let rhs_t = self.expr_type(expr.rhs());
	}

	fn expr_type(&self, expr: ast::Expr) -> Option<&rti::Handle<TypeDef>> {
		match expr {
			ast::Expr::Binary(e_bin) => {
				let lhs_t = self.expr_type(e_bin.lhs());
				let rhs_t = self.expr_type(e_bin.rhs());

				if lhs_t != rhs_t {
					self.raise(Issue {
						level: IssueLevel::Error(issue::Error::BinExprTypeMismatch),
						id: FileSpan::new(&self.tu.path, e_bin.syntax().text_range()),
						label: None,
					});

					return None;
				}

				todo!()
			}
			ast::Expr::Call(_) => todo!(),
			ast::Expr::ClassCast(_) => todo!(),
			ast::Expr::Group(e_grp) => self.expr_type(e_grp.inner()),
			ast::Expr::Ident(_) => todo!(),
			ast::Expr::Index(e_index) => self.expr_type(e_index.indexed()),
			ast::Expr::Literal(e_lit) => {
				let token = e_lit.token();

				if token.null() {
					todo!() // ptr
				} else if token.bool().is_some() {
					todo!() // bool
				} else if token.float().is_some() {
					todo!() // float
				} else if token.string().is_some() {
					todo!() // string
				} else if token.int().is_some() {
					todo!() // integer
				} else if token.name().is_some() {
					todo!() // TBD
				} else {
					unreachable!()
				}
			}
			ast::Expr::Member(e_member) => {
				todo!()
			}
			ast::Expr::Postfix(e_post) => self.expr_type(e_post.operand()),
			ast::Expr::Prefix(e_pre) => self.expr_type(e_pre.operand()),
			ast::Expr::Super(_) => todo!(),
			ast::Expr::Ternary(_) => todo!(),
			ast::Expr::Vector(e_vec) => {
				if e_vec.w().is_some() {
					todo!() // dvec4
				} else if e_vec.z().is_some() {
					todo!() // dvec3
				} else {
					todo!() // dvec2
				}
			}
		}
	}
}
