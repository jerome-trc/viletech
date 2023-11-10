mod actor;
mod common;
mod expr;
mod stat;
mod structure;
mod top;
mod types;

#[cfg(test)]
mod test;

use crate::{parser::Parser, zdoom::Token};

use super::Syn;

use self::common::*;

pub use self::{actor::*, expr::*, stat::*, structure::*, top::*, types::*};

/// Builds a [`Syn::Root`] node.
pub fn file(p: &mut Parser<Syn>) {
	let root = p.open();

	while !p.eof() {
		if trivia_no_doc(p) {
			continue;
		}

		let token = p.find(0, |token| !token.is_trivia());

		match token {
			Token::KwClass => {
				class_def(p);
				continue;
			}
			Token::KwStruct => {
				struct_def(p);
				continue;
			}
			Token::KwMixin => {
				mixin_class_def(p);
				continue;
			}
			Token::KwConst => {
				const_def(p);
				continue;
			}
			Token::KwEnum => {
				enum_def(p);
				continue;
			}
			_ => {}
		}

		if p.at(Token::DocComment) {
			// Top-level items outside this set can not start with a doc comment.
			p.advance(Syn::Comment);
			continue;
		}

		match token {
			Token::KwExtend => class_or_struct_extend(p),
			Token::KwInclude => include_directive(p),
			Token::KwVersion => version_directive(p),
			_ => p.advance_with_error(
				Syn::from(p.nth(0)),
				&[&[
					"`const`",
					"`enum`",
					"`class`",
					"`struct`",
					"`mixin`",
					"`extend`",
					"`#include`",
					"`version`",
				]],
			),
		}
	}

	p.close(root, Syn::Root);
}
