--- @meta

math = {
	--- Infinity. Alias for `math.huge`, for consistency.
	INF = math.huge,

	--- Negative infinity. Alias for `-math.huge`, for consistency.
	NEG_INF = -math.huge,

	--- The difference between a number and the next number that's higher or lower.
	EPSILON = 2.2204460492503131e-16,

	--- The largest possible number.
	MAX = 1.7976931348623157e+308,

	--- The smallest possible number.
	MIN = -1.7976931348623157e+308,

	--- Returns the inverse hyperbolic cosine of `x` (in radians).
	--- @param x number
	--- @return number
	--- @nodiscard
	acosh = function(x) end,

	--- Returns the inverse hyperbolic sine of `x` (in radians).
	--- @param x number
	--- @return number
	--- @nodiscard
	asinh = function(x) end,

	--- Returns the inverse hyperbolic tangent of `x` (in radians).
	--- @param x number
	--- @return number
	--- @nodiscard
	atanh = function(x) end,

	--- Returns the cube root of `x`.
	--- @param x number
	--- @return number
	--- @nodiscard
	cbrt = function(x) end,

	--- Linearly interpolates between `a` and `b` by `t`.
	--- If `t` is 0.0, the `a` will be returned. If `t` is 1.0, `b` will be returned.
	--- If `t` is outside the range of [0.0, 1.0], the result is linearly extrapolated.
	--- @param a number Starting value.
	--- @param b number Ending value.
	--- @param t number Interpolation value between `a` and `b`.
	--- @return number
	--- @nodiscard
	lerp = function(a, b, t) end,

	--- Returns the logarithm of `n` with respect to arbitrary base `b`.
	--- Prefer `log2` or `log10` if possible, since they may be more accurat.e
	--- @param x number
	--- @param b number
	--- @return number
	--- @nodiscard
	logn = function(x, b) end,

	--- Returns the base 2 logarithm of `x`.
	--- @param x number
	--- @return number
	--- @nodiscard
	log2 = function(x) end,

	--- @param a number
	--- @param b number
	--- @return number
	--- @nodiscard
	hypotenuse = function(a, b) end,
}

return math
