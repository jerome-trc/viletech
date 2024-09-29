const std = @import("std");

const c = @import("tree-sitter.h.zig");

pub const version = c.TREE_SITTER_LANGUAGE_VERSION;

pub const StateId = u16;
pub const FieldId = u16;
pub const Symbol = u16;

pub fn Language(comptime lang: []const u8) type {
    return struct {
        const tsFn = @extern(
            *const fn () callconv(.C) *const c.TSLanguage,
            .{ .name = "tree_sitter_" ++ lang },
        );

        pub const Node = struct {
            pub fn Children(comptime which: enum { all, named }) type {
                return struct {
                    parent: *const Node,
                    pos: u32,

                    pub fn next(self: *@This()) ?Node {
                        const end = switch (which) {
                            .all => self.parent.childCount(),
                            .named => self.parent.namedChildCount(),
                        };

                        if (self.pos >= end) return null;
                        self.pos += 1;

                        return switch (which) {
                            .all => self.parent.child(self.pos - 1),
                            .named => self.parent.namedChild(self.pos - 1),
                        };
                    }

                    pub fn reset(self: *@This()) void {
                        self.pos = 0;
                    }
                };
            }

            inner: c.TSNode,

            pub fn child(self: Node, index: u32) ?Node {
                if (index >= self.childCount()) return null;
                return Node{ .inner = c.ts_node_child(self.inner, index) };
            }

            pub fn childCount(self: Node) u32 {
                return c.ts_node_child_count(self.inner);
            }

            pub fn children(self: *const Node) Children(.all) {
                return Children(.all){ .parent = self, .pos = 0 };
            }

            pub fn edit(self: *Node, input_edit: InputEdit) void {
                const e = c.TSInputEdit{
                    .start_byte = input_edit.start_byte,
                    .old_end_byte = input_edit.old_end_byte,
                    .new_end_byte = input_edit.new_end_byte,
                    .start_point = c.TSPoint{
                        .row = input_edit.start_point.row,
                        .column = input_edit.start_point.column,
                    },
                    .old_end_point = c.TSPoint{
                        .row = input_edit.old_end_point.row,
                        .column = input_edit.old_end_point.column,
                    },
                    .new_end_point = c.TSPoint{
                        .row = input_edit.new_end_point.row,
                        .column = input_edit.new_end_point.column,
                    },
                };

                c.ts_node_edit(&self.inner, &e);
            }

            pub fn endPoint(self: Node) Point {
                const pt = c.ts_node_end_point(self.inner);
                return Point{ .row = pt.row, .column = pt.column };
            }

            pub fn eq(self: Node, other: Node) bool {
                return c.ts_node_eq(self.inner, other.inner);
            }

            pub fn field(self: Node, name: []const u8) ?Node {
                const ret = c.ts_node_child_by_field_name(
                    self.inner,
                    name.ptr,
                    @truncate(name.len),
                );

                if (c.ts_node_is_null(ret)) return null;

                return Node{ .inner = ret };
            }

            pub fn grammarSymbol(self: Node) Symbol {
                return c.ts_node_grammar_symbol(self.inner);
            }

            pub fn hasChanges(self: Node) bool {
                return c.ts_node_has_changes(self.inner);
            }

            pub fn hasError(self: Node) bool {
                return c.ts_node_has_error(self.inner);
            }

            pub fn isError(self: Node) bool {
                return c.ts_node_is_error(self.inner);
            }

            pub fn isExtra(self: Node) bool {
                return c.ts_node_is_extra(self.inner);
            }

            pub fn isNamed(self: Node) bool {
                return c.ts_node_is_named(self.inner);
            }

            pub fn namedChild(self: Node, index: u32) ?Node {
                if (index >= self.namedChildCount()) return null;
                return Node{ .inner = c.ts_node_named_child(self.inner, index) };
            }

            pub fn namedChildCount(self: Node) u32 {
                return c.ts_node_named_child_count(self.inner);
            }

            pub fn namedChildren(self: *const Node) Children(.named) {
                return Children(.named){ .parent = self, .pos = 0 };
            }

            pub fn nextSibling(self: Node) ?Node {
                const s = c.ts_node_next_sibling(self.inner);
                if (c.ts_node_is_null(s)) return null;
                return Node{ .inner = s };
            }

            pub fn nextNamedSibling(self: Node) ?Node {
                const s = c.ts_node_next_named_sibling(self.inner);
                if (c.ts_node_is_null(s)) return null;
                return Node{ .inner = s };
            }

            pub fn slice(self: Node, source: []const u8) []const u8 {
                return source[c.ts_node_start_byte(self.inner)..c.ts_node_end_byte(self.inner)];
            }

            pub fn startPoint(self: Node) Point {
                const pt = c.ts_node_start_point(self.inner);
                return Point{ .row = pt.row, .column = pt.column };
            }

            pub fn symbol(self: Node) Symbol {
                return c.ts_node_symbol(self.inner);
            }

            pub fn sExpr(self: Node) []u8 {
                return std.mem.sliceTo(c.ts_node_string(self.inner), 0);
            }

            pub fn typeStr(self: Node) []const u8 {
                return std.mem.sliceTo(c.ts_node_type(self.inner), 0);
            }
        };

        pub const Parser = struct {
            inner: *c.TSParser,

            pub fn init() Error!Parser {
                const inner = c.ts_parser_new() orelse
                    return error.ParserCreateFail;

                if (!c.ts_parser_set_language(inner, tsFn()))
                    return error.LanguageSetFail;

                return Parser{ .inner = inner };
            }

            pub fn deinit(self: Parser) void {
                c.ts_parser_delete(self.inner);
            }

            pub fn reset(self: Parser) void {
                c.ts_parser_reset(self.inner);
            }

            pub fn parse(
                self: Parser,
                encoding: InputEncoding,
                context: anytype,
            ) Error!Tree {
                var shim = struct {
                    user_data: @TypeOf(context),

                    fn read(
                        this: ?*anyopaque,
                        byte_index: u32,
                        pos: c.TSPoint,
                        bytes_read: [*c]u32,
                    ) callconv(.C) [*c]const u8 {
                        var shim: *@This() = @alignCast(@ptrCast(this));
                        const point = Point{ .row = pos.row, .column = pos.column };
                        const slice: []const u8 = shim.user_data.read(byte_index, point);
                        bytes_read.* = @truncate(slice.len);
                        return slice.ptr;
                    }
                }{ .user_data = context };

                const in = c.TSInput{
                    .encoding = @intFromEnum(encoding),
                    .payload = &shim,
                    .read = @TypeOf(shim).read,
                };

                return Tree{
                    .inner = c.ts_parser_parse(self.inner, null, in) orelse return error.ParseFail,
                };
            }

            pub fn parseString(self: Parser, source: []const u8) Error!Tree {
                return Tree{
                    .inner = c.ts_parser_parse_string(
                        self.inner,
                        null,
                        source.ptr,
                        @truncate(source.len),
                    ) orelse return error.ParseFail,
                };
            }

            pub fn parseWithEncoding(
                self: Parser,
                source: []const u8,
                encoding: InputEncoding,
            ) Error!Tree {
                return Tree{
                    .inner = c.ts_parser_parse_string_encoding(
                        self.inner,
                        null,
                        source.ptr,
                        @truncate(source.len),
                        @intFromEnum(encoding),
                    ) orelse return error.ParseFail,
                };
            }

            pub fn printDotGraphs(self: Parser, file: std.c.fd_t) void {
                c.ts_parser_print_dot_graphs(self.inner, file);
            }

            pub fn setTimeout(self: Parser, microseconds: u64) void {
                c.ts_parser_set_timeout_micros(self.inner, microseconds);
            }

            pub fn setLoggerRaw(
                self: Parser,
                context: ?*anyopaque,
                log: *const fn (?*anyopaque, LogKind, [*:0]const u8) callconv(.C) void,
            ) void {
                c.ts_parser_set_logger(self.inner, c.TSLogger{
                    .payload = context,
                    .log = @ptrCast(log),
                });
            }

            pub fn clearLogger(self: Parser) void {
                c.ts_parser_set_logger(self.inner, c.TSLogger{ .payload = null, .log = null });
            }

            fn logShim(_: ?*anyopaque, kind: c.TSLogType, msg: [*c]const u8) callconv(.C) void {
                const kind_str = switch (kind) {
                    c.TSLogTypeLex => "lex",
                    c.TSLogTypeParse => "parse",
                    else => "",
                };

                std.debug.print("{s} - {s}\n", .{ kind_str, msg });
            }
        };

        pub const Tree = struct {
            inner: *c.TSTree,

            pub fn deinit(self: Tree) void {
                c.ts_tree_delete(self.inner);
            }

            /// `deinit` has to be called on the new tree.
            pub fn copy(self: Tree) std.mem.Allocator.Error!Tree {
                const ret = c.ts_tree_copy(self.inner) orelse return error.OutOfMemory;
                return Tree{ .inner = ret };
            }

            pub fn root(self: Tree) Node {
                return Node{ .inner = c.ts_tree_root_node(self.inner) };
            }
        };
    };
}

