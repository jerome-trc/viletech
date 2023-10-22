use rowan::{ast::AstNode, GreenNode};

use crate::zdoom::zscript::{ast, Syn};

use super::AutoFormatter;

#[must_use]
pub fn expr(f: &mut AutoFormatter, ast: ast::Expr) -> GreenNode {
	match ast {
		ast::Expr::Binary(e_bin) => expr_bin(f, e_bin),
		ast::Expr::Call(_) => todo!(),
		ast::Expr::ClassCast(_) => todo!(),
		ast::Expr::Group(_) => todo!(),
		ast::Expr::Ident(_) => todo!(),
		ast::Expr::Index(_) => todo!(),
		ast::Expr::Literal(e_lit) => e_lit.syntax().green().into_owned(),
		ast::Expr::Member(_) => todo!(),
		ast::Expr::Postfix(_) => todo!(),
		ast::Expr::Prefix(_) => todo!(),
		ast::Expr::Super(_) => todo!(),
		ast::Expr::Ternary(_) => todo!(),
		ast::Expr::Vector(_) => todo!(),
	}
}

#[must_use]
pub fn expr_bin(f: &mut AutoFormatter, ast: ast::BinExpr) -> GreenNode {
	let mut children = vec![];

	let operator = ast.operator().0;

	for elem in ast.syntax().children_with_tokens() {
		match elem {
			rowan::NodeOrToken::Node(node) => {
				let is_rhs = ast.syntax().last_child().is_some_and(|c| c == node);

				if let Some(e) = ast::Expr::cast(node) {
					children.push(expr(f, e).into());
				}

				if !is_rhs {
					children.push(f.ctx.space());
				}
			}
			rowan::NodeOrToken::Token(token) => {
				if token.kind() == Syn::Whitespace {
					continue;
				}

				let mut space_needed = false;
				space_needed |= token.kind() == Syn::Comment;
				space_needed |= token.index() == operator.index();

				children.push(token.green().to_owned().into());

				if space_needed {
					children.push(f.ctx.space());
				}
			}
		}
	}

	GreenNode::new(ast.syntax().kind().into(), children)
}
