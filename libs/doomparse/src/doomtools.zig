//! Parsing support for the languages of the [DoomTools] project.
//!
//! [DoomTools]: https://mtrop.github.io/DoomTools/index.html

const tree_sitter = @import("tree-sitter.zig");

pub const Lexer = @import("doomtools/Lexer.zig");
pub const Preprocessor = @import("doomtools/Preprocessor.zig");

pub const wadmerge = struct {
    pub const Language = tree_sitter.Language("wadmerge");
    pub const Node = Language.Node;
    pub const Parser = Language.Parser;
    pub const Syntax = tree_sitter.SymbolEnum(@embedFile("doomtools/wadmerge/parser.c"));
    pub const Tree = Language.Tree;
};

test {
    const std = @import("std");
    std.testing.refAllDecls(@import("doomtools/wadmerge/test/parse.zig"));
}
