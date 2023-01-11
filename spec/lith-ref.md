# LithScript Reference

- [LithScript Reference](#lithscript-reference)
	- [Grammar Basics](#grammar-basics)
	- [Whitespace](#whitespace)
	- [Comments](#comments)
	- [Keywords](#keywords)
	- [Literals](#literals)
	- [Identifiers](#identifiers)
	- [Type System](#type-system)
		- [Primitive Types](#primitive-types)
		- [Enumerations](#enumerations)
		- [Classes](#classes)
		- [Structures](#structures)
		- [Unions](#unions)
		- [Bitfields](#bitfields)
		- [Pointers and References](#pointers-and-references)
	- [Resolvers](#resolvers)
	- [Operators](#operators)
		- [Unary](#unary)
		- [Binary](#binary)
	- [Annotations](#annotations)
	- [Functions](#functions)
	- [Macros](#macros)
	- [Type Aliases](#type-aliases)

Note that this is a living document.

## Grammar Basics

All grammar specifications are in Backus-Naur Form. In any given specification, the character `_` is used to indicate the requirement for one or more whitespace characters.

```bnf
<ASCII_LETTER> ::= a to z or A to Z
<ASCII_DIGIT> ::= 0 to 9
<ASCII_ALPHANUM> ::= <ASCII_LETTER> | <ASCII_DIGIT>
<ASCII_WORD> ::= ("_" | <ASCII-letter>) ("_" | <ASCII_ALPHANUM>)*
```

## Whitespace

The following characters are considered whitespace and are insignificant to the LithScript syntax:
- ` ` (0x20)
- `\n` (0x10)
- `\r` (0x13)
- `\t` (0x11)

## Comments

LithScript uses Rust-style comments. These are treated as whitespace by the compiler.

`// This is a single-line comment.`
`x = /* Block comments can be written between tokens.*/ y + z`
```
/*
Block comments can span multiple lines.
*/
```

## Keywords

The following keywords are reserved and can not be used as identifiers or labels:

`await` `break` `catch` `continue` `defer` `do` `else` `finally` `for` `foreach` `if` `in` `loop` `recover` `return` `switch` `try` `until` `use` `while` `yield`

`abstract` `ceval` `const` `final` `override` `private` `protected` `public` `static` `throws` `throw` `using` `virtual`

`class` `enum` `extend` `interface` `macro` `mixin` `module` `property` `struct` `union` `unsafe`

`as` `case` `default` `let` `use` `where` `with`

This list is intended to be overly restrictive, and is eligible for relaxation in the future as the design of the language crystallizes.

## Literals

`true` and `false` are boolean literals. `null` is a pointer literal.

String literals are delimited by `"`.

Character literals are delimited by `'`, and contain exactly one UTF-8 character.

## Identifiers

```bnf
<IDENTIFIER> ::= <ASCII_WORD>
```

Identifiers are used to name items, types, and variables. An identifier can consist of any ASCII alphanumerical characters (a-z A-Z 0-9) and underscores, with the restriction that the first character of any identifier can not be numeric.

Identifiers beginning with an underscore (e.g. `_id_tech`) are to be used as a hint to tools that the thing under the identifier is allowed to go unused.

Identifiers beginning and/or ending with two underscores (e.g. `__escape`, `Castle__`, or `__WOLFENSTEIN__`) are invalid for use by user scripts, as they are reserved for internal use.

The "discard" identifier consists of only one underscore (`_`) and is used to indicate to both humans and tools that a declared local variable or parameter is unused by the program. This identifier can not overlap or shadow other identical identifiers and can not be consumed; a variable under this identifier can not be passed as an argument.

## Type System

LithScript leans towards strong, gradual typing.

`void` is a non-type that only exists for the purposes of function return type specification.

### Primitive Types

`bool` is a strongly-typed true-or-false byte.

`char` uses Rust character semantics; it represents a single valid UTF-8 code point.

LithScript's integral and floating-point types map to LLVM integral types for brevity and immediate clarity as to their size:

`i8` `u8`
`i16` `u16`
`i32` `u32`
`i64` `u64`
`i128` `u128`
`f32` `f64`

Integers of different signedness and bit-width can only be converted via cast.

The function pointer type is written `function<(A...) R>`, where `A` is any number of arguments and `R` is a return type.

LithScript supports a type information primitive, the name of which remains up for bikeshedding. Current choices are `typedef` and `typeinfo`.

### Enumerations

LithScript enumerations behave similarly to C++ scoped enumerations. Each is a series of named integral constants, incrementing with each variant declared, with an optional discriminant in the form of a constant expression.

Enumerations support methods, but they may only be declared in an `extend enum EnumIdentifier` block. Enum extensions can not have more variants declared in them.

### Classes

A class object in LithScript is a reference-counted heap object that can never be held by value.

Classes may be "extended" with an `extend class ClassIdentifier` block. These get merged into the original class definition during preprocessing, and are meant as a way to ease code organization.

A class qualified with `abstract` can not be instantiated, and serves only as a base for other classes.

A class qualified with `final` may not be inherited from. This keyword and `abstract` are mutually-exclusive.

### Structures

A LithScript structure is a compositional aid, meant to be used primarily for grouping fields and functions together.

Structs may be "extended" with an `extend struct StructIdentifier` block. These get merged into the original struct definition during preprocessing, and are meant as a way to ease code organization.

### Unions

LithScript unions are tagged algebraic types.

Unions support methods, but they may only be declared in an `extend union UnionIdentifier` block. Union extensions can not have more variants declared in them.

### Bitfields

```bnf
<BITFIELD_DEF> ::= <DECL_QUAL>* _ "bitfield" _ <IDENTIFIER> _ ":" _ <TYPE_EXPR> _ "{" <BITFIELD_BITDEF>* "}"
<BITFIELD_BITDEF> ::= <IDENTIFIER> ":" ((<LITERAL_INT> | <IDENTIFIER>) ",")*
```

A bitfield is a type which wraps a single unsigned integer and behaves like a structure of booleans.

A bitfield's underlying integer is given the identifier `__bits`.

### Pointers and References

LithScript offers two ways to manage memory in the VM heap: `Ref` and `Ptr`. `Ref` is a handle to a heap value that is guaranteed at compile time to be non-null. `Ptr` is a type known to the compiler which uses the same semantics as `Option`. These handle types are the only ways to interact with class objects, which are never held as script values.

## Resolvers

```bnf
<RESOLVER> ::= "::"? <RESOLVER_PART> ("::" <RESOLVER_PART>)*
<RESOLVER_PART> ::= <ASCII_WORD> | "super" | "Self"
```

"Resolver" is the name given to what is known in Rust as a "path"; a series of identifiers with optionally-interspersed generic arguments joined by the scope resolution operator `::`. The name was chosen as such since "path" is a term reserved for filesystem nomenclature.

## Operators

What follows is the complete list of LithScript operators.

### Unary

`-`: arithmetic negation operator.
`++`: increment operator. Can be used in pre- or post-fix form on an integer variable.
`--`: decrement operator. Can be used in pre- or post-fix form on an integer variable.
`!`: logical negation operator.
`~`: bitwise negation operator.

### Binary

`is`: type equality comparison with transitive inheritance.
`!is`: type inequality comparison (since `!` can only be applied to expression operands) is a special token.
`..`: string concatenation.
`..=`: string concatenation assignment.
`.`: field access.
`=`: variable assignment.
`::`: scope resolution.

`+`: arithmetic addition.
`-`: arithmetic subtraction.
`*`: arithmetic multiplication.
`/`: arithmetic division.
`**`: arithmetic exponentiation.
`%`: arithmetic modulus.

`+=`: arithmetic addition assignment.
`-=`: arithmetic subtraction assignment.
`*=`: arithmetic multiplication assignment.
`/=`: arithmetic division assignment.
`**=`: arithmetic exponentiation assignment.
`%=`: arithmetic modulus assignment.

`<`: arithmetic less-than comparison.
`>`: arithmetic greater-than comparison.
`<=`: arithmetic less-than-or-equal comparison.
`>=`: arithmetic greater-than-or-equal comparison.

`<<`: bitwise left shift.
`>>`: bitwise right shift.
`>>>`: bitwise right shift, unsigned-specific.
`&`: bitwise AND.
`|`: bitwise OR.
`^`: bitwise XOR.

`<<=`: bitwise left shift.
`>>=`: bitwise right shift.
`>>>=`: bitwise right shift, unsigned-specific.
`&=`: bitwise AND assignment.
`|=`: bitwise OR assignment.
`^=`: bitwise XOR assignment.

`&&`: logical AND comparison.
`||`: logical OR comparison.
`^^`: logical XOR comparison.

`&&=`: logical AND compound assignment.
`||=`: logical OR compound assignment.
`^^=`: logical XOR compound assignment.

`==`: logical equality comparison.
`!=`: logical negative comparison.
`~==`: logical approximate equality comparison. Can be used on strings to test case-insensitive equality and on floating-point numbers to check for equality with a small tolerance margin.

## Annotations

```bnf
ANNOTATION ::= "@" "!"? RESOLVER ANNOTATION_ARGS?
ANNOTATION_ARGS ::= "(" (EXPR ",")* ")"
```

Annotations are an all-purpose system for compile-time qualifications of arbitrary elements of code. They are modeled after attributes in Rust and C# and "annotations" from Java, from which this feature gets its name and syntax. All annotations begin with a `@` character. If that glyph is followed by a `!`, it's an "inner" annotation, and applies to the parent/"surrounding" AST node (e.g. the translation unit or block).

## Functions

LithScript does not support function overloading; identifiers must be unique.

A function declared in the body of a class or structure is considered a class method unless it is qualified with `static`.

## Macros

A LithScript macro is a function with special semantics. Declaration of macro requires the signature `#[macro] TokenStream macro_name(input: TokenStream) {}`;  `macro_name` is any valid identifier, and the annotation `#[macro]` is a compiler built-in. The block within can be omitted for the purpose of declaring the macro without defining it, but only if the declaration is also annotated as `#[native]`.

Invocation of a macro uses Rust form, wherein the macro's identifier is followed by a `!` and the input string is surrounded a pair of delimiters, both of which must belong to one kind from a choice of three. If using braces (`{}`), no semicolon is needed after the closing delimiter, but one is needed if using parentheses (`()`) or brackets (`[]`).

## Type Aliases

```bnf
<TYPE_ALIAS> ::= "using" _ <IDENTIFIER> "=" <TYPE_EXPR>
```
