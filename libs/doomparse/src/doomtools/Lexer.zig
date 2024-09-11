//! This code is adapted from DoomTools. See `/legal/doomtools.txt`.

const std = @import("std");

const Self = @This();

const end_of_stream = '\u{fffe}';

codepoints: std.unicode.Utf8View,

pub fn init(text: []const u8) !Self {
    return Self{
        .codepoints = try std.unicode.Utf8View.init(text),
    };
}

fn isDelimiterStart(cp: u21) bool {
    return switch (cp) {
        '(', ')' => true,
        '{', '}' => true,
        ',', '.' => true,
        ':', '|' => true,
        '+', '-' => true,
        else => false,
    };
}

/// <https://docs.oracle.com/javase/8/docs/api/java/lang/Character.html#isDigit-char->.
fn isJavaDigit(cp: u21) bool {
    switch (cp) {
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9' => true,
        // TODO: every other Unicode digit Java acknowledges.
        // Zig standard library doesn't offer any facilities for this yet.
        else => false,
    }
}

/// <https://docs.oracle.com/javase/8/docs/api/java/lang/Character.html#isWhitespace-char->
fn isJavaWhitespace(cp: u21) bool {
    return switch (cp) {
        '\t', '\n', '\r' => true,
        '\u{000C}', '\u{000B}', '\u{001C}', '\u{001D}', '\u{001E}', '\u{001F}' => true,
        '\u{2028}' => true,
        '\u{2029}' => true,
        '\u{0020}',
        '\u{00A0}',
        '\u{1680}',
        '\u{2000}',
        '\u{2001}',
        '\u{2002}',
        '\u{2003}',
        '\u{2004}',
        '\u{2005}',
        '\u{2006}',
        '\u{2007}',
        '\u{2008}',
        '\u{2009}',
        '\u{200A}',
        '\u{202F}',
        '\u{205F}',
        '\u{3000}',
        => true,
        else => false,
    };
}

fn isNewline(cp: u21) bool {
    return cp == '\n' or cp == '\r';
}
