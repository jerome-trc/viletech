use std::collections::HashMap;

use doom_front::zscript::{
	ast,
	err::ParsingErrorLevel as ParseIssueLevel,
	filesystem::Files,
	interner::NameSymbol,
	ir_common::{self as ir, DottableId, EnumVariant, Expression, IntType},
	Span,
};

use super::ParseOutput;

/// Instantiated when performing an asset load if a namespace has any ZScript in it.
/// Emits one complete Lith module per namespace, accumulating types and constants
/// so that later namespaces can reference symbols from earlier ones.
#[derive(Debug, Default)]
pub struct Transpiler {
	trees: Vec<FileTree>,
}

impl Transpiler {
	/// Performs transpilation on the ZScript tree in one namespace (whose ID
	/// must be given). The transpiler itself goes un-mutated if any errors arise.
	pub fn transpile(
		&mut self,
		namespace_id: String,
		parsed: ParseOutput,
	) -> Result<String, Vec<Issue>> {
		debug_assert!(!parsed.issues.iter().any(|issue| match issue.level {
			ParseIssueLevel::Error => true,
			ParseIssueLevel::Warning => false,
		}));

		let mut tree = FileTree {
			id: namespace_id,
			files: parsed.files,
			classes: Default::default(),
			mixins: Default::default(),
			enums: Default::default(),
			constants: Default::default(),
		};

		let mut issues = Vec::<Issue>::default();

		// Pass 1: collect declarations

		for ast in parsed.asts {
			for topdef in ast.ast.definitions {
				match topdef.kind {
					ast::TopLevelDefinitionKind::Class(def) => {
						tree.declare_class(def, &mut issues);
					}
					ast::TopLevelDefinitionKind::Struct(def) => {
						tree.declare_struct(def, &mut issues);
					}
					ast::TopLevelDefinitionKind::MixinClass(def) => {
						tree.declare_mixin(def);
					}
					ast::TopLevelDefinitionKind::Enum(def) => {
						tree.declare_enum(def);
					}
					ast::TopLevelDefinitionKind::Const(def) => {
						tree.declare_constant(def);
					}
					_ => {
						// Extend class/struct definitions get pushed back
						// to be consumed in another pass
						// ast.ast.definitions.push(topdef);
					}
				}
			}
		}

		// Success: store build artifacts for later namespaces

		self.trees.push(tree);

		unimplemented!()
	}
}

#[derive(Debug)]
struct FileTree {
	id: String,
	files: Files,
	classes: HashMap<NameSymbol, ClassDef>,
	mixins: HashMap<NameSymbol, MixinDef>,
	enums: HashMap<NameSymbol, EnumDef>,
	constants: HashMap<NameSymbol, ConstantDef>,
}

impl FileTree {
	fn declare_class(&mut self, astdef: ast::ClassDefinition, issues: &mut Vec<Issue>) {
		let ast::ClassDefinition {
			doc_comment: _,
			span: _,
			name,
			ancestor,
			metadata,
			inners,
		} = astdef;

		let mut def = ClassDef {
			name: name.symbol,
			is_abstract: false,
			scope: Scope::Data,
			replaces: None,
			ancestor,
			inners,
		};

		let mut scope_specified = false;

		for metadata in metadata {
			match metadata.kind {
				ast::ClassMetadataItemKind::Abstract => {
					if !def.is_abstract {
						def.is_abstract = true;
					} else {
						issues.push(Issue {
							level: IssueLevel::Error(Error::MultiAbstract(name.symbol)),
							span: metadata.span,
						})
					}
				}
				ast::ClassMetadataItemKind::Native => {
					issues.push(Issue {
						level: IssueLevel::Error(Error::NativeClass(name.symbol)),
						span: metadata.span,
					});
				}
				ast::ClassMetadataItemKind::UI => {
					if !scope_specified {
						scope_specified = true;
						def.scope = Scope::Ui;
					} else {
						issues.push(Issue {
							level: IssueLevel::Error(Error::MultiScope(name.symbol)),
							span: metadata.span,
						});
					}
				}
				ast::ClassMetadataItemKind::Play => {
					if !scope_specified {
						scope_specified = true;
						def.scope = Scope::Play;
					} else {
						issues.push(Issue {
							level: IssueLevel::Error(Error::MultiScope(name.symbol)),
							span: metadata.span,
						});
					}
				}
				ast::ClassMetadataItemKind::Replaces(replace) => {
					if def.replaces.is_none() {
						def.replaces = Some(replace);
					} else {
						issues.push(Issue {
							level: IssueLevel::Error(Error::MultiReplace(name.symbol)),
							span: metadata.span,
						});
					}
				}
				_ => {}
			}
		}

		if any_errors(issues) {
			return;
		}

		self.classes.insert(name.symbol, def);
	}

