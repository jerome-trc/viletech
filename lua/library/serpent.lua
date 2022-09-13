--- @meta

--- @class serpent_options
--- @field indent string Indentation; triggers long multi-line output.
--- @field comment boolean|integer Provide stringified value in a comment (up to `maxlevel` of depth).
--- @field sortkeys boolean|function Sort keys.
--- @field sparse boolean Force sparse encoding (no nil filling based on `#t`).
--- @field compact boolean Remove spaces.
--- @field fatal boolean Raise fatal error on non-serializable values.
--- @field fixradix boolean Change radix character set depending on locale to decimal dot.
--- @field nocode boolean Disable bytecode serialization for easy comparison.
--- @field nohuge boolean Disable checking numbers against undefined and huge values.
--- @field maxlevel number Specify max level up to which to expand nested tables.
--- @field maxnum number Specify max number of elements in a table.
--- @field maxlength number Specify max length for all table elements.
--- @field metatostring boolean Use `__tostring` metamethod when serializing tables; set to `false` to disable and serialize the table as-is, even when `__tostring` is present.
--- @field numformat string e.g. `"%.17g"`. Specify format for numberic values as shortest-possible round-trippable double. Use `"%.16g"` for better readability and `"%.17g"` (the default) to preserve floating point precision.
--- @field valignore table Allows specifying a list of values (as argument's keys) to ignore.
--- @field keyallow table Allows specifying the list of keys (as argument's keys) to be serialized. Any keys not in this list are not included in the final output.
--- @field keyignore table Allows specifying the list of keys to ignore during serialization.
--- @field custom function Provide custom output for tables.
--- @field name string Triggers full serialization with self-ref section.

--- @class serpent
serpent = {
	--- @param a any
	--- @param opts serpent_options
	--- @return string
	serialize = function(a, opts) end,
	--- Settings preset for `serialize`. Sets `compact` and `sparse` to true.
	--- @param a any
	--- @param opts serpent_options
	--- @return string
	dump = function(a, opts) end,
	--- Settings preset for `serialize`. Sets `sortkeys` and `comment` to `true`.
	--- @param a any
	--- @param opts serpent_options
	--- @return string
	line = function(a, opts) end,
	--- Settings preset for `serialize`. Sets `sortkeys` and `comment` to `true` and `indent` to `' '`.
	--- @param a any
	--- @param opts serpent_options
	--- @return string
	block = function(a, opts) end,

	--- @param data string The output of a call to `serialize`.
	--- @param opts serpent_options The same options passed to `serialize` to generate `data`.
	--- @return any
	deserialize = function(data, opts) end,
}

return serpent
