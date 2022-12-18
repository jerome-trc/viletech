/*

Copyright (C) 2022 ***REMOVED***

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

use std::sync::Arc;

use parking_lot::RwLock;
use vec1::Vec1;

use super::ast::*;

use crate::utils::lang::{Identifier, Interner, Span};

pub type Error = peg::error::ParseError<<str as peg::Parse>::PositionRepr>;

pub fn parse_statement(input: &str, interner: &Arc<RwLock<Interner>>) -> Result<Statement, Error> {
	parser::statement(input, interner)
}

pub fn parse_expression(
	input: &str,
	interner: &Arc<RwLock<Interner>>,
) -> Result<Expression, Error> {
	parser::expr(input, interner)
}

pub fn parse_module(input: &str, interner: &Arc<RwLock<Interner>>) -> Result<ModuleTree, Error> {
	parser::module_tree(input, interner)
}

peg::parser! {
	grammar parser(interner: &Arc<RwLock<Interner>>) for str {
		use unicode_xid::UnicodeXID;

		// Whitespace, comments ////////////////////////////////////////////////

		rule _ = (" " / "\n" / "\r" / "\t" / line_comment() / block_comment())*

		rule line_comment()
			= "//" ([^ '/' | '!'] / "//") "\n"*
			/ "//"

		rule block_comment() = "/*"

		// Foundational, common ////////////////////////////////////////////////

		rule isolated_cr() -> &'input str = $("\r" [^ '\n'])

		rule digit_dec() -> char = ['0'..='9']
		rule digit_dec_nonzero() -> char = ['1'..='9']
		rule digit_dec_or_underscore() -> char = ['0'..='9' | '_']

		rule digit_bin() -> char = ['0' | '1']
		rule digit_bin_or_underscore() -> char = ['0' | '1' | '_']

		rule digit_hex() -> char = ['0'..='9' | 'a'..='f' | 'A'..='F']
		rule digit_hex_or_underscore() -> char = ['0'..='9' | 'a'..='f' | 'A'..='F' | '_']

		rule digit_oct() -> char = ['0'..='7']
		rule digit_oct_or_underscore() -> char = ['0'..='7' | '_']

		rule ascii_word() -> &'input str
			= $(
				['a'..='z' | 'A'..='Z' | '_']
				['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*
			)

		rule identifier() -> Identifier
			= start:position!() ident:ascii_word() end:position!()
			{
				Identifier {
					span: Span::new(start, end),
					string: Interner::intern(interner, ident),
				}
			}

		rule resolver_part_kind() -> ResolverPartKind
			= 	start:position!()
				string:$(
				['a'..='z' | 'A'..='Z' | '_']
				['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*
				)
				end:position!()
			{
				match string {
					"super" => ResolverPartKind::Super,
					"Self" => ResolverPartKind::SelfUppercase,
					other => ResolverPartKind::Identifier(
						Identifier {
							span: Span::new(start, end),
							string: Interner::intern(interner, other),
						}
					)
				}
			}

		rule resolver_part() -> ResolverPart
			= start:position!() kind:resolver_part_kind() end:position!() {
				ResolverPart {
					span: Span::new(start, end),
					kind,
				}
			}

		rule resolver() -> Resolver
			= 	start:position!()
				"::"?
				parts:(resolver_part() ** "::")
				end:position!()
			{
				Resolver {
					span: Span::new(start, end),
					parts: Vec1::try_from_vec(parts).unwrap(),
				}
			}

		rule annotation() -> Annotation
			= 	start:position!()
				"#" inner:"!"? "[" _ resolver:resolver() _ "]"
				end:position!()
			{
				Annotation {
					span: Span::new(start, end),
					resolver,
					inner: inner.is_some(),
					args: Vec::default(),
				}
			}

		rule block_label() -> BlockLabel
			= start:position!() name:ascii_word() ":" end:position!() {
				BlockLabel {
					span: Span::new(start, end),
					name: name.to_string(),
				}
			}

		rule ident_as_vec() -> Vec<Identifier>
			= name:identifier() { vec![name] }

		/// Specifically for destructuring and range-based for loops.
		rule ident_list() -> Vec<Identifier>
			= "(" _ names:(identifier() ** ",") _ ")" {
				names
			}

		rule type_spec() -> TypeExpr
			= start:position!() ":" _ type_expr:type_expr() end:position!() {
				type_expr
			}

		rule initializer() -> Expression = "=" _ expr:expr() { expr }

		rule block(label_allowed: bool) -> StatementBlock
			=	start:position!()
				label:block_label()?
				_
				"{"
				_
				annotations:annotation()*
				statements:statement()*
				_
				"}"
				end:position!()
			{?
				if label.is_some() && !label_allowed {
					return Err("unlabeled block");
				}

				Ok(
					StatementBlock {
						span: Span::new(start, end),
						label,
						statements,
						annotations,
					}
				)
			}

		// Type ////////////////////////////////////////////////////////////////

		rule type_expr_void() -> TypeExprKind
			= "void" { TypeExprKind::Void }

		rule type_expr_primitive() -> TypeExprKind
			= name:$(
				"i8" / "u8" / "i16" / "u16" /
				"i32" / "u32" / "i64" / "u64" /
				"i128" / "u128" / "bool" / "char"
			) {
				TypeExprKind::Primitive(match name {
					"bool" => PrimitiveTypeKind::Bool,
					"char" => PrimitiveTypeKind::Char,
					"i8" => PrimitiveTypeKind::I8,
					"u8" => PrimitiveTypeKind::U8,
					"i16" => PrimitiveTypeKind::I16,
					"u16" => PrimitiveTypeKind::U16,
					"i32" => PrimitiveTypeKind::I32,
					"u32" => PrimitiveTypeKind::U32,
					"i64" => PrimitiveTypeKind::I64,
					"u64" => PrimitiveTypeKind::U64,
					"i128" => PrimitiveTypeKind::I128,
					"u128" => PrimitiveTypeKind::U128,
					_ => unreachable!()
				})
			}

		rule type_expr_array() -> TypeExprKind
			= "[" _ storage:type_expr() _ ";" _ length:expr() _ "]" {
				TypeExprKind::Array(
					Box::new(
						ArrayTypeExpr {
							storage,
							length,
						}
					)
				)
			}

		rule type_expr_resolver() -> TypeExprKind
			= resolver:resolver() { TypeExprKind::Resolver(resolver) }

		rule type_expr_tuple() -> TypeExprKind
			= "(" _ members:(type_expr() ** ",") ")" {
				TypeExprKind::Tuple { members }
			}

		rule type_expr_inferred() -> TypeExprKind
			= "_" { TypeExprKind::Inferred }

		rule type_expr() -> TypeExpr
			=	start:position!()
				kind:(
					type_expr_void() /
					type_expr_primitive() /
					type_expr_array() /
					type_expr_tuple() /
					type_expr_resolver() /
					type_expr_inferred()
				)
				end:position!()
			{
				TypeExpr {
					span: Span::new(start, end),
					kind,
				}
			}

		// Literals ////////////////////////////////////////////////////////////

		rule lit_null() -> LiteralKind
			= "null" { LiteralKind::Null }

		rule lit_bool() -> LiteralKind
			= lit:$("true" / "false") {
				match lit {
					"true" => LiteralKind::Bool(true),
					"false" => LiteralKind::Bool(false),
					_ => unreachable!()
				}
			}

		rule int_suffix() -> IntType
			= string:$(
				"i8" / "u8" / "i16" / "u16" /
				"i32" / "u32" / "i64" / "u64" /
				"i128" / "u128"
			) {
				match string {
					"i8" => IntType::I8,
					"u8" => IntType::U8,
					"i16" => IntType::I16,
					"u16" => IntType::U16,
					"i32" => IntType::I32,
					"u32" => IntType::U32,
					"i64" => IntType::I64,
					"u64" => IntType::U64,
					"i128" => IntType::I128,
					"u128" => IntType::U128,
					_ => unreachable!(),
				}
			}

		rule float_suffix() -> FloatType
			= string:$("f32" / "f64") {
				match string {
					"f32" => FloatType::F32,
					"f64" => FloatType::F64,
					_ => unreachable!(),
				}
			}

		rule float_exponent() -> &'input str
			= $(
				['e' | 'E']
				['+' | '-']?
				digit_dec_or_underscore()*
				digit_dec()
				digit_dec_or_underscore()*
			)

		rule lit_int() -> LiteralKind
			= 	start:position!()
				value:$(
					lit_decimal() / lit_binary() / lit_octal() / lit_hexadecimal()
				)
				suffix:int_suffix()?
				end:position!()
			{?
				let num = match value.parse::<u128>() {
					Ok(n) => n,
					Err(_) => {
						return Err("decimal, binary, octal, or hexadecimal number");
					}
				};

				Ok(LiteralKind::Int(
					IntLiteral {
						span: Span::new(start, end),
						value: num,
						type_spec: suffix,
					}
				))
			}

		rule lit_float() -> LiteralKind
			= 	start:position!()
				string:$(
					(lit_decimal() "." ![c @ '.' | c @ '_' | c if c.is_xid_start()]) /
					(lit_decimal() float_exponent()) /
					(lit_decimal() "." lit_decimal() float_exponent()?) /
					(lit_decimal() ("." lit_decimal())? float_exponent()? float_suffix())
				)
				end:position!()
			{?
				let type_spec = if string.ends_with("f32") {
					Some(FloatType::F32)
				} else if string.ends_with("f64") {
					Some(FloatType::F64)
				} else {
					None
				};

				let value = string.parse::<f64>().or(Err("floating-point number"))?;

				Ok(
					LiteralKind::Float(
						FloatLiteral {
							span: Span::new(start, end),
							value,
							type_spec,
						}
					)
				)
			}

		rule lit_decimal() -> u128
			= string:$(digit_dec() digit_dec_or_underscore()*) {?
				string.parse::<u128>().or(Err("decimal (base ten) number"))
			}

		rule lit_binary() -> u128
			= string:$(
				"0b"
				digit_bin_or_underscore()*
				digit_bin()
				digit_bin_or_underscore()*
			) {?
				string.parse::<u128>().or(Err("binary number"))
			}

		rule lit_hexadecimal() -> u128
			= string:$(
				"0x"
				digit_hex_or_underscore()*
				digit_hex()
				digit_hex_or_underscore()*
			) {?
				string.parse::<u128>().or(Err("hexadecimal number"))
			}

		rule lit_octal() -> u128
			= string:$(
				"0o"
				digit_oct_or_underscore()*
				digit_oct()
				digit_oct_or_underscore()*
			) {?
				string.parse::<u128>().or(Err("octal number"))
			}

		rule quote_escape() -> &'input str = $("\\'" / "\\\"")

		rule ascii_escape() -> &'input str
			= $(
				("\\x" digit_oct() digit_hex()) /
				"\\n" / "\\r" / "\\t" / "\\\\" / "\\0"
			)

		rule lit_char() -> LiteralKind
			=	start:position!()
				"'"
				character:[
					^ '\'' | '\\' | '\n' | '\r' | '\t'
				]
				"'"
				end:position!()
			{
				LiteralKind::Char(
					CharLiteral {
						span: Span::new(start, end),
						character,
					}
				)
			}

		rule lit_string() -> LiteralKind
			=	start:position!()
				"\""
				inner:$(!isolated_cr() [^ '\"' | '\\']+)
				"\""
				end:position!()
			{
				LiteralKind::String(
					StringLiteral {
						span: Span::new(start, end),
						string: Interner::intern(interner, inner),
					}
				)
			}

		rule literal() -> Literal
			= 	start:position!()
				kind:(
					lit_null() / lit_bool() /
					lit_int() / lit_float() /
					lit_char() / lit_string()
				)
				end:position!()
			{
				Literal {
					span: Span::new(start, end),
					kind,
				}
			}

		// Expressions /////////////////////////////////////////////////////////

		#[cache_left_rec]
		rule op_ternary() -> TernaryOpExprs
			= cond:expr() _ "?" _ if_true:expr() _ ":" _ if_false:expr()
			{
				TernaryOpExprs {
					cond,
					if_true,
					if_false
				}
			}

		#[cache_left_rec]
		pub(crate) rule expr() -> Expression = precedence! {
			start:position!() expr:@ end:position!() {
				Expression {
					span: Span::new(start, end),
					kind: expr,
				}
			}
			--
			ternary:op_ternary() {
				ExpressionKind::Ternary(Box::new(ternary))
			}
			-- // Binary, general/miscellaneous
			lhs:@ _ "=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::Assign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs })
				}
			}
			lhs:(@) "." field:identifier() {
				ExpressionKind::Field(
					Box::new(
						FieldExpr {
							owner: lhs,
							field,
						}
					)
				)
			}
			lhs:(@) "(" args:call_args() ")" {
				ExpressionKind::Call {
					lhs: Box::new(lhs),
					args,
				}
			}
			lhs:(@) "." method:resolver_part() "(" args:call_args() ")" {
				ExpressionKind::MethodCall {
					lhs: Box::new(lhs),
					method,
					args,
				}
			}
			lhs:@ _ ".." _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::Concat,
					exprs: Box::new(BinaryOpExprs { lhs, rhs })
				}
			}
			lhs:@ _ "..=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::ConcatAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "is" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::TypeCompare,
					exprs: Box::new(BinaryOpExprs { lhs, rhs })
				}
			}
			lhs:@ _ "!is" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::NegativeTypeCompare,
					exprs: Box::new(BinaryOpExprs { lhs, rhs })
				}
			}
			lhs:@ _ "::" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::ScopeRes,
					exprs: Box::new(BinaryOpExprs { lhs, rhs })
				}
			}
			// Binary, arithmetic, non-compound
			lhs:(@) _ "+" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::Add,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "-" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::Subtract,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "*" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::Multiply,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "/" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::Divide,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "**" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::Raise,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "%" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::Modulo,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			// Binary, arithmetic, compound assignment
			lhs:@ _ "+=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::AddAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "-=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::SubtractAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "*=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::MultiplyAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "/=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::DivideAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "**=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::RaiseAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "%=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::ModuloAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			// Binary, bitwise, non-compound
			lhs:(@) _ "<<" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::LeftShift,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ ">>" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::RightShift,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ ">>>" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::UnsignedRightShift,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "&" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::BitwiseAnd,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "|" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::BitwiseOr,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "^" _ rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::BitwiseXor,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			// Binary, bitwise, compound assignment
			lhs:@ _ "<<=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::LeftShiftAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ ">>=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::RightShiftAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ ">>>=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::UnsignedRightShiftAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "&=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::BitwiseAndAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "|=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::BitwiseOrAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "^=" _ rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::BitwiseXorAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			// Binary, logical, non-compound
			lhs:(@) _ "&&" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::LogicalAnd,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "||" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::LogicalOr,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "^^" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::LogicalXor,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			// Binary, logical, compound assignment
			lhs:@ _ "&&=" rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::LogicalAndAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "||=" rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::LogicalOrAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:@ _ "^^=" rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::LogicalXorAssign,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			// Binary, comparison
			lhs:@ _ "==" rhs:(@) {
				ExpressionKind::Binary {
					op: BinaryOp::Equals,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "!=" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::NotEquals,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "~==" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::ApproxEquals,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "<" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::LessThan,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "<=" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::LessThanOrEquals,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ ">" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::GreaterThan,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ ">=" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::GreaterThanOrEquals,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			lhs:(@) _ "<>=" rhs:@ {
				ExpressionKind::Binary {
					op: BinaryOp::ThreeWayComp,
					exprs: Box::new(BinaryOpExprs { lhs, rhs, })
				}
			}
			-- // Unary
			"+" operand:@ {
				ExpressionKind::Prefix(Box::new(PrefixOpExpr {
					op: PrefixOp::AntiNegate,
					expr: operand,
				}))
			}
			"-" operand:@ {
				ExpressionKind::Prefix(Box::new(PrefixOpExpr {
					op: PrefixOp::Negate,
					expr: operand,
				}))
			}
			"++" operand:@ {
				ExpressionKind::Prefix(Box::new(PrefixOpExpr {
					op: PrefixOp::Increment,
					expr: operand,
				}))
			}
			"--" operand:@ {
				ExpressionKind::Prefix(Box::new(PrefixOpExpr {
					op: PrefixOp::Decrement,
					expr: operand,
				}))
			}
			"!" operand:@ {
				ExpressionKind::Prefix(Box::new(PrefixOpExpr {
					op: PrefixOp::LogicalNot,
					expr: operand,
				}))
			}
			"~" operand:@ {
				ExpressionKind::Prefix(Box::new(PrefixOpExpr {
					op: PrefixOp::BitwiseNot,
					expr: operand,
				}))
			}
			operand:@ "++" {
				ExpressionKind::Postfix(Box::new(PostfixOpExpr {
					op: PostfixOp::Increment,
					expr: operand,
				}))
			}
			operand:@ "--" {
				ExpressionKind::Postfix(Box::new(PostfixOpExpr {
					op: PostfixOp::Decrement,
					expr: operand,
				}))
			}
			-- // Atoms
			lit:literal() {
				ExpressionKind::Literal(lit)
			}
			ident:identifier() {
				ExpressionKind::Identifier(ident)
			}
		}

		rule arg_anon() -> CallArg
			= start:position!() expr:expr() end:position!() {
				CallArg {
					span: Span::new(start, end),
					kind: CallArgKind::Unnamed(expr),
				}
			}

		rule arg_named() -> CallArg
			=	start:position!()
				name:identifier() _ ":" _
				expr:expr()
				end:position!()
			{
				CallArg {
					span: Span::new(start, end),
					kind: CallArgKind::Named {
						name,
						expr,
					}
				}
			}

		rule call_arg() -> CallArg = arg_named() / arg_anon()
		rule call_args() -> Vec<CallArg> = call_arg() ** ","

		// Statements //////////////////////////////////////////////////////////

		rule stat_empty() -> StatementKind = ";" { StatementKind::Empty }

		rule stat_break() -> StatementKind
			= "break" _ tgt:ascii_word()? _ ";" {
				StatementKind::Break {
					target: tgt.map(|s| s.to_owned())
				}
			}

		rule stat_continue() -> StatementKind
			= "continue" _ tgt:ascii_word()? _ ";" {
				StatementKind::Continue {
					target: tgt.map(|s| s.to_owned())
				}
			}

		rule stat_expr() -> StatementKind
			= expr:expr() _ ";" {
				StatementKind::Expression(expr)
			}

		rule stat_block() -> StatementKind
			= block:block(true) { StatementKind::Block(block) }

		rule stat_item() -> StatementKind
			= item:item() { StatementKind::Item(item) }

		rule stat_binding_single() -> StatementKind
			=	start:position!()
				annotations:annotation()* _
				"let" _
				name:identifier() _
				type_spec:type_spec()? _
				"=" _
				init:expr()? _
				";"
				end:position!()
			{
				StatementKind::Binding(
					Binding {
						span: Span::new(start, end),
						names: vec![name],
						type_spec,
						init,
						annotations,
					}
				)
			}

		rule stat_binding_multi() -> StatementKind
			=	start:position!()
				annotations:annotation()* _
				"let" _
				names:(identifier() ** ",") _
				type_spec:type_spec()? _
				"=" _
				init:expr()? _
				";"
				end:position!()
			{
				StatementKind::Binding(
					Binding {
						span: Span::new(start, end),
						names,
						type_spec,
						init,
						annotations,
					}
				)
			}

		rule stat_if() -> StatementKind
			= 	annotations:annotation()*
				 "if" _ "(" _ cond:expr() _ ")" _ body:block(true) _
				else_body:else_body()?
			 {
				StatementKind::If {
					cond,
					body,
					else_body: else_body.map(Box::new),
					annotations,
				}
			}

		rule else_body() -> Statement
			=	start:position!()
				"else" _ else_body:(stat_if() / stat_block())
				end:position!()
			{
				Statement {
					span: Span::new(start, end),
					kind: else_body,
				}
			}

		rule switch_case_specific() -> SwitchCase
			=	start:position!()
				"case" _ expr:expr() _ ":" _ block:block(true)
				end:position!()
			{
				SwitchCase {
					span: Span::new(start, end),
					kind: SwitchCaseKind::Specific(expr),
					block,
				}
			}

		rule switch_case_default() -> SwitchCase
			=	start:position!()
				"default" _ ":" _ block:block(true)
				end:position!()
			{
				SwitchCase {
					span: Span::new(start, end),
					kind: SwitchCaseKind::Default,
					block,
				}
			}

		rule stat_switch() -> StatementKind
			=	annotations:annotation()*
				label:block_label()? _
				"switch" _ "(" _ val:expr() _ ")" _
				"{" _
				cases:(switch_case_specific() / switch_case_default())* _
				"}"
			{
				StatementKind::Switch {
					val,
					label,
					cases,
					annotations,
				}
			}

		rule stat_loop_infinite() -> StatementKind
			=	annotations:annotation()*
				label:block_label()? _
				"loop" _ body:block(false)
			{
				StatementKind::Loop {
					kind: LoopKind::Infinite,
					body: Box::new(body),
					annotations,
				}
			}

		rule stat_loop_range() -> StatementKind
			=	annotations:annotation()*
				label:block_label()? _
				"for" _ names:(ident_as_vec() / ident_list()) _
				"in" _ sequence:expr() _ body:block(false)
			{
				StatementKind::Loop {
					kind: LoopKind::Range {
						bindings: names,
						sequence,
					},
					body: Box::new(body),
					annotations,
				}
			}

		rule stat_loop_while() -> StatementKind
			=	annotations:annotation()*
				label:block_label()? _
				"while" _ "(" _ cond:expr() _ ")" _ body:block(false)
			{
				StatementKind::Loop {
					kind: LoopKind::While { condition: cond, },
					body: Box::new(body),
					annotations,
				}
			}

		rule stat_loop_dowhile() -> StatementKind
			=	annotations:annotation()*
				label:block_label()? _
				"do" _ body:block(false) _ "while" _ "(" _ cond:expr() _ ")" _ ";"
			{
				StatementKind::Loop {
					kind: LoopKind::DoWhile { condition: cond, },
					body: Box::new(body),
					annotations,
				}
			}

		rule stat_loop_dountil() -> StatementKind
			=	annotations:annotation()*
				label:block_label()? _
				"do" _ body:block(false) _ "until" _ "(" _ cond:expr() _ ")" _ ";"
			{
				StatementKind::Loop {
					kind: LoopKind::DoUntil { condition: cond, },
					body: Box::new(body),
					annotations,
				}
			}

		pub(crate) rule statement() -> Statement
			= 	start:position!()
				kind:(
					stat_empty() / stat_break() / stat_continue() /
					stat_expr() / stat_block() / stat_item() /
					stat_binding_single() / stat_binding_multi() /
					stat_if() / stat_switch() /
					stat_loop_infinite() / stat_loop_range() /
					stat_loop_while() / stat_loop_dowhile() / stat_loop_dountil()
				)
				end:position!()
			{
				Statement {
					span: Span::new(start, end),
					kind,
				}
			}

		// Item ////////////////////////////////////////////////////////////////

		rule decl_qual() -> DeclQualifier
			= 	start:position!()
				keyword:$(
					"abstract" / "virtual" / "override" / "final" /
					"ceval" /
					"private" / "protected" / "public" /
					"static"
				)
				end:position!()
			{
				DeclQualifier {
					span: Span::new(start, end),
					kind: match keyword {
						"abstract" => DeclQualifierKind::Abstract,
						"virtual" => DeclQualifierKind::Virtual,
						"override" => DeclQualifierKind::Override,
						"final" => DeclQualifierKind::Final,
						"ceval" => DeclQualifierKind::CEval,
						"private" => DeclQualifierKind::Private,
						"protected" => DeclQualifierKind::Protected,
						"public" => DeclQualifierKind::Public,
						"static" => DeclQualifierKind::Static,
						_ => unreachable!()
					}
				}
			}

		rule type_alias() -> ItemKind
			= 	start:position!()
				quals:(decl_qual() ** _)
				_
				"type"
				_
				name:identifier()
				_
				"="
				_
				underlying:type_expr()
				end:position!()
			{
				ItemKind::TypeAlias(
					TypeAlias {
						span: Span::new(start, end),
						name,
						quals,
						underlying
					}
				)
			}

		rule constant() -> ItemKind
			=	start:position!()
				quals:(decl_qual() ** _)
				_
				"const"
				_
				name:identifier()
				type_spec:type_spec()?
				_
				"="
				_
				value:expr()
				end:position!()
			{
				ItemKind::Constant(
					Constant {
						span: Span::new(start, end),
						name,
						quals,
						type_spec,
						value,
					}
				)
			}

		rule macro_invoc() -> ItemKind
			=	start:position!()
				resolver:resolver()
				"!"
				_
				delim_l:['(' | '[' | '{']
				_
				inner:$([_]*)
				_
				delim_r:['(' | '[' | '{']
				end:position!()
			{?
				if delim_r != delim_l {
					return Err("");
				};

				Ok(ItemKind::MacroInvoc(
					MacroInvocation {
						span: Span::new(start, end),
						inner: inner.to_owned(),
					}
				))
			}

		rule enum_variant() -> EnumVariant
			=	start:position!()
				annotations:annotation()*
				name:identifier() _
				init:initializer()?
				end:position!()
			{
				EnumVariant {
					span: Span::new(start, end),
					name,
					init,
					annotations,
				}
			}

		rule enum_decl() -> ItemKind
			=	start:position!()
				quals:(decl_qual() ** _) _
				"enum" _
				name:identifier() _
				type_spec:type_spec()? _
				"{" _
				variants:enum_variant()* _
				"}"
				end:position!()
			{
				ItemKind::Enum(
					EnumDef {
						span: Span::new(start, end),
						name,
						quals,
						type_spec,
						variants,
					}
				)
			}

		rule func_param_qual() -> FuncParamQualifier
			=	start:position!()
				string:$("in" / "out")
				end:position!()
			{
				let kind = match string {
					"in" => FuncParamQualKind::In,
					"out" => FuncParamQualKind::Out,
					_ => unreachable!(),
				};

				FuncParamQualifier {
					span: Span::new(start, end),
					kind,
				}
			}

		rule func_param() -> FuncParameter
			= 	start:position!()
				annotations:annotation()*
				quals:(func_param_qual() ** _) _
				name:identifier() _
				type_spec:type_spec() _
				default:initializer()?
				end:position!()
			{
				FuncParameter {
					span: Span::new(start, end),
					name,
					quals,
					type_spec,
					default,
					annotations,
				}
			}

		rule function_decl() -> ItemKind
			=	start:position!()
				return_type:type_expr() _
				quals:(decl_qual() ** _) _
				name:identifier() _
				"(" _ params:(func_param() ** ",") _ ")" _
				body:block(false)? _
				term:";"?
				end:position!()
			{?
				if body.is_none() && term.is_none() {
					return Err("a function body or trailing semicolon");
				}

				Ok(
					ItemKind::Function(
						FunctionDeclaration {
							span: Span::new(start, end),
							name,
							quals,
							return_type,
							params,
							body,
						}
					)
				)
			}

		rule item() -> Item
			=	start:position!()
				annotations:annotation()* _
				kind:(
					type_alias() / constant() /
					enum_decl() / function_decl() /
					macro_invoc()
				)
				end:position!()
			{
				Item {
					span: Span::new(start, end),
					kind,
					annotations,
				}
			}

		rule top_level_item() -> TopLevel
			= item:item() {
				TopLevel::Item(item)
			}

		rule top_level_annotation() -> TopLevel
			= annotation:annotation() {
				TopLevel::Annotation(annotation)
			}

		pub(crate) rule module_tree() -> ModuleTree
			= tops:top_level_item()* ![_] {
				ModuleTree {
					top_level: tops,
				}
			}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn hello_world() {
		const SOURCE: &str = "print(fmt: \"hello world!\");";
		let interner = Interner::new_arc();
		println!("{:#?}", super::parse_statement(SOURCE, &interner));
	}
}
