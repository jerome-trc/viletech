use rowan::{ast::AstNode, GreenNode, NodeOrToken};

use crate::{
	zdoom::zscript::{ast, Syntax},
	GreenElement,
};

use super::AutoFormatter;

#[must_use]
pub fn expr(f: &mut AutoFormatter, ast: ast::Expr) -> GreenElement {
	match ast {
		ast::Expr::Binary(e_bin) => expr_bin(f, e_bin).into(),
		ast::Expr::Call(_) => todo!(),
		ast::Expr::ClassCast(_) => todo!(),
		ast::Expr::Group(_) => todo!(),
		ast::Expr::Ident(e_ident) => e_ident.token().green().to_owned().into(),
		ast::Expr::Index(_) => todo!(),
		ast::Expr::Literal(e_lit) => e_lit.syntax().green().into_owned().into(),
		ast::Expr::Member(_) => todo!(),
		ast::Expr::Postfix(e_post) => expr_postfix(f, e_post).into(),
		ast::Expr::Prefix(e_pre) => expr_prefix(f, e_pre).into(),
		ast::Expr::Super(e_super) => e_super.token().green().to_owned().into(),
		ast::Expr::Ternary(e_ternary) => expr_ternary(f, e_ternary).into(),
		ast::Expr::Vector(_) => todo!(),
	}
}

#[must_use]
pub fn expr_bin(f: &mut AutoFormatter, ast: ast::BinExpr) -> GreenNode {
	let mut children = vec![];

	let operator = ast.operator().0;

	for elem in ast.syntax().children_with_tokens() {
		match elem {
			NodeOrToken::Node(node) => {
				let is_rhs = ast.syntax().last_child().is_some_and(|c| c == node);

				if let Some(e) = ast::Expr::cast(node) {
					children.push(expr(f, e));
				}

				if !is_rhs {
					children.push(f.ctx.space());
				}
			}
			NodeOrToken::Token(token) => {
				if token.kind() == Syntax::Whitespace {
					continue;
				}

				let mut space_needed = false;
				space_needed |= token.kind() == Syntax::Comment;
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

#[must_use]
pub fn expr_postfix(f: &mut AutoFormatter, ast: ast::PostfixExpr) -> GreenNode {
	let mut children = vec![expr(f, ast.operand())];

	for elem in ast.syntax().children_with_tokens() {
		match elem {
			NodeOrToken::Token(token) => {
				if token.kind() == Syntax::Whitespace {
					continue;
				}

				if token.kind() == Syntax::Comment {
					children.push(f.ctx.space());
					children.push(token.green().to_owned().into());
					children.push(f.ctx.space());
				} else {
					children.push(token.green().to_owned().into());
				}
			}
			NodeOrToken::Node(node) => {
				if node.kind() == Syntax::Error {
					children.push(node.green().into_owned().into());
				}
			}
		}
	}

	GreenNode::new(ast.syntax().kind().into(), children)
}

#[must_use]
pub fn expr_prefix(f: &mut AutoFormatter, ast: ast::PrefixExpr) -> GreenNode {
	let mut children = vec![ast.operator().0.green().to_owned().into()];

	for elem in ast.syntax().children_with_tokens() {
		match elem {
			NodeOrToken::Token(token) => {
				if token.kind() == Syntax::Whitespace {
					continue;
				}

				if token.kind() == Syntax::Comment {
					children.push(f.ctx.space());
					children.push(token.green().to_owned().into());
					children.push(f.ctx.space());
				}
			}
			NodeOrToken::Node(node) => {
				if let Some(e) = ast::Expr::cast(node.clone()) {
					children.push(expr(f, e));
				} else {
					children.push(node.green().into_owned().into());
				}
			}
		}
	}

	GreenNode::new(ast.syntax().kind().into(), children)
}

#[must_use]
pub fn expr_ternary(f: &mut AutoFormatter, ast: ast::TernaryExpr) -> GreenNode {
	let mut children = vec![];
	let mut on_newline = false;

	for elem in ast.syntax().children_with_tokens() {
		match elem {
			NodeOrToken::Token(token) => {
				if token.kind() == Syntax::Whitespace {
					continue;
				}

				let need_newline = matches!(token.kind(), Syntax::RegionStart | Syntax::RegionEnd);

				if need_newline {
					children.push(super::newline(f));
				} else if !on_newline {
					children.push(f.ctx.space());
				}

				children.push(token.green().to_owned().into());

				on_newline = need_newline;
			}
			NodeOrToken::Node(node) => {
				if !on_newline && node.prev_sibling_or_token().is_some() {
					children.push(f.ctx.space());
				}

				if let Some(e) = ast::Expr::cast(node.clone()) {
					children.push(expr(f, e));
				} else {
					children.push(node.green().into_owned().into());
				}
			}
		}
	}

	GreenNode::new(ast.syntax().kind().into(), children)
}
