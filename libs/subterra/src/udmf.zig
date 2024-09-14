//! Parsing support for the [Universal Doom Map Format].
//!
//! [Universal Doom Map Format]: https://doomwiki.org/wiki/UDMF

const std = @import("std");

const Lexer = @import("UdmfLexer.zig");

/// `textmap` needs to be null-terminated as a concession to drastically simplify
/// and optimize the lexer. When reading the source, allocate accordingly.
pub fn parse(textmap: [:0]const u8) noreturn {
    _ = textmap;
    @panic("not yet implemented");
}

pub const Token = struct {
    pub const Kind = enum {
        brace_l,
        brace_r,
        eq,
        ident,
        kw_false,
        kw_linedef,
        kw_namespace,
        kw_sector,
        kw_sidedef,
        kw_thing,
        kw_true,
        kw_vertex,
        lit_float,
        lit_int,
        lit_string,
        semicolon,
    };

    kind: Kind,
    start: usize,
    end: usize,
};

test "lexer, smoke" {
    const sample =
        \\siDedef {
        \\  offsetx = 4;
        \\  bespoke = 6.66;
        \\  // the stormdrain
        \\  boolean = true;
        \\  /* town square */
        \\  comment = "ebb and flow";
        \\}
    ;

    const expect = struct {
        fn nextToken(
            lexer: *Lexer,
            comptime expected_kind: Token.Kind,
            comptime expected_string: []const u8,
        ) !void {
            const token = lexer.next().?;
            try std.testing.expectEqual(expected_kind, token.kind);
            try std.testing.expectEqualStrings(expected_string, sample[token.start..token.end]);
        }
    };

    var lexer = Lexer.init(sample);
    try expect.nextToken(&lexer, .kw_sidedef, "siDedef");
    try expect.nextToken(&lexer, .brace_l, "{");

    try expect.nextToken(&lexer, .ident, "offsetx");
    try expect.nextToken(&lexer, .eq, "=");
    try expect.nextToken(&lexer, .lit_int, "4");
    try expect.nextToken(&lexer, .semicolon, ";");

    try expect.nextToken(&lexer, .ident, "bespoke");
    try expect.nextToken(&lexer, .eq, "=");
    try expect.nextToken(&lexer, .lit_float, "6.66");
    try expect.nextToken(&lexer, .semicolon, ";");

    try expect.nextToken(&lexer, .ident, "boolean");
    try expect.nextToken(&lexer, .eq, "=");
    try expect.nextToken(&lexer, .kw_true, "true");
    try expect.nextToken(&lexer, .semicolon, ";");

    try expect.nextToken(&lexer, .ident, "comment");
    try expect.nextToken(&lexer, .eq, "=");
    try expect.nextToken(&lexer, .lit_string, "\"ebb and flow\"");
    try expect.nextToken(&lexer, .semicolon, ";");

    try expect.nextToken(&lexer, .brace_r, "}");

    try std.testing.expectEqual(null, lexer.next());
}
