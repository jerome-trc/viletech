--- @meta

--- @class rnglib
rng = {
	--- @param min integer The lowest number that can be returned.
	--- @param max integer The highest number that can be returned.
	--- @param rng string? If given, a distinct random number generator matching this ID will be used. If `nil` is passed, the unnamed RNG will be used as a default. Passing the ID of an RNG that hasn't been pre-declared causes an error.
	--- @return integer
	--- @nodiscard
	int = function(min, max, rng) end,
	--- @param min number The lowest number that can be returned.
	--- @param max number The highest number that can be returned.
	--- @param rng string? If given, a distinct random number generator matching this ID will be used. If `nil` is passed, the unnamed RNG will be used as a default. Passing the ID of an RNG that hasn't been pre-declared causes an error.
	--- @return number
	--- @nodiscard
	float = function(min, max, rng) end,
	--- Returns one of the values from the given array at random.
	--- @generic T
	--- @param array T[] If `#array == 0`, `nil` gets returned.
	--- @param rng string? If given, a distinct random number generator matching this ID will be used. If `nil` is passed, the unnamed RNG will be used as a default. Passing the ID of an RNG that hasn't been pre-declared causes an error.
	--- @return T
	--- @nodiscard
	pick = function(array, rng) end,
	--- Returns `true` or `false` at random.
	--- @param rng string? If given, a distinct random number generator matching this ID will be used. If `nil` is passed, the unnamed RNG will be used as a default. Passing the ID of an RNG that hasn't been pre-declared causes an error.
	--- @return boolean
	--- @nodiscard
	coinflip = function(rng) end,
}

return rng
