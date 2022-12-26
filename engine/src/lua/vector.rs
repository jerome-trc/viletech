//! [`mlua::UserData`] implementations for [`glam`]'s vector types.

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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use mlua::prelude::*;

use super::UserDataWrapper;

macro_rules! vec_operator_impl {
	(
		$inner:ty,
		$tyname:tt,
		$opname:tt,
		$func:ident,
		$($field:ident),+
		$(; $castto:ty)?
	) => {
		|_, operands: (Self, LuaValue)| {
			const RHS_ERR_MSG: &'static str = concat!(
				"Right-hand side of `",
				stringify!($tyname),
				"` ",
				stringify!($opname),
				" must be a `",
				stringify!($tyname),
				"` or a number."
			);

			let (lhs, rhs) = operands;

			let ret = match rhs {
				LuaValue::UserData(userdata) => {
					let rhs = userdata.borrow::<Self>()?;

					Self(<$inner>::new(
						$(lhs.$field.$func(rhs.$field),)+
					))
				}
				LuaValue::Number(num) => {
					let num = num $(as $castto)?;

					Self(<$inner>::new(
						$(lhs.$field.$func(num),)+
					))
				}
				other => {
					return Err(LuaError::FromLuaConversionError {
						from: other.type_name(),
						to: stringify!($tyname or number),
						message: Some(RHS_ERR_MSG.to_string()),
					});
				}
			};

			Ok(ret)
		}
	};
}

macro_rules! vec_methods_common {
	() => {
		fn add_methods_common<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
			methods.add_method("abs", |_, this, ()| Ok(Self(this.abs())));

			methods.add_method("abs_diff_eq", |_, this, args: (Self, LuaNumber)| {
				Ok(this.abs_diff_eq(*args.0, args.1))
			});

			methods.add_method("abs_diff_neq", |_, this, args: (Self, LuaNumber)| {
				Ok(!this.abs_diff_eq(*args.0, args.1))
			});

			methods.add_method("approx_eq", |_, this, other: Self| {
				Ok(this.abs_diff_eq(*other, 1.0e-4_f64))
			});

			methods.add_method("approx_neq", |_, this, other: Self| {
				Ok(!this.abs_diff_eq(*other, 1.0e-4_f64))
			});

			methods.add_method("ceil", |_, this, ()| Ok(Self(this.ceil())));

			methods.add_method("clamp", |_, this, args: (Self, Self)| {
				Ok(Self(this.clamp(*args.0, *args.1)))
			});

			methods.add_method("distance", |_, this, other: Self| Ok(this.distance(*other)));

			methods.add_method("distsq", |_, this, other: Self| {
				Ok(this.distance_squared(*other))
			});

			methods.add_method("dot", |_, this, other: Self| Ok(this.dot(*other)));

			methods.add_method("floor", |_, this, ()| Ok(Self(this.floor())));

			methods.add_method("is_normalized", |_, this, ()| Ok(this.is_normalized()));

			methods.add_method("length", |_, this, ()| Ok(this.length()));

			methods.add_method("lenrecip", |_, this, ()| Ok(this.length_recip()));

			methods.add_method("lensq", |_, this, ()| Ok(this.length_squared()));

			methods.add_method("lerp", |_, this, args: (Self, LuaNumber)| {
				Ok(Self(this.lerp(*args.0, args.1)))
			});

			methods.add_method("max", |_, this, other: Self| Ok(Self(this.max(*other))));

			methods.add_method("min", |_, this, other: Self| Ok(Self(this.min(*other))));

			methods.add_method("mul_add", |_, this, args: (Self, Self)| {
				Ok(Self(this.mul_add(*args.0, *args.1)))
			});

			methods.add_method("try_unit", |_, this, ()| Ok(this.try_normalize().map(Self)));

			methods.add_method("unit", |_, this, ()| Ok(Self(this.normalize())));

			methods.add_method("unit_or_zero", |_, this, ()| {
				Ok(Self(this.normalize_or_zero()))
			});
		}
	};
}

impl UserDataWrapper<glam::DVec2> {
	vec_methods_common!();
}

