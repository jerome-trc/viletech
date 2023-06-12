//! Utilities for making it easier succinctly build [`rowan`] green trees.

use rowan::{GreenNode, GreenToken, SyntaxKind};
use smallvec::SmallVec;

use crate::GreenElement;

#[must_use]
pub fn coalesce_node<C: Coalesce>(input: C, syn: impl Into<SyntaxKind>) -> GreenNode {
	let mut elems = vec![];
	input.coalesce(&mut elems);
	GreenNode::new(syn.into(), elems)
}

#[must_use]
pub fn coalesce_vec<C: Coalesce>(input: C) -> Vec<GreenElement> {
	let mut ret = vec![];
	input.coalesce(&mut ret);
	ret
}

pub trait Coalesce: 'static {
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

impl<T, const C: usize> Coalesce for SmallVec<[T; C]>
where
	T: Coalesce,
{
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		for input in self {
			input.coalesce(container);
		}
	}
}

#[impl_trait_for_tuples::impl_for_tuples(1, 16)]
impl Coalesce for Tuple {
	fn coalesce(self, container: &mut Vec<GreenElement>) {
		let _ = for_tuples!((#(Tuple::coalesce(self.Tuple, container)),*));
	}
}
