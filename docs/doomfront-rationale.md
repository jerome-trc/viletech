# DoomFront Rationale

This is a living document intended to rationalize DoomFront's existence, as well as all of its underlying design and implementation decisions.

## Why DoomFront?

Out of all the dozens of domain-specific languages (henceforth just "DSLs") invented for Doom's source ports, not one currently has any tooling more sophisticated than syntax highlighting, and even this, as of Q2 2023, extends only to GZDoom. Given that GZDoom in particular suffers badly from its flagship scripting language "ZScript" lacking documentation from its developers, this is an apparent open niche.

Additionally, these languages are all very unspecified. Only ZScript has something resembling a formalized grammar by way of an input file for the Lemon parser generator, but because it is - unlike a purpose-made specification - written for Lemon first and humans second, it is overly sparse and filled with line noise.

DoomFront is meant to make it possible to handle all of these DSLs in a way that can be consumed by any other Rust project (and by any C project, if the requisite bindings are created).

## Rowan

The [Rowan library](https://crates.io/crates/rowan) was originally developed by Alexey Kladov et al. to meet the specific needs of Rust language server development, although it was designed to be generic enough to work for any language. Seeing as how these needs were also my own, it seemed fitting to use it, especially considering that its performance characteristics are already thoroughly proven (I use rust-analyzer on a daily basis and it is more than fast enough). It is all the better that Rowan makes for a very ergonomic experience when traversing abstract syntax trees without compromising on its ability to represent malformed code.

## Chumsky

Rust has no shortage of libraries with which to build parsers; I singled out [Chumsky](https://crates.io/crates/chumsky) after testing multiple other solutions on the basis of the following factors:

1. Productivity. There are too many languages that need coverage by a project such as this to painstakingly hand-write a "local optimum" parser for each one. Combinators meet this need well; imperative code can be seamlessly mingled with constructs that are as easy to read as regular expressions.
2. Performance. An IDE and its constituent parts should react to user changes fast, which means parsing can not do unnecessary work.
3. Granularity. I wanted any given language frontend to offer the capability to parse syntax elements in isolation, to enable tools to work incrementally rather than having to re-parse an entire file whenever a user so much as changed a character. Again, combinators are well-suited to this. Every syntax element can be mapped to a function returning a combinator.
4. Resilience. A parser for IDEs is of little use if it can not move past an error; Chumsky makes error recovery a headline feature.

As a bonus, the organizational problems that tend to come with Rust's existing parser generator ecosystem are avoided; there is no obligation to create single massive procedural macro inputs or grammar files. Combinators can be spread over several modules according to sensible categories.
