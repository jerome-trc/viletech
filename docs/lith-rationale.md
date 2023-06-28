# LithScript Rationale

This is a living document intended to rationalize LithScript's existence, as well as all of its underlying design decisions.

## Why a new language?

A successful modding-oriented Doom source port can not leave behind the entirety of the ZDoom content ecosystem, much like modern languages benefit from having access to  C libraries. This means being compatible with ZScript, either by supporting it directly or allowing transpilation, but either way a custom memory model is needed, since ZScript allows arbitrary object destruction via a specialised read barrier. This already precludes every reference-counted language and every language with a traditional garbage collector.

Note, however, that a language such as this is not necessarily confined to being useful to VileTech. LithScript is designed such that minimal work would be needed to separate it into its own library and integrate it into other programs, whether they are games, game engines, or neither. In terms of options for a performance-first statically-typed language that comes with a garbage collector, interpreter, and JIT (without having to link against LLVM), [few options exist](https://github.com/dbohdan/embedded-scripting-languages).

There are other, more personal reasons too. Learning how to construct a programming language has been a highly rewarding personal exercise and learning experience, and I am looking forward to exploring ways to bring metaprogramming features that are expressive, powerful, and still accessible to users new to programming.

## File Organization

ZScript uses a filesystem evocative of the C preprocessor; translation units are virtually inlined into a root file using `#include` directives.

Lith instead opted to go the route of making the user define a root directory instead, and recursively including all files in the directory into one library, as long as they are text and use the path extension `.lith`. This has two benefits over ZS:

1. `#include` directives have proven redundant, since the average user keeps all their scripts in one directory and then includes them all anyway. Now this redundancy is gone. If a file needs to be excluded, it can be suffixed with an added extension.
2. A file extension is enforced, promoting better organizational practises.

The only potential downside to this system is that a WAD file intended to include Lith can not provide multiple files with different names, which would be possible with the `#include` system, but this is an extremely niche case, and can possibly be solved via specification of an external TOML manifest anyway.

An alternative solution would have been to keep `#include` but allow supplying glob patterns, although this would only be a reduction of problem 1 rather than a solution.
