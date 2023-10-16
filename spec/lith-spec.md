# Lithica Specification

Note that this is a living document.

## Versioning

This specification follows [Semantic Versioning](https://semver.org/). Its content is relevant to the latest version; this section contains a list of major versions and the breaking changes introduced therein to lead up to this docuent.

## Notation

All grammar specifications are in an Extended Backus-Naur Form (EBNF), as laid out by this document:

https://www.w3.org/TR/xml/#sec-notation

```ebnf
AsciiLetter ::= [a-zA-Z]
AsciiDigit ::= [0-9]
AsciiAlphanum ::= AsciiLetter | AsciiDigit
AsciiWord ::= ('_' | AsciiLetter) ('_' | AsciiAlphanum)*
```

## Source

Lithica source shall be UTF-8 encoded; files containing Lithica source code shall be extended with `.lith`.

## Whitespace

The following characters shall be considered whitespace and thus be insignificant to the Lithica syntax:
- ` ` (0x20)
- `\n` (0x10)
- `\r` (0x13)
- `\t` (0x11)

## Comments

The regular expressions for a Lithica non-documenting comment shall be:
- `//[^/\n][^\n]*`
- `////[^\n]*`
- `//`

Lithica non-documenting comments shall be treated like whitespace and thus be insignificant to the Lithica syntax.

## Type System

### Primitives

Implicit conversions between numeric types shall always implicitly widen but never implicitly narrow.

Lithica shall have the following integer types:
- `i8`; 8 bits signed
- `u8`; 8 bits unsigned
- `i16`; 16 bits signed
- `u16`; 16 bits unsigned
- `i32`; 32 bits signed
- `u32`; 32 bits unsigned
- `i64`; 64 bits signed
- `u64`; 64 bits unsigned

Optionally, implementors may provide the following integer types:
- `i128`; 128 bits signed
- `u128`; 128 bits unsigned

Integer operations which may overflow shall wrap.

Lithica shall have the following floating-point types:
- `f32`; IEEE-754-2008 binary32
- `f64`; IEEE-754-2008 binary64

## Expressions

### Call

Arguments in a call expression shall be evaluated from left to right.

The return value of a call expression may be discarded but an implementor's default diagnostic behavior shall be to emit a warning unless the function declaration is annotated `can_discard`.

## Annotations

```ebnf
Annotation ::= '#' '!'? '[' (Identifier '.')? Identifier ArgumentList? ']'
```

Annotations are Lithica's generalized system for adding metadata to pieces of syntax.
