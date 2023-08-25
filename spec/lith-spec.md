# LithScript Specification

- [LithScript Specification](#lithscript-specification)
	- [Notation](#notation)
	- [Whitespace](#whitespace)
	- [Comments](#comments)
	- [Keywords](#keywords)
	- [Identifiers](#identifiers)
	- [Type System](#type-system)
		- [Primitive Types](#primitive-types)
	- [Editions](#editions)

Note that this is a living document.

## Notation

All grammar specifications are in an Extended Backus-Naur Form (EBNF), as laid out by this document:

https://www.w3.org/TR/xml/#sec-notation

For clarity, the symbol rule `_` is used to indicate the requirement for one or more whitespace characters.

```ebnf
AsciiLetter ::= [a-zA-Z]
AsciiDigit ::= [0-9]
AsciiAlphanum ::= AsciiLetter | AsciiDigit
AsciiWord ::= ('_' | AsciiLetter) ('_' | AsciiAlphanum)*
```

## Whitespace

The following characters are considered whitespace and are insignificant to the LithScript syntax:
- ` ` (0x20)
- `\n` (0x10)
- `\r` (0x13)
- `\t` (0x11)

## Comments

LithScript uses C++/post-C99 single-line comments. These are treated as though they were whitespace by the compiler.

`// This is a single-line comment.`

## Keywords

The following keywords are reserved and can not be used as identifiers or labels:

`await` `break` `catch` `continue` `defer` `do` `else` `finally` `for` `foreach` `if` `in` `loop` `recover` `return` `switch` `try` `until` `use` `while` `yield`

`abstract` `ceval` `const` `final` `override` `private` `protected` `public` `static` `throws` `throw` `using` `virtual`

`class` `enum` `extend` `interface` `macro` `mixin` `module` `property` `struct` `union` `unsafe`

`as` `case` `default` `let` `use` `where` `with`

This list is intended to be overly restrictive, and is eligible for relaxation in the future as the design of the language crystallizes.

## Identifiers

```ebnf
Identifier ::= AsciiWord
```

Identifiers are used to name items, types, and variables. An identifier can consist of any ASCII alphanumerical characters (a-z A-Z 0-9) and underscores, with the restriction that the first character of any identifier can not be numeric. The ASCII restriction is arbitrary and may be lifted to allow identifiers of characters covering the UTF-8 XID range in the future.

Identifiers beginning with an underscore (e.g. `_id_tech`) are to be used as a hint to tools that the thing under the identifier is allowed to go unused.

Identifiers beginning and/or ending with two underscores (e.g. `__escape`, `Castle__`, or `__WOLFENSTEIN__`) are invalid for use by user scripts, as they are reserved for internal use.

The "discard" identifier consists of only one underscore (`_`) and is used to indicate to both humans and tools that a declared local variable or parameter is unused by the program. This identifier can not overlap or shadow other identical identifiers and can not be consumed; a variable under this identifier can not be passed as an argument.

## Type System

LithScript leans towards strong, gradual typing.

### Primitive Types

`void` is a tangible, zero-size type with the literal `()`.
`bool` is a strongly-typed true-or-false byte.
`char` uses Rust character semantics; it represents a single valid UTF-8 code point.

LithScript has the following numeric types:

`int8` `uint8`
`int16` `uint16`
`int32` `uint32`
`int64` `uint64`
`float` `float64`

The following aliases are also provided:
- `int` for `int32`
- `uint` for `uint32`

## Editions

This specification follows [Semantic Versioning](https://semver.org/). Its content is relevant to the latest version; this section contains a list of major versions and the breaking changes introduced therein to lead up to this docuent.
