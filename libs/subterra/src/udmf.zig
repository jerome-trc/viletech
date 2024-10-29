//! Parsing support for the [Universal Doom Map Format].
//!
//! [Universal Doom Map Format]: https://doomwiki.org/wiki/UDMF

const std = @import("std");

const Lexer = @import("UdmfLexer.zig");

pub fn parse(textmap: []const u8, context: anytype) Error!void {
    const Context = @TypeOf(context);

    var lexer = Lexer.init(textmap);

    try parseNamespace(&lexer, context);

    var parser = switch (@typeInfo(Context)) {
        .Pointer => |ptr_t| Parser(ptr_t.child){
            .context = context,
            .lexer = lexer,
            .buf = null,
        },
        else => Parser(Context){
            .context = &context,
            .lexer = lexer,
            .buf = null,
        },
    };

    while (parser.advance()) |token| {
        switch (token.kind) {
            .kw_linedef => {
                parser.lineDef();
            },
            .kw_sector => {
                parser.sector();
            },
            .kw_sidedef => {
                parser.sideDef();
            },
            .kw_thing => {
                parser.thing();
            },
            .kw_vertex => {
                parser.vertex();
            },
            else => {
                parser.syntaxErr(token, &[_]Token.Kind{
                    .kw_linedef,
                    .kw_sector,
                    .kw_sidedef,
                    .kw_thing,
                    .kw_vertex,
                });

                parser.skipUntil(Token.isTopLevelKeyword);
            },
        }
    }
}

pub const Value = union(enum) {
    false: void,
    float: []const u8,
    int: []const u8,
    true: void,
    string: []const u8,
};

fn parseNamespace(lexer: *Lexer, context: anytype) Error!void {
    const kw = lexer.next() orelse return error.NamespaceMissing;
    if (kw.kind != .kw_namespace) return error.NamespaceMissing;

    const eq = lexer.next() orelse return error.NamespaceMalformed;
    if (eq.kind != .eq) return error.NamespaceMalformed;

    const string = lexer.next() orelse return error.NamespaceMalformed;
    if (string.kind != .lit_string) return error.NamespaceMalformed;

    const semicolon = lexer.next() orelse return error.NamespaceMalformed;
    if (semicolon.kind != .semicolon) return error.NamespaceMalformed;

    context.namespace(lexer.yyinput[string.start..string.end]);
}

pub const Error = error{
    NamespaceMissing,
    NamespaceMalformed,
};

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

        eof,
        unknown,
    };

    kind: Kind,
    start: usize,
    end: usize,

    fn isTopLevelKeyword(self: Token) bool {
        return switch (self.kind) {
            .kw_linedef, .kw_sector, .kw_sidedef, .kw_thing, .kw_vertex => true,
            else => false,
        };
    }

    fn isRightBraceOrSemicolon(self: Token) bool {
        return switch (self.kind) {
            .brace_r, .semicolon => true,
            else => false,
        };
    }

    fn isRightBraceOrTopLevelKeyword(self: Token) bool {
        if (self.kind == .brace_r)
            return true
        else
            return self.isTopLevelKeyword();
    }
};

