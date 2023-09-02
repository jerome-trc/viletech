# VZScript Specification

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

## Versioning

This specification follows [Semantic Versioning](https://semver.org/). Its content is relevant to the latest version; this section contains a list of major versions and the breaking changes introduced therein to lead up to this docuent.