impl LuaUserData for UserDataWrapper<glam::DVec2> {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, this| Ok(this.x));
		fields.add_field_method_get("y", |_, this| Ok(this.y));

		fields.add_field_method_set("x", |_, this, num: LuaNumber| {
			this.x = num;
			Ok(())
		});
		fields.add_field_method_set("y", |_, this, num: LuaNumber| {
			this.y = num;
			Ok(())
		});
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		// Operators ///////////////////////////////////////////////////////////

		methods.add_meta_method(LuaMetaMethod::Unm, |_, this, ()| {
			Ok(Self(glam::DVec2 {
				x: this.x.neg(),
				y: this.y.neg(),
			}))
		});

		methods.add_meta_function(
			LuaMetaMethod::Add,
			vec_operator_impl!(glam::DVec2, dvec2, addition, add, x, y),
		);

		methods.add_meta_function(
			LuaMetaMethod::Div,
			vec_operator_impl!(glam::DVec2, dvec2, division, div, x, y),
		);

		methods.add_meta_function(
			LuaMetaMethod::Sub,
			vec_operator_impl!(glam::DVec2, dvec2, subtraction, sub, x, y),
		);

		methods.add_meta_function(
			LuaMetaMethod::Mul,
			vec_operator_impl!(glam::DVec2, dvec2, multiplication, mul, x, y),
		);

		methods.add_meta_function(
			LuaMetaMethod::Mod,
			vec_operator_impl!(glam::DVec2, dvec2, modulo, rem, x, y),
		);

		methods.add_meta_function(
			LuaMetaMethod::Pow,
			vec_operator_impl!(glam::DVec2, dvec2, exponentiation, powf, x, y),
		);

		methods.add_meta_function(
			LuaMetaMethod::Eq,
			|_, operands: (LuaAnyUserData, LuaAnyUserData)| {
				let (lhs, rhs) = operands;
				let lhs = lhs.borrow::<Self>()?;
				let rhs = rhs.borrow::<Self>()?;
				Ok(lhs.eq(&rhs.0))
			},
		);

		// Methods /////////////////////////////////////////////////////////////

		Self::add_methods_common(methods);

		methods.add_method("angle_between", |_, this, other: Self| {
			Ok(this.angle_between(*other))
		});

		methods.add_method("cross", |_, this, other: Self| Ok(this.perp_dot(*other)));

		methods.add_method("extend", |_, this, z: LuaNumber| {
			Ok(UserDataWrapper(this.extend(z)))
		});

		methods.add_method("perp", |_, this, ()| Ok(Self(this.perp())));

		methods.add_method("rotate", |_, this, other: Self| {
			Ok(Self(this.rotate(*other)))
		});
	}
}

impl UserDataWrapper<glam::DVec3> {
	vec_methods_common!();
}

