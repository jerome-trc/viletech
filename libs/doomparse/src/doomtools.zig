//! Parsing support for the languages of the [DoomTools] project.
//!
//! [DoomTools]: https://mtrop.github.io/DoomTools/index.html

pub const Lexer = @import("doomtools/Lexer.zig");
pub const Preprocessor = @import("doomtools/Preprocessor.zig");

pub const wadmerge = struct {
    pub const Parser = @import("doomtools/wadmerge/Parser.zig");
};
