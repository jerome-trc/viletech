--- @meta

--- @class floatvec: userdata
local floatvec = {
	--- Returns a vector with the absolute values of `self`.
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	abs = function(self) end,

	--- Returns true if the absolute difference of all elements between
	--- `self` and `other` is less than or equal to `diff`.
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @param diff number
	--- @return boolean
	--- @nodiscard
	abs_diff_eq = function(self, other, diff) end,

	--- Shorthand for `not abs_diff_neq(self, other, diff)`.
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @param diff number
	--- @return boolean
	--- @nodiscard
	abs_diff_neq = function(self, other, diff) end,

	--- Shorthand for `abs_diff_eq(self, other, vector.APPROX)`.
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @return boolean
	--- @nodiscard
	approx_eq = function(self, other) end,

	--- Shorthand for `not approx_eq(self, other)`.
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @return boolean
	--- @nodiscard
	approx_neq = function(self, other) end,

	--- Returns a vector containing the smallest integer greater than or equal
	--- to a number for each element of `self`.
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	ceil = function(self) end,

	--- Component-wise clamping of values.
	--- @generic T: floatvec
	--- @param self T
	--- @param min T
	--- @param max T
	--- @return T
	--- @nodiscard
	clamp = function(self, min, max) end,

	--- Euclidean distance between two points.
	--- @see floatvec.distsq
	--- @generic T
	--- @param self T
	--- @param other T
	--- @return number
	--- @nodiscard
	distance = function(self, other) end,

	--- Squared Euclidean distance.
	--- Faster than `distance`, since a square root operation isn't required.
	--- @see floatvec.distance
	--- @generic T
	--- @param self T
	--- @param other T
	--- @return number
	--- @nodiscard
	distsq = function(self, other) end,

	--- Computes the dot product of `self` and `other`.
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @return T
	--- @nodiscard
	dot = function(self, other) end,

	--- Returns a vector containing the largest integer less than or equal
	--- to a number for each element of `self`.
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	floor = function(self) end,

	--- Returns whether `self` is approximately length 1.0, using
	--- `vector.APPROX` as a precision threshold.
	--- @generic T: floatvec
	--- @param self T
	--- @return boolean
	--- @nodiscard
	is_normalized = function(self) end,

	--- Computes the square root of the dot product of `self` with `self`.
	--- @see floatvec.lenrecip
	--- @see floatvec.lensq
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	length = function(self) end,

	--- "Reciprocal length". Shorthand for 1.0 divided by the vector's length.
	--- The return value will be infinite if `self`'s length is zero.
	--- @see floatvec.length
	--- @see floatvec.lensq
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	lenrecip = function(self) end,

	--- "Length squared"; computes the dot product of `self` with `self`.
	--- Faster than `length` since no square root is required. Use wherever possible.
	--- @see floatvec.length
	--- @see floatvec.lenrecip
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	lensq = function(self) end,

	--- Performs a linear interpolation between `self` and `other` based on `t`.
	--- When `t` is 0.0, the result will be equal to `self`.
	--- When `t` is 1.0, the result will be equal to `other`.
	--- When `t` is outside of range [0.0, 1.0], the result is linearly extrapolated.
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @param t number
	--- @return T
	--- @nodiscard
	lerp = function(self, other, t) end,

	--- Returns a vector containing the largest values for each element of `self`
	--- and `other`. (e.g. `math.max(self.x, other.x)`, et cetera).
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @return T
	--- @nodiscard
	max = function(self, other) end,

	--- Returns a vector containing the smallest values for each element of `self`
	--- and `other`. (e.g. `math.min(self.x, other.x)`, et cetera).
	--- @generic T: floatvec
	--- @param self T
	--- @param other T
	--- @return T
	--- @nodiscard
	min = function(self, other) end,

	--- Fused multiply-add. Computes `(self * a) + b` element-wise with only one
	--- rounding error, yielding a more accurate result than an unfused multiply-add.
	--- May also be faster, depending on the underlying CPU and platform.
	--- @generic T: floatvec
	--- @param self T
	--- @param a T
	--- @param b T
	--- @return T
	--- @nodiscard
	mul_add = function(self, a, b) end,

	--- Returns a vector containing each element of `self` raised to a power.
	--- @generic T: floatvec
	--- @param self T
	--- @param power number
	--- @return T
	--- @nodiscard
	powf = function(self, power) end,

	--- Like `unit` but returns nil if `self`'s length is zero or near-zero.
	--- @see floatvec.unit
	--- @see floatvec.unit_or_zero
	--- @generic T: floatvec
	--- @param self T
	--- @return T?
	--- @nodiscard
	try_unit = function(self) end,

	--- Also known as "normalizing". Shorthand for `self * self.lenrecip`.
	--- @see floatvec.try_unit
	--- @see floatvec.unit_or_zero
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	unit = function(self) end,

	--- Like `unit` but returns a zeroed vector if `self`'s length is zero or near-zero.
	--- @see floatvec.unit
	--- @see floatvec.try_unit
	--- @generic T: floatvec
	--- @param self T
	--- @return T
	--- @nodiscard
	unit_or_zero = function(self) end,
}