pub const InputEdit = struct {
    start_byte: u32,
    old_end_byte: u32,
    new_end_byte: u32,
    start_point: Point,
    old_end_point: Point,
    new_end_point: Point,
};

pub const InputEncoding = enum(c_uint) {
    utf8,
    utf16,
};

pub const LogKind = enum(c_uint) {
    lex,
    parse,
};

pub const Point = struct {
    row: u32,
    column: u32,
};

pub const Quantifier = enum(c_uint) {
    zero,
    zero_or_one,
    zero_or_more,
    one,
    one_or_more,
};

pub const Range = struct {
    start_point: Point,
    end_point: Point,
    start_byte: u32,
    end_byte: u32,
};

pub const Error = error{
    LanguageSetFail,
    ParseFail,
    ParserCreateFail,
};

pub const QueryError = error{
    Capture,
    Field,
    Language,
    NodeType,
    Structure,
    Syntax,
};

/// Given the contents of the `parser.c` file generated by Tree-sitter,
/// produces an enum with values corresponding to every named symbol in the grammar.
pub fn SymbolEnum(comptime parser_src: []const u8) type {
    @setEvalBranchQuota(8192);

    var buf: [1024]u8 = undefined;
    var fba = std.heap.FixedBufferAllocator.init(buf[0..]);
    const alloc = fba.allocator();

    var fields = std.BoundedArray(std.builtin.Type.EnumField, 64).init(0) catch unreachable;
    var lines = std.mem.splitAny(u8, parser_src, "\n");

    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");
        if (std.mem.startsWith(u8, trimmed, "enum {")) break;
    }

    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");
        if (std.mem.eql(u8, trimmed, "};")) break;
        var parts = std.mem.splitScalar(u8, trimmed, '=');
        const name = std.mem.trim(u8, parts.next().?, " \t");

        if (std.mem.indexOf(u8, name, "__") != null) continue;
        if (std.mem.indexOf(u8, name, "aux") != null) continue;
        if (std.mem.indexOf(u8, name, "anon") != null) continue;

        const val = std.mem.trim(u8, parts.next().?, " \t,");
        var name_parts = std.mem.splitSequence(u8, name, "sym_");
        _ = name_parts.next().?;
        const final_name = name_parts.next().?;
        // const final_name = std.mem.trimLeft(u8, name, "sym_");
        const name_z = alloc.dupeZ(u8, final_name) catch unreachable;

        fields.append(std.builtin.Type.EnumField{
            .name = name_z,
            .value = std.fmt.parseInt(u16, val, 0) catch unreachable,
        }) catch unreachable;
    }

    return @Type(std.builtin.Type{
        .Enum = std.builtin.Type.Enum{
            .tag_type = u16,
            .is_exhaustive = false,
            .decls = &.{},
            .fields = fields.slice(),
        },
    });
}