impl LuaUserData for UserDataWrapper<glam::DVec3> {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, this| Ok(this.x));
		fields.add_field_method_get("y", |_, this| Ok(this.y));
		fields.add_field_method_get("z", |_, this| Ok(this.z));

		fields.add_field_method_set("x", |_, this, num: LuaNumber| {
			this.x = num;
			Ok(())
		});
		fields.add_field_method_set("y", |_, this, num: LuaNumber| {
			this.y = num;
			Ok(())
		});
		fields.add_field_method_set("z", |_, this, num: LuaNumber| {
			this.z = num;
			Ok(())
		});
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		// Operators ///////////////////////////////////////////////////////////

		methods.add_meta_method(LuaMetaMethod::Unm, |_, this, ()| {
			Ok(Self(glam::DVec3 {
				x: this.x.neg(),
				y: this.y.neg(),
				z: this.z.neg(),
			}))
		});

		methods.add_meta_function(
			LuaMetaMethod::Add,
			vec_operator_impl!(glam::DVec3, dvec3, addition, add, x, y, z),
		);

		methods.add_meta_function(
			LuaMetaMethod::Div,
			vec_operator_impl!(glam::DVec3, dvec3, division, div, x, y, z),
		);

		methods.add_meta_function(
			LuaMetaMethod::Sub,
			vec_operator_impl!(glam::DVec3, dvec3, subtraction, sub, x, y, z),
		);

		methods.add_meta_function(
			LuaMetaMethod::Mul,
			vec_operator_impl!(glam::DVec3, dvec3, multiplication, mul, x, y, z),
		);

		methods.add_meta_function(
			LuaMetaMethod::Mod,
			vec_operator_impl!(glam::DVec3, dvec3, modulo, rem, x, y, z),
		);

		methods.add_meta_function(
			LuaMetaMethod::Pow,
			vec_operator_impl!(glam::DVec3, dvec3, exponentiation, powf, x, y, z),
		);

		methods.add_meta_function(
			LuaMetaMethod::Eq,
			|_, operands: (LuaAnyUserData, LuaAnyUserData)| {
				let (lhs, rhs) = operands;
				let lhs = lhs.borrow::<Self>()?;
				let rhs = rhs.borrow::<Self>()?;
				Ok(lhs.eq(&rhs.0))
			},
		);

		// Methods /////////////////////////////////////////////////////////////

		Self::add_methods_common(methods);

		methods.add_method("angle_between", |_, this, other: Self| {
			Ok(this.angle_between(*other))
		});

		methods.add_method("extend", |_, this, z: LuaNumber| {
			Ok(UserDataWrapper(this.extend(z)))
		});
	}
}

impl UserDataWrapper<glam::DVec4> {
	vec_methods_common!();
}

impl LuaUserData for UserDataWrapper<glam::DVec4> {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, this| Ok(this.x));
		fields.add_field_method_get("y", |_, this| Ok(this.y));
		fields.add_field_method_get("z", |_, this| Ok(this.z));
		fields.add_field_method_get("w", |_, this| Ok(this.w));

		fields.add_field_method_set("x", |_, this, num: LuaNumber| {
			this.x = num;
			Ok(())
		});
		fields.add_field_method_set("y", |_, this, num: LuaNumber| {
			this.y = num;
			Ok(())
		});
		fields.add_field_method_set("z", |_, this, num: LuaNumber| {
			this.z = num;
			Ok(())
		});
		fields.add_field_method_set("w", |_, this, num: LuaNumber| {
			this.w = num;
			Ok(())
		});
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		// Operators ///////////////////////////////////////////////////////////

		methods.add_meta_method(LuaMetaMethod::Unm, |_, this, ()| {
			Ok(Self(glam::DVec4 {
				x: this.x.neg(),
				y: this.y.neg(),
				z: this.z.neg(),
				w: this.w.neg(),
			}))
		});

		methods.add_meta_function(
			LuaMetaMethod::Add,
			vec_operator_impl!(glam::DVec4, dvec4, addition, add, x, y, z, w),
		);

		methods.add_meta_function(
			LuaMetaMethod::Div,
			vec_operator_impl!(glam::DVec4, dvec4, division, div, x, y, z, w),
		);

		methods.add_meta_function(
			LuaMetaMethod::Sub,
			vec_operator_impl!(glam::DVec4, dvec4, subtraction, sub, x, y, z, w),
		);

		methods.add_meta_function(
			LuaMetaMethod::Mul,
			vec_operator_impl!(glam::DVec4, dvec4, multiplication, mul, x, y, z, w),
		);

		methods.add_meta_function(
			LuaMetaMethod::Mod,
			vec_operator_impl!(glam::DVec4, dvec4, modulo, rem, x, y, z, w),
		);

		methods.add_meta_function(
			LuaMetaMethod::Pow,
			vec_operator_impl!(glam::DVec4, dvec4, exponentiation, powf, x, y, z, w),
		);

		methods.add_meta_function(
			LuaMetaMethod::Eq,
			|_, operands: (LuaAnyUserData, LuaAnyUserData)| {
				let (lhs, rhs) = operands;
				let lhs = lhs.borrow::<Self>()?;
				let rhs = rhs.borrow::<Self>()?;
				Ok(lhs.eq(&rhs.0))
			},
		);

		// Methods /////////////////////////////////////////////////////////////

		Self::add_methods_common(methods);
	}
}