--- 2-dimensional vector of 64-bit floating point numbers.
--- @class dvec2: floatvec
--- @field x number
--- @field y number
--- @operator unm:dvec2
--- @operator add(dvec2|number):dvec2
--- @operator sub(dvec2|number):dvec2
--- @operator mul(dvec2|number):dvec2
--- @operator div(dvec2|number):dvec2
--- @operator mod(dvec2|number):dvec2
--- @operator pow(dvec2|number):dvec2
local dvec2_t = {
	--- The angle in radians between `self` and `other`.
	--- @param self dvec2
	--- @param other dvec2
	--- @return number
	--- @nodiscard
	angle_between = function(self, other) end,

	--- The perpendicular dot product of `self` and `other`.
	--- Also known as the wedge product or determinant.
	--- @param self dvec2
	--- @param other dvec2
	--- @return number
	--- @nodiscard
	cross = function(self, other) end,

	--- Creates a 3D vector from `self` and the given `z` value.
	--- @param self dvec2
	--- @param z number
	--- @return dvec3
	--- @nodiscard
	extend = function(self, z) end,

	--- Returns a vector that is equal to `self` rotated by 90 degrees.
	--- @param self dvec2
	--- @return dvec2
	--- @nodiscard
	perp = function(self) end,

	--- Returns `other` rotated by the angle of `self`. If `self` isn't normalized,
	--- expect the result to be multiplied by `self`'s length, which may not be desired.
	--- @param self dvec2
	--- @param other dvec2
	--- @return dvec2
	--- @nodiscard
	rotate = function(self, other) end,
}

--- 3-dimensional vector of 64-bit floating point numbers.
--- @class dvec3: floatvec
--- @field x number
--- @field y number
--- @field z number
--- @operator unm:dvec3
--- @operator add(dvec3|number):dvec3
--- @operator sub(dvec3|number):dvec3
--- @operator mul(dvec3|number):dvec3
--- @operator div(dvec3|number):dvec3
--- @operator mod(dvec3|number):dvec3
--- @operator pow(dvec3|number):dvec3
local dvec3_t = {}

--- 4-dimensional vector of 64-bit floating point numbers.
--- @class dvec4: floatvec
--- @field x number
--- @field y number
--- @field z number
--- @field w number
--- @operator unm:dvec4
--- @operator add(dvec4|number):dvec4
--- @operator sub(dvec4|number):dvec4
--- @operator mul(dvec4|number):dvec4
--- @operator div(dvec4|number):dvec4
--- @operator mod(dvec4|number):dvec4
--- @operator pow(dvec4|number):dvec4
local dvec4_t = {}

--- Constants for geometric vectors that don't belong in a more specific library.
--- @class vectorlib
vector = {
	--- Precision threshold for approximate vector comparisons.
	--- @see floatvec.approx_eq
	--- @see floatvec.approx_neq
	--- @see floatvec.is_normalized
	APPROX = 1.0e-4,

	--- Precision threshold used by (G)ZDoom for its vector comparisons.
	--- Comes from 1.0 / 65536.0.
	APPROX_ZS = 1.5285e-5,
}

