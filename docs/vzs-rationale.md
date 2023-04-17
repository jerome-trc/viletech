# Why VZScript?

A successful modding-oriented Doom source port can not leave behind the entirety of the ZDoom content ecosystem, much like modern languages benefit from having access to  C libraries. This means being compatible with ZScript, either by supporting it directly or allowing transpilation, but either way a custom memory model is needed, since ZScript allows arbitrary object destruction via a specialised read barrier. This already precludes every reference-counted language and every language with a traditional garbage collector.

Note, however, that a language such as this is not necessarily confined to being useful to VileTech. VZScript is designed such that minimal work would be needed to separate it into its own library and integrate it into other programs, whether they are games, game engines, or neither. In terms of options for a performance-first statically-typed language that comes with a garbage collector, interpreter, and JIT (without having to link against LLVM), [few options exist](https://github.com/dbohdan/embedded-scripting-languages).

There are other, more personal reasons too. Learning how to construct a programming language has been a highly rewarding personal exercise and learning experience, and I am looking forward to exploring ways to bring metaprogramming features that are expressive, powerful, and still accessible to users new to programming.