	fn declare_struct(&mut self, astdef: ast::StructDefinition, issues: &mut Vec<Issue>) {
		let ast::StructDefinition {
			doc_comment: _,
			span: _,
			name,
			metadata,
			mut inners,
		} = astdef;

		let inners = inners
			.drain(..)
			.map(|inner| ast::ClassInner {
				span: inner.span,
				kind: match inner.kind {
					ast::StructInnerKind::Declaration(decl) => {
						ast::ClassInnerKind::Declaration(decl)
					}
					ast::StructInnerKind::Enum(enum_def) => ast::ClassInnerKind::Enum(enum_def),
					ast::StructInnerKind::Const(const_def) => ast::ClassInnerKind::Const(const_def),
					ast::StructInnerKind::StaticConstArray(scarr) => {
						ast::ClassInnerKind::StaticConstArray(scarr)
					}
				},
			})
			.collect();

		let mut def = ClassDef {
			name: name.symbol,
			is_abstract: false,
			scope: Scope::Data,
			replaces: None,
			ancestor: None,
			inners,
		};

		let mut scope_specified = false;

		for metadata in metadata {
			match metadata.kind {
				ast::StructMetadataItemKind::ClearScope => {
					if !scope_specified {
						scope_specified = true;
						// `Data` is already the default, but set it again anyway
						// so no other code changes affect the correctness of this case
						def.scope = Scope::Data;
					} else {
						issues.push(Issue {
							level: IssueLevel::Error(Error::MultiScope(name.symbol)),
							span: metadata.span,
						});
					}
				}
				ast::StructMetadataItemKind::Native => {
					issues.push(Issue {
						level: IssueLevel::Error(Error::NativeClass(name.symbol)),
						span: metadata.span,
					});
				}
				ast::StructMetadataItemKind::UI => {
					if !scope_specified {
						scope_specified = true;
						def.scope = Scope::Ui;
					} else {
						issues.push(Issue {
							level: IssueLevel::Error(Error::MultiScope(name.symbol)),
							span: metadata.span,
						});
					}
				}
				ast::StructMetadataItemKind::Play => {
					if !scope_specified {
						scope_specified = true;
						def.scope = Scope::Play;
					} else {
						issues.push(Issue {
							level: IssueLevel::Error(Error::MultiScope(name.symbol)),
							span: metadata.span,
						});
					}
				}
				_ => {}
			}
		}

		self.classes.insert(name.symbol, def);
	}

	fn declare_mixin(&mut self, mut astdef: ast::MixinClassDefinition) {
		self.mixins.insert(
			astdef.name.symbol,
			MixinDef {
				name: astdef.name.symbol,
				#[allow(unreachable_code)]
				inners: astdef
					.inners
					.drain(..)
					.map(|inner| ast::ClassInner {
						span: inner.span,
						kind: unimplemented!("`inner.kind.map_to_class_inner_kind()`"),
					})
					.collect(),
			},
		);
	}

	fn declare_enum(&mut self, astdef: ir::EnumDefinition) {
		self.enums.insert(
			astdef.name.symbol,
			EnumDef {
				name: astdef.name.symbol,
				underlying: astdef.enum_type,
				variants: astdef.variants,
			},
		);
	}

	fn declare_constant(&mut self, astdef: ir::ConstDefinition) {
		self.constants.insert(
			astdef.name.symbol,
			ConstantDef {
				name: astdef.name.symbol,
				expr: astdef.expr,
			},
		);
	}
}

#[derive(Debug, Default, PartialEq, Eq)]
enum Scope {
	#[default]
	Data,
	Ui,
	Play,
}

#[derive(Debug)]
struct ClassDef {
	name: NameSymbol,
	is_abstract: bool,
	scope: Scope,
	replaces: Option<DottableId>,
	/// This should only be `None` if this is ZScript's base `Object` class,
	/// or if this is a struct, which can't inherit.
	ancestor: Option<DottableId>,
	inners: Vec<ast::ClassInner>,
}

#[derive(Debug)]
struct MixinDef {
	name: NameSymbol,
	inners: Vec<ast::ClassInner>,
}

#[derive(Debug)]
struct EnumDef {
	name: NameSymbol,
	underlying: Option<IntType>,
	variants: Vec<EnumVariant>,
}

#[derive(Debug)]
struct ConstantDef {
	name: NameSymbol,
	expr: Expression,
}

#[derive(Debug)]
pub struct Issue {
	level: IssueLevel,
	span: Span,
}

#[derive(Debug)]
pub enum IssueLevel {
	Warning(Warning),
	Error(Error),
}

#[derive(Debug)]
pub enum Warning {
	// ???
}

#[derive(Debug)]
pub enum Error {
	/// Attempted to define a class with the `ui` scope qualifier more than once.
	MultiAbstract(NameSymbol),
	/// Attempted to define more than one `replaces` directive for a class.
	MultiReplace(NameSymbol),
	/// Attempted to add more than one scope qualifier to a class or struct
	/// definition (`ui`, `play`, `clearScope`).
	MultiScope(NameSymbol),
	/// Attempted to define a class or struct with the `native` qualifier.
	NativeClass(NameSymbol),
	/// Attempted to add the `replaces` directive to a class that doesn't
	/// inherit from ZScript's `Actor` class.
	NonActorReplacement(NameSymbol),
	/// Attempted to add the directive `replaces XYZ` to a class, where `XYX`
	/// doesn't inherit from ZScript's `Actor` class.
	NonActorReplacee {
		replacement: NameSymbol,
		replacee: NameSymbol,
	},
}

#[must_use]
fn any_errors(issues: &[Issue]) -> bool {
	issues.iter().any(|issue| match issue.level {
		IssueLevel::Warning(_) => false,
		IssueLevel::Error(_) => true,
	})
}
