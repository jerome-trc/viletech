//! Name resolution and semantic checking for GZDoom's ZScript.

use doomfront::{
	rowan::ast::AstNode,
	zdoom::{
		inctree::FileParseTree,
		zscript::{ast, Syn, SyntaxNode},
	},
};

use crate::{
	compile::{Compiler, Scope, SymbolPtr},
	front::{Symbol, UndefKind},
	issue::{self, FileSpan, Issue, IssueLevel},
};

use super::{CompilerExt, ExtCrit, ZName};

pub(super) struct DeclPass<'c> {
	pub(super) compiler: &'c Compiler<CompilerExt>,
	pub(super) ext: &'c mut ExtCrit,
	pub(super) ptree: &'c FileParseTree<Syn>,
	pub(super) cur_path: &'c String,
}

impl DeclPass<'_> {
	pub(super) fn run(&mut self) {
		let ast = self.ptree.cursor();

		// First "sub-pass" needs to make mixin classes known so their contents
		// can be expanded in class definitions by the second "sub-pass".

		for child in ast.children() {
			let top = ast::TopLevel::cast(child).unwrap();

			let ast::TopLevel::MixinClassDef(mixindef) = top else {
				continue;
			};

			self.declare_mixin_class(mixindef);
		}

		for child in ast.children() {
			let top = ast::TopLevel::cast(child).unwrap();

			match top {
				ast::TopLevel::ClassDef(classdef) => {
					self.declare_class(classdef);
				}
				ast::TopLevel::ClassExtend(_) => todo!(),
				ast::TopLevel::ConstDef(_) => todo!(),
				ast::TopLevel::EnumDef(_) => todo!(),
				ast::TopLevel::StructDef(_) => todo!(),
				ast::TopLevel::StructExtend(_) => todo!(),
				ast::TopLevel::MixinClassDef(_)
				| ast::TopLevel::Version(_)
				| ast::TopLevel::Include(_) => continue,
			}
		}
	}

	fn declare_class(&mut self, classdef: ast::ClassDef) {
		let name_tok = classdef.name().unwrap();
		let zname = ZName::Type(self.compiler.interner().intern(name_tok.text()));

		self.ext.cur_scope_mut().scope.insert(
			zname,
			SymbolPtr::from(Symbol::Undefined {
				kind: UndefKind::Struct,
				scope: Scope::default(),
			}),
		);

		let mut scope = Scope::<ZName>::default();

		for innard in classdef.innards() {
			match innard {
				ast::ClassInnard::Const(_) => todo!(),
				ast::ClassInnard::Enum(_) => todo!(),
				ast::ClassInnard::StaticConst(_) => todo!(),
				ast::ClassInnard::Function(_) => todo!(),
				ast::ClassInnard::Field(field) => self.declare_field(&mut scope, field),
				ast::ClassInnard::Mixin(mixin_stat) => self.expand_mixin(mixin_stat),
				ast::ClassInnard::Default(_)
				| ast::ClassInnard::Flag(_)
				| ast::ClassInnard::Property(_)
				| ast::ClassInnard::States(_) => {}
			}
		}
	}

	fn declare_field(&mut self, scope: &mut Scope<ZName>, field: ast::FieldDecl) {
		for name in field.names() {
			let iname = self.compiler.interner.intern(name.ident().text());
			let zname = ZName::Value(iname);

			scope.insert(
				zname,
				SymbolPtr::from(Symbol::Undefined {
					kind: UndefKind::Value { mutable: true },
					scope: Scope::default(),
				}),
			);
		}
	}

	fn declare_mixin_class(&mut self, mixindef: ast::MixinClassDef) {
		let name_tok = mixindef.name().unwrap();
		let zname = ZName::Type(self.compiler.interner().intern(name_tok.text()));
		let node = mixindef.syntax().green().into_owned();
		self.ext.cur_scope_mut().mixins.insert(zname, node);
	}

	fn expand_mixin(&mut self, mixin_stat: ast::MixinStat) {
		let name_tok = mixin_stat.name().unwrap();
		let zname = ZName::Type(self.compiler.interner().intern(name_tok.text()));

		let Some(green) = self.ext.scopes.iter().rev().find_map(|lscope| {
			lscope.mixins.get(&zname)
		}) else {
			self.compiler.raise(Issue {
				id: FileSpan::new(self.cur_path, name_tok.text_range()),
				level: IssueLevel::Error(issue::Error::SymbolNotFound),
				message: format!("mixin `{}` not found", name_tok.text()),
				label: None,
			});

			return;
		};

		let cursor = SyntaxNode::new_root(green.clone());
		let mixindef = ast::MixinClassDef::cast(cursor).unwrap();

		for innard in mixindef.innards() {
			match innard {
				ast::ClassInnard::Const(constdef) => todo!(),
				ast::ClassInnard::Enum(enumdef) => todo!(),
				ast::ClassInnard::StaticConst(sconst) => todo!(),
				ast::ClassInnard::Function(fndecl) => todo!(),
				ast::ClassInnard::Field(field) => {}
				ast::ClassInnard::Mixin(_) => {
					// Note that there's nothing stopping LithV from recursively
					// expanding mixins, but it is a parser error to put a mixin
					// statement in a mixin class definition as of GZDoom 4.10.0,
					// so this is unreachable code.
				}
				ast::ClassInnard::Default(_)
				| ast::ClassInnard::States(_)
				| ast::ClassInnard::Property(_)
				| ast::ClassInnard::Flag(_) => continue,
			}
		}
	}
}

/*

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

*/