fn Parser(Context: type) type {
    return struct {
        const Self = @This();

        context: *Context,
        lexer: Lexer,
        buf: ?Token,

        fn advance(self: *Self) ?Token {
            if (self.buf) |token| {
                const ret = token;
                self.buf = self.lexer.next();
                return ret;
            } else {
                const ret = self.lexer.next();
                self.buf = self.lexer.next();
                return ret;
            }
        }

        fn expect(self: *Self, expected: []const Token.Kind) ?Token {
            if (self.advance()) |token| {
                _ = std.mem.indexOf(Token.Kind, expected, &.{token.kind}) orelse {
                    self.syntaxErr(token, expected);
                    return null;
                };

                return token;
            } else {
                self.syntaxErr(Token{
                    .kind = .eof,
                    .start = self.lexer.yyinput.len,
                    .end = self.lexer.yyinput.len,
                }, expected);

                return null;
            }
        }

        fn lexeme(self: *const Self, token: Token) []const u8 {
            return self.lexer.yyinput[token.start..token.end];
        }

        fn skipUntil(self: *Self, predicate: fn (Token) bool) void {
            while (true) {
                if (self.advance()) |token|
                    if (predicate(token)) return else return;
            }
        }

        fn syntaxErr(self: *Self, token: Token, expected: []const Token.Kind) void {
            if (std.meta.hasMethod(@TypeOf(self.context), "onParseError")) {
                self.context.onParseError(token, expected);
            }
        }

        fn lineDef(self: *Self) void {
            _ = self.expect(&.{.brace_l}) orelse {
                self.skipUntil(Token.isRightBraceOrTopLevelKeyword);
                return;
            };

            if (std.meta.hasMethod(Context, "onLineDefStart")) {
                self.context.onLineDefStart();
            }

            if (std.meta.hasMethod(Context, "perLineDefField")) {
                self.fields(Context.perLineDefField);
            }

            if (std.meta.hasMethod(Context, "onLineDefEnd")) {
                self.context.onLineDefEnd();
            }

            _ = self.expect(&.{.brace_r});
        }

        fn sector(self: *Self) void {
            _ = self.expect(&.{.brace_l}) orelse {
                self.skipUntil(Token.isRightBraceOrTopLevelKeyword);
                return;
            };

            if (std.meta.hasMethod(Context, "onSectorStart")) {
                self.context.onSectorStart();
            }

            if (std.meta.hasMethod(Context, "perSectorField")) {
                self.fields(Context.perSectorField);
            }

            if (std.meta.hasMethod(Context, "onSectorEnd")) {
                self.context.onSectorEnd();
            }

            _ = self.expect(&.{.brace_r});
        }

        fn sideDef(self: *Self) void {
            _ = self.expect(&.{.brace_l}) orelse {
                self.skipUntil(Token.isRightBraceOrTopLevelKeyword);
                return;
            };

            if (std.meta.hasMethod(Context, "onSideDefStart")) {
                self.context.onSideDefStart();
            }

            if (std.meta.hasMethod(Context, "perSideDefField")) {
                self.fields(Context.perSideDefField);
            }

            if (std.meta.hasMethod(Context, "onSideDefEnd")) {
                self.context.onSideDefEnd();
            }

            _ = self.expect(&.{.brace_r});
        }

        fn thing(self: *Self) void {
            _ = self.expect(&.{.brace_l}) orelse {
                self.skipUntil(Token.isRightBraceOrTopLevelKeyword);
                return;
            };

            if (std.meta.hasMethod(Context, "onThingStart")) {
                self.context.onThingStart();
            }

            if (std.meta.hasMethod(Context, "perThingField")) {
                self.fields(Context.perThingField);
            }

            if (std.meta.hasMethod(Context, "onThingEnd")) {
                self.context.onThingEnd();
            }

            _ = self.expect(&.{.brace_r});
        }

        fn vertex(self: *Self) void {
            _ = self.expect(&.{.brace_l}) orelse {
                self.skipUntil(Token.isRightBraceOrTopLevelKeyword);
                return;
            };

            if (std.meta.hasMethod(Context, "onVertexStart")) {
                self.context.onVertexStart();
            }

            if (std.meta.hasMethod(Context, "perVertexField")) {
                self.fields(Context.perVertexField);
            }

            if (std.meta.hasMethod(Context, "onVertexEnd")) {
                self.context.onVertexEnd();
            }

            _ = self.expect(&.{.brace_r});
        }

        fn fields(self: *Self, callback: fn (*Context, []const u8, Value) void) void {
            while (true) {
                if (self.buf) |ahead| {
                    switch (ahead.kind) {
                        .ident,
                        .kw_linedef,
                        .kw_sector,
                        .kw_sidedef,
                        .kw_thing,
                        .kw_vertex,
                        .kw_namespace,
                        => {},
                        else => break,
                    }
                } else break;

                const ident = self.expect(&.{.ident}) orelse {
                    self.skipUntil(Token.isRightBraceOrSemicolon);
                    continue;
                };

                _ = self.expect(&.{.eq}) orelse {
                    self.skipUntil(Token.isRightBraceOrSemicolon);
                    continue;
                };

                const expected = comptime &[_]Token.Kind{
                    .lit_int,
                    .lit_float,
                    .lit_string,
                    .kw_true,
                    .kw_false,
                };

                const val_token = self.expect(expected) orelse {
                    self.skipUntil(Token.isRightBraceOrSemicolon);
                    continue;
                };

                const val = switch (val_token.kind) {
                    .kw_true => Value{ .true = {} },
                    .kw_false => Value{ .false = {} },
                    .lit_float => Value{ .float = self.lexeme(val_token) },
                    .lit_int => Value{ .int = self.lexeme(val_token) },
                    .lit_string => Value{ .string = self.lexeme(val_token) },
                    else => unreachable,
                };

                callback(self.context, self.lexeme(ident), val);

                _ = self.expect(&.{.semicolon});
            }
        }
    };
}

