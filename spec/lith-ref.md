# LithScript Reference

Note that this is a living document.

## Whitespace

The following characters are considered whitespace and are insignificant to the LithScript syntax:
- ` ` (0x20)
- `\n` (0x10)
- `\r` (0x13)
- `\t` (0x11)

## Comments

Lith uses Rust-style comments. These are treated as whitespace by the compiler.

`// This is a single-line comment.`
`x = /* Block comments can be written between tokens.*/ y + z`
```
/*
Block comments can span multiple lines.
*/
```

## Keywords

The following keywords are reserved and can not be used as identifiers or labels:

`await` `break` `catch` `continue` `defer` `do` `else` `finally` `for` `foreach` `if` `in` `loop` `return` `switch` `try` `until` `use` `while` `yield`

`abstract` `ceval` `const` `final` `override` `private` `protected` `public` `static` `using` `virtual`

`async` `class` `enum` `extend` `interface` `macro` `mixin` `module` `property` `struct` `type` `union` `unsafe`

`as` `case` `default` `let` `use` `where` `with`

This list is intended to be overly restrictive, and is eligible for relaxation in the future as the design of the language crystallizes.

## Literals

`true` and `false` are boolean literals. `null` is a pointer literal.

String literals are delimited by `"`.

## Identifiers

Identifiers are used to name items, types, and variables. An identifier can consist of any ASCII alphanumerical characters (a-z A-Z 0-9) and underscores, with the restriction that the first character of any identifier can not be numeric.

Identifiers beginning with an underscore (e.g. `_id_tech`) are to be used as a hint to tools that the thing under the identifier is allowed to go unused.

The "discard" identifier consists of only one underscore (`_`) and is used to indicate to both humans and tools that a declared local variable or parameter is unused by the program. This identifier can not overlap or shadow other identical identifiers and can not be consumed; a variable under this identifier can not be passed as an argument.

## Type System

Lith leans to strong, gradual typing.

`void` is a non-type that only exists for the purposes of function return type specification.

### Primitive Types

`bool` is a strongly-typed true-or-false byte.

`char` uses Rust character semantics; it represents a single valid UTF-8 code point.

Lith's integral and floating-point types map to LLVM integral types for brevity and immediate clarity as to their size:

`i8` `u8`
`i16` `u16`
`i32` `u32`
`i64` `u64`
`i128` `u128`
`f32` `f64`

Integers of different signedness and bit-width can only be converted via cast.

The function pointer type is written `function<(A...) R>`, where `A` is any number of arguments and `R` is a return type.

## Resolvers

"Resolver" is the name given to what is known in Rust as a "path"; a series of identifiers with optionally-interspersed generic arguments joined by the scope resolution operator `::`. The name was chosen as such since "path" is a term reserved for filesystem nomenclature.

## Operators

What follows is the complete list of Lith operators.

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

Annotations are an all-purpose system for compile-time qualifications of arbitrary elements of code. They are modeled directly after Rust's "attributes" down to the syntax, while being named after Java/C# annotations. An "outer" annotation is written `#[resolver]`, and will apply to the next node in the AST. An "inner" annotation is written `#![resolver]`, and will apply to the parent/"surrounding" AST node (e.g. the translation unit, the block).

## Functions

Lith does not support function overloading; identifiers must be unique.

## Macros

Definition of a Lith macro takes the following syntax: `macro macro_name(input, output) {}`, where `macro_name` is any valid identifier. The parameters can not take on any other name and do not require type specification. `input` is always a token stream and `output` is always a stream into which new AST nodes can be passed.

The block within can be omitted for the purpose of declaring the macro without defining it, but only if the declaration is annotated as `#[native]`.

Invocation of a macro uses Rust form, wherein the macro's identifier is followed by a `!` and the input string is surrounded a pair of delimiters, both of which must belong to one kind from a choice of three. If using braces (`{}`), no semicolon is needed after the closing delimiter, but one is needed if using parentheses (`()`) or brackets (`[]`).
