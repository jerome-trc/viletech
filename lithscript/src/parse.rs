//! Functions for parsing Lith source code.

mod common;
mod expr;
mod item;
mod stat;
#[cfg(test)]
mod test;

use doomfront::parser::Parser;

use crate::Syn;

use self::{common::*, item::*};

impl doomfront::LangExt for Syn {
	type Token = Self;
	const EOF: Self::Token = Self::Eof;
	const ERR_NODE: Self::Kind = Self::Error;
}

pub type Error = doomfront::ParseError<Syn>;

pub fn file(p: &mut Parser<Syn>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		item(p);
	}

	p.close(root, Syn::FileRoot);
}

/// Useful if using a parser generator or combinators.
#[cfg(any())]
mod help {
	#[must_use]
	fn coalesce_node<C: Coalesce>(input: C, syn: impl Into<SyntaxKind>) -> GreenNode {
		let mut elems = vec![];
		input.coalesce(&mut elems);
		GreenNode::new(syn.into(), elems)
	}

	#[must_use]
	fn coalesce_vec<C: Coalesce>(input: C) -> Vec<GreenElement> {
		let mut ret = vec![];
		input.coalesce(&mut ret);
		ret
	}

	trait Coalesce: 'static {
		fn coalesce(self, container: &mut Vec<GreenElement>);
	}

	impl Coalesce for GreenNode {
		fn coalesce(self, container: &mut Vec<GreenElement>) {
			container.push(self.into());
		}
	}

	impl Coalesce for GreenToken {
		fn coalesce(self, container: &mut Vec<GreenElement>) {
			container.push(self.into());
		}
	}

	impl Coalesce for GreenElement {
		fn coalesce(self, container: &mut Vec<GreenElement>) {
			container.push(self);
		}
	}

	impl<T> Coalesce for Option<T>
	where
		T: Coalesce,
	{
		fn coalesce(self, container: &mut Vec<GreenElement>) {
			if let Some(input) = self {
				input.coalesce(container);
			}
		}
	}

	impl<T> Coalesce for Vec<T>
	where
		T: Coalesce,
	{
		fn coalesce(self, container: &mut Vec<GreenElement>) {
			for input in self {
				input.coalesce(container);
			}
		}
	}

	#[impl_trait_for_tuples::impl_for_tuples(1, 10)]
	impl Coalesce for Tuple {
		fn coalesce(self, container: &mut Vec<GreenElement>) {
			let _ = for_tuples!((#(Tuple::coalesce(self.Tuple, container)),*));
		}
	}
}
