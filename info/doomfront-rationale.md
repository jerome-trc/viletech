# DoomFront Rationale

This is a living document intended to rationalize DoomFront's existence, as well as all of its underlying design and implementation decisions.

## Why DoomFront?

Out of all the dozens of domain-specific languages (henceforth just "DSLs") invented for Doom's source ports, not one currently has any tooling more sophisticated than syntax highlighting, and even this, as of Q2 2023, extends only to GZDoom. Given that GZDoom in particular suffers badly from its flagship scripting language "ZScript" lacking documentation from its developers, this is an apparent open niche.

Additionally, these languages are all very unspecified. Only ZScript has something resembling a formalized grammar by way of an input file for the Lemon parser generator, but because it is - unlike a purpose-made specification - written for Lemon first and humans second, it is overly sparse and filled with line noise.

DoomFront is meant to make it possible to handle all of these DSLs in a way that can be consumed by any other Rust project (and by any C project, if the requisite bindings are created).

## Rowan

The [Rowan library](https://crates.io/crates/rowan) was originally developed by Alexey Kladov et al. to meet the specific needs of Rust language server development, although it was designed to be generic enough to work for any language. Seeing as how these needs were also my own, it seemed fitting to use it, especially considering that its performance characteristics are already thoroughly proven (I use rust-analyzer on a daily basis and it is more than fast enough). It is all the better that Rowan makes for a very ergonomic experience when traversing abstract syntax trees without compromising on its ability to represent malformed code.

## Parsers

All DoomFront parsers are hand-written LL recursive descent. The stability of all the languages in DoomFront's scope make this easy, and the other apparent benefits are in sheer speed and flexibility in error recovery. That said, there are multiple other factors which make it undesirable to use other solutions:

- Parser generators harder for IDE tools to reconcile with and not very amenable to step-through debugging.
- [`peg`] and [`pest`] in particular both have no native error recovery facilities, making them non-starters.
- The Rust compiler is unfriendly towards combinator parsing libraries, due to the complexity of the created types; because of the depth of syntaxes like DECORATE and ZScript, combinator types can become so large as to make DoomFront the majority consumer of time in a full build of the VileTech project at best. At worst, rustc can use exponential memory and OOM.
- LALR parsers are a non-option due to the significance of whitespace to lossless syntax trees, which introduces shift ambiguities.

## Token-to-Syntax Mapping

(G)ZDoom uses [one lexer](https://github.com/ZDoom/gzdoom/blob/master/src/common/engine/sc_man_scanner.re) for all of its languages, and DoomFront [re-implements it internally](../doomfront/src/zdoom/lex.rs) in the interest of parity. Rowan, however, demands that every language built on it has one "syntax tag" type used for both tokens and higher-level "syntax nodes" (e.g. expressions).

As such, the token type and syntax type can never be reconciled, but it is beneficial to have a direct (and fast) conversion from the lexed token to the correspdonding syntax token. As an example, it may be helpful for a diagnostic tool to recognize the presence of a foreign keyword so that it recommend to the user that the keyword in question is not supported by the language, and offer a different solution by inferring the user's possible intent. This is why every "`Syn`" type has a token-to-syntax mapping table; they are bulky and tedious to prepare but pay dividends.