--- Static functions and constants for 2-dimensional 64-bit floating point vectors.
--- @class dvec2lib
dvec2 = {
	--- @param x number
	--- @param y number
	--- @return dvec2
	--- @nodiscard
	new = function(x, y) end,

	--- Return a `vec2` with both elements set to `scalar`.
	--- @param scalar number
	--- @return dvec2
	--- @nodiscard
	splat = function(scalar) end,

	--- Creates a `dvec2` with `x` set to `math.cos(angle)` and `y` set to `math.sin(angle)`.
	--- @param angle number
	--- @return dvec2
	--- @nodiscard
	from_angle = function(angle) end,

	--- Returns a `dvec2` with both elements set to 0.0.
	--- @return dvec2
	--- @nodiscard
	zero = function() end,

	--- Returns a `dvec2` with both elements set to 1.0.
	--- @return dvec2
	--- @nodiscard
	one = function() end,

	--- Returns a `dvec2` with both elements set to -1.0.
	--- @return dvec2
	--- @nodiscard
	neg_one = function() end,

	--- Returns the x-axis unit vector.
	--- `x` is 1.0; `y` is 0.0.
	--- @return dvec2
	--- @nodiscard
	x = function() end,

	--- Returns the y-axis unit vector.
	--- `y` is 1.0; `x` is 0.0.
	--- @return dvec2
	--- @nodiscard
	y = function() end,

	--- Returns the negative x-axis unit vector.
	--- `x` is -1.0; `y` is 0.0.
	--- @return dvec2
	--- @nodiscard
	neg_x = function() end,

	--- Returns the negative y-axis unit vector.
	--- `y` is -1.0; `x` is 0.0.
	--- @return dvec2
	--- @nodiscard
	neg_y = function() end,
}

--- Static functions and constants for 3-dimensional 64-bit floating point vectors.
--- @class dvec3lib
dvec3 = {
	--- @param x number
	--- @param y number
	--- @param z number
	--- @return dvec3
	--- @nodiscard
	new = function(x, y, z) end,

	--- Return a `dvec3` with all elements set to `scalar`.
	--- @param scalar number
	--- @return dvec3
	--- @nodiscard
	splat = function(scalar) end,

	--- Returns a `dvec3` with all elements set to 0.0.
	--- @return dvec3
	--- @nodiscard
	zero = function() end,

	--- Returns a `dvec3` with all elements set to 1.0.
	--- @return dvec3
	--- @nodiscard
	one = function() end,

	--- Returns a `dvec3` with all elements set to -1.0.
	--- @return dvec3
	--- @nodiscard
	neg_one = function() end,

	--- Returns the x-axis unit vector.
	--- `x` is 1.0; all other elements are 0.0.
	--- @return dvec3
	--- @nodiscard
	x = function() end,

	--- Returns the y-axis unit vector.
	--- `y` is 1.0; all other elements are 0.0.
	--- @return dvec3
	--- @nodiscard
	y = function() end,

	--- Returns the z-axis unit vector.
	--- `z` is 1.0; all other elements are 0.0.
	--- @return dvec3
	--- @nodiscard
	z = function() end,

	--- Returns the negative x-axis unit vector.
	--- `x` is -1.0; all other elements are 0.0.
	--- @return dvec3
	--- @nodiscard
	neg_x = function() end,

	--- Returns the negative y-axis unit vector.
	--- `y` is -1.0; all other elements are 0.0.
	--- @return dvec3
	--- @nodiscard
	neg_y = function() end,

	--- Returns the negative z-axis unit vector.
	--- `z` is -1.0; all other elements are 0.0.
	--- @return dvec3
	--- @nodiscard
	neg_z = function() end,
}

--- Static functions and constants for 4-dimensional 64-bit floating point vectors.
--- @class dvec4lib
dvec4 = {
	--- @param x number
	--- @param y number
	--- @param z number
	--- @param w number
	--- @return dvec4
	--- @nodiscard
	new = function(x, y, z, w) end,

	--- Return a `dvec4` with all elements set to `scalar`.
	--- @param scalar number
	--- @return dvec4
	--- @nodiscard
	splat = function(scalar) end,

	--- Returns a `dvec4` with all elements set to 0.0.
	--- @return dvec4
	--- @nodiscard
	zero = function() end,

	--- Returns a `dvec4` with all elements set to 1.0.
	--- @return dvec4
	--- @nodiscard
	one = function() end,

	--- Returns a `dvec4` with all elements set to -1.0.
	--- @return dvec4
	--- @nodiscard
	neg_one = function() end,

	--- Returns the x-axis unit vector.
	--- `x` is 1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	x = function() end,

	--- Returns the y-axis unit vector.
	--- `y` is 1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	y = function() end,

	--- Returns the z-axis unit vector.
	--- `z` is 1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	z = function() end,

	--- Returns the w-axis unit vector.
	--- `w` is 1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	w = function() end,

	--- Returns the negative x-axis unit vector.
	--- `x` is -1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	neg_x = function() end,

	--- Returns the negative y-axis unit vector.
	--- `y` is -1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	neg_y = function() end,

	--- Returns the negative z-axis unit vector.
	--- `z` is -1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	neg_z = function() end,

	--- Returns the nagtive w-axis unit vector.
	--- `w` is -1.0; all other elements are 0.0.
	--- @return dvec4
	--- @nodiscard
	neg_w = function() end,
}