/// This is deliberately public to act as a demonstration of what methods [`parse`]
/// checks for on its `context` parameter. All are optional, and it will also
/// never check for any fields or non-method declarations.
pub const TestContext = struct {
    const BlockSeen = struct {
        start: bool = false,
        innards: bool = false,
        end: bool = false,

        fn all(self: BlockSeen) bool {
            return self.start and self.innards and self.end;
        }
    };

    seen_namespace: bool = false,
    any_errors: bool = false,

    seen_linedef: BlockSeen = .{},
    seen_sector: BlockSeen = .{},
    seen_sidedef: BlockSeen = .{},
    seen_thing: BlockSeen = .{},
    seen_vertex: BlockSeen = .{},

    pub fn namespace(self: *TestContext, string: []const u8) void {
        self.seen_namespace = true;
        std.testing.expectEqualStrings("\"doom\"", string) catch unreachable;
    }

    pub fn onParseError(self: *TestContext, found: Token, expected: []const Token.Kind) void {
        _ = found;
        _ = expected;
        self.any_errors = true;
    }

    pub fn onLineDefStart(self: *TestContext) void {
        self.seen_linedef.start = true;
    }

    pub fn perLineDefField(self: *TestContext, key: []const u8, val: Value) void {
        self.seen_linedef.innards = true;
        std.testing.expectEqualStrings("on_the_block", key) catch unreachable;
        std.testing.expectEqual(val.true, {}) catch unreachable;
    }

    pub fn onLineDefEnd(self: *TestContext) void {
        self.seen_linedef.end = true;
    }

    pub fn onSectorStart(self: *TestContext) void {
        self.seen_sector.start = true;
    }

    pub fn perSectorField(self: *TestContext, key: []const u8, val: Value) void {
        self.seen_sector.innards = true;
        std.testing.expectEqualStrings("demolition", key) catch unreachable;
        std.testing.expectEqualStrings("\"zone\"", val.string) catch unreachable;
    }

    pub fn onSectorEnd(self: *TestContext) void {
        self.seen_sector.end = true;
    }

    pub fn onSideDefStart(self: *TestContext) void {
        self.seen_sidedef.start = true;
    }

    pub fn perSideDefField(self: *TestContext, key: []const u8, val: Value) void {
        self.seen_sidedef.innards = true;
        std.testing.expectEqualStrings("steep_drain", key) catch unreachable;
        std.testing.expectEqualStrings("0.9", val.float) catch unreachable;
    }

    pub fn onSideDefEnd(self: *TestContext) void {
        self.seen_sidedef.end = true;
    }

    pub fn onThingStart(self: *TestContext) void {
        self.seen_thing.start = true;
    }

    pub fn perThingField(self: *TestContext, key: []const u8, val: Value) void {
        self.seen_thing.innards = true;
        std.testing.expectEqualStrings("ground_beneath", key) catch unreachable;
        std.testing.expectEqual({}, val.false) catch unreachable;
    }

    pub fn onThingEnd(self: *TestContext) void {
        self.seen_thing.end = true;
    }

    pub fn onVertexStart(self: *TestContext) void {
        self.seen_vertex.start = true;
    }

    pub fn perVertexField(self: *TestContext, key: []const u8, val: Value) void {
        self.seen_vertex.innards = true;
        std.testing.expectEqualStrings("gateway", key) catch unreachable;
        std.testing.expectEqualStrings("8", val.int) catch unreachable;
    }

    pub fn onVertexEnd(self: *TestContext) void {
        self.seen_vertex.end = true;
    }
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

test "parser, smoke" {
    const sample =
        \\nAmespace = "doom" ;
        \\
        \\linedef { on_the_block = true; }
        \\sector{ demolition= "zone"; }
        \\sidedef  {steep_drain=0.9; }
        \\thing { ground_beneath=false;}
        \\vertex {gateway  = 8;}
        \\
    ;

    var context = TestContext{};
    try parse(sample, &context);

    try std.testing.expect(context.seen_namespace);
    try std.testing.expect(!context.any_errors);

    try std.testing.expect(context.seen_linedef.all());
    try std.testing.expect(context.seen_sector.all());
    try std.testing.expect(context.seen_sidedef.all());
    try std.testing.expect(context.seen_thing.all());
    try std.testing.expect(context.seen_vertex.all());
}
