//! Parsing support for the languages of the [DoomTools] project.
//!
//! [DoomTools]: https://mtrop.github.io/DoomTools/index.html

pub const Lexer = @import("doomtools/Lexer.zig");
pub const Preprocessor = @import("doomtools/Preprocessor.zig");

pub const Token = struct {
    kind: u32,
    line: u32,
    char: u32,
};
