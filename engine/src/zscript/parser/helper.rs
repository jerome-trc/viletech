/*

Copyright (C) 2021-2022 Jessica "Gutawer" Russell

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use super::{ir::*, tokenizer::*};

pub(super) fn get_prefix_op(t: &Option<Token>) -> Option<PrefixOp> {
	if t.is_none() {
		return None;
	}
	let t = t.as_ref().unwrap();

	match t {
		Token {
			data: TokenData::Punctuation(p),
			..
		} => match p {
			Punctuation::Plus => Some(PrefixOp::Plus),
			Punctuation::Minus => Some(PrefixOp::Minus),
			Punctuation::Increment => Some(PrefixOp::Increment),
			Punctuation::Decrement => Some(PrefixOp::Decrement),
			Punctuation::LogicalNot => Some(PrefixOp::LogicalNot),
			Punctuation::BitwiseNot => Some(PrefixOp::BitwiseNot),
			_ => None,
		},
		Token {
			data: TokenData::Keyword(k),
			..
		} => match k {
			Keyword::SizeOf => Some(PrefixOp::SizeOf),
			Keyword::AlignOf => Some(PrefixOp::AlignOf),
			_ => None,
		},
		_ => None,
	}
}

pub(super) fn get_postfix_op(t: &Option<Token>) -> Option<PostfixOp> {
	if t.is_none() {
		return None;
	}
	let t = t.as_ref().unwrap();

	if let Token {
		data: TokenData::Punctuation(p),
		..
	} = t
	{
		match p {
			Punctuation::Increment => Some(PostfixOp::Increment),
			Punctuation::Decrement => Some(PostfixOp::Decrement),
			_ => None,
		}
	} else {
		None
	}
}

#[derive(Debug, Clone, Copy)]
pub(super) enum InfixOp {
	Binary(BinaryOp),
	LeftRound,
	LeftSquare,
	Ternary,
}

pub(super) fn get_infix_op(t: &Option<Token>) -> Option<InfixOp> {
	if t.is_none() {
		return None;
	}
	let t = t.as_ref().unwrap();

	match t {
		Token {
			data: TokenData::Punctuation(p),
			..
		} => match p {
			Punctuation::Plus => Some(InfixOp::Binary(BinaryOp::Add)),
			Punctuation::Minus => Some(InfixOp::Binary(BinaryOp::Subtract)),
			Punctuation::Times => Some(InfixOp::Binary(BinaryOp::Times)),
			Punctuation::Divide => Some(InfixOp::Binary(BinaryOp::Divide)),
			Punctuation::Modulo => Some(InfixOp::Binary(BinaryOp::Modulo)),
			Punctuation::Raise => Some(InfixOp::Binary(BinaryOp::Raise)),

			Punctuation::LeftShift => Some(InfixOp::Binary(BinaryOp::LeftShift)),
			Punctuation::RightShift => Some(InfixOp::Binary(BinaryOp::RightShift)),
			Punctuation::UnsignedRightShift => Some(InfixOp::Binary(BinaryOp::UnsignedRightShift)),

			Punctuation::DotDot => Some(InfixOp::Binary(BinaryOp::Concat)),

			Punctuation::LeftAngle => Some(InfixOp::Binary(BinaryOp::LessThan)),
			Punctuation::LessThanEquals => Some(InfixOp::Binary(BinaryOp::LessThanEquals)),
			Punctuation::RightAngle => Some(InfixOp::Binary(BinaryOp::GreaterThan)),
			Punctuation::GreaterThanEquals => Some(InfixOp::Binary(BinaryOp::GreaterThanEquals)),
			Punctuation::Equals => Some(InfixOp::Binary(BinaryOp::Equals)),
			Punctuation::NotEquals => Some(InfixOp::Binary(BinaryOp::NotEquals)),
			Punctuation::ApproxEquals => Some(InfixOp::Binary(BinaryOp::ApproxEquals)),
			Punctuation::ThreeWayComp => Some(InfixOp::Binary(BinaryOp::ThreeWayComp)),

			Punctuation::LogicalAnd => Some(InfixOp::Binary(BinaryOp::LogicalAnd)),
			Punctuation::BitwiseAnd => Some(InfixOp::Binary(BinaryOp::BitwiseAnd)),
			Punctuation::LogicalOr => Some(InfixOp::Binary(BinaryOp::LogicalOr)),
			Punctuation::BitwiseOr => Some(InfixOp::Binary(BinaryOp::BitwiseOr)),
			Punctuation::BitwiseXor => Some(InfixOp::Binary(BinaryOp::BitwiseXor)),

			Punctuation::DoubleColon => Some(InfixOp::Binary(BinaryOp::Scope)),
			Punctuation::Dot => Some(InfixOp::Binary(BinaryOp::MemberAccess)),

			Punctuation::Assign => Some(InfixOp::Binary(BinaryOp::Assign)),
			Punctuation::PlusAssign => Some(InfixOp::Binary(BinaryOp::PlusAssign)),
			Punctuation::MinusAssign => Some(InfixOp::Binary(BinaryOp::MinusAssign)),
			Punctuation::TimesAssign => Some(InfixOp::Binary(BinaryOp::TimesAssign)),
			Punctuation::DivideAssign => Some(InfixOp::Binary(BinaryOp::DivideAssign)),
			Punctuation::ModuloAssign => Some(InfixOp::Binary(BinaryOp::ModuloAssign)),
			Punctuation::LeftShiftAssign => Some(InfixOp::Binary(BinaryOp::LeftShiftAssign)),
			Punctuation::RightShiftAssign => Some(InfixOp::Binary(BinaryOp::RightShiftAssign)),
			Punctuation::UnsignedRightShiftAssign => {
				Some(InfixOp::Binary(BinaryOp::UnsignedRightShiftAssign))
			}
			Punctuation::BitwiseOrAssign => Some(InfixOp::Binary(BinaryOp::BitwiseOrAssign)),
			Punctuation::BitwiseAndAssign => Some(InfixOp::Binary(BinaryOp::BitwiseAndAssign)),
			Punctuation::BitwiseXorAssign => Some(InfixOp::Binary(BinaryOp::BitwiseXorAssign)),

			Punctuation::QuestionMark => Some(InfixOp::Ternary),
			Punctuation::LeftSquare => Some(InfixOp::LeftSquare),
			Punctuation::LeftRound => Some(InfixOp::LeftRound),

			_ => None,
		},
		Token {
			data: TokenData::Keyword(k),
			..
		} => match k {
			Keyword::Dot => Some(InfixOp::Binary(BinaryOp::DotProd)),
			Keyword::Cross => Some(InfixOp::Binary(BinaryOp::CrossProd)),
			Keyword::Is => Some(InfixOp::Binary(BinaryOp::Is)),
			_ => None,
		},
		_ => None,
	}
}

pub(super) fn get_prefix_precedence(_: PrefixOp) -> ((), usize) {
	((), 27)
}

pub(super) fn get_postfix_precedence(_: PostfixOp) -> (usize, ()) {
	(27, ())
}

pub(super) fn get_infix_precedence(b: InfixOp) -> (usize, usize) {
	use BinaryOp::*;
	match b {
		InfixOp::Ternary => (3, 2),

		InfixOp::LeftRound => (30, 31),
		InfixOp::LeftSquare => (30, 31),

		InfixOp::Binary(b) => match b {
			Assign => (1, 0),
			PlusAssign => (1, 0),
			MinusAssign => (1, 0),
			TimesAssign => (1, 0),
			DivideAssign => (1, 0),
			ModuloAssign => (1, 0),
			LeftShiftAssign => (1, 0),
			RightShiftAssign => (1, 0),
			UnsignedRightShiftAssign => (1, 0),
			BitwiseOrAssign => (1, 0),
			BitwiseAndAssign => (1, 0),
			BitwiseXorAssign => (1, 0),

			LogicalOr => (4, 5),

			LogicalAnd => (6, 7),

			Equals => (8, 9),
			NotEquals => (8, 9),
			ApproxEquals => (8, 9),

			LessThan => (10, 11),
			LessThanEquals => (10, 11),
			GreaterThan => (10, 11),
			GreaterThanEquals => (10, 11),
			ThreeWayComp => (10, 11),
			Is => (10, 11),

			Concat => (12, 13),

			BitwiseOr => (14, 15),

			BitwiseXor => (16, 17),

			BitwiseAnd => (18, 19),

			LeftShift => (20, 21),
			RightShift => (20, 21),
			UnsignedRightShift => (20, 21),

			Add => (22, 23),
			Subtract => (22, 23),

			Times => (24, 25),
			Divide => (24, 25),
			Modulo => (24, 25),
			CrossProd => (24, 25),
			DotProd => (24, 25),

			Raise => (26, 27),

			MemberAccess => (30, 31),

			Scope => (32, 33),
		},
	}
}
