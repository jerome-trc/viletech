const std = @import("std");

const c = @import("tree-sitter.h.zig");

pub fn Language(comptime lang: []const u8) type {
    return struct {
        const tsFn = @extern(
            *const fn () callconv(.C) *const c.TSLanguage,
            .{ .name = "tree_sitter_" ++ lang },
        );

        pub const Symbol = u16;

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

            pub fn child(self: *Node, index: u32) ?Node {
                if (index >= self.childCount()) return null;
                return Node{ .inner = c.ts_node_child(self.inner, index) };
            }

            pub fn childCount(self: *Node) u32 {
                return c.ts_node_child_count(self.inner);
            }

            pub fn children(self: *Node) Children(.all) {
                return Children(.all){ .parent = self, .pos = 0 };
            }

            pub fn edit(self: *Node, input_edit: InputEdit) void {
                const e = c.TSInputEdit{
                    .start_byte = input_edit.start_byte,
                    .old_end_byte = input_edit.old_end_byte,
                    .new_end_byte = input_edit.new_end_byte,
                    .start_point = input_edit.start_point,
                    .old_end_point = input_edit.old_end_point,
                    .new_end_point = input_edit.new_end_point,
                };

                c.ts_node_edit(self, &e);
            }

            pub fn grammarSymbol(self: Node) Symbol {
                return c.ts_node_grammar_symbol(self.inner);
            }

            pub fn namedChild(self: Node, index: u32) ?Node {
                if (index >= self.namedChildCount()) return null;
                return Node{ .inner = c.ts_node_named_child(self.inner, index) };
            }

            pub fn namedChildCount(self: Node) u32 {
                return c.ts_node_named_child_count(self.inner);
            }

            pub fn namedChildren(self: *Node) Children(.named) {
                return Children(.named){ .parent = self, .pos = 0 };
            }

            pub fn slice(self: Node, source: []const u8) []const u8 {
                return source[c.ts_node_start_byte(self.inner)..c.ts_node_end_byte(self.inner)];
            }

            pub fn symbol(self: Node) Symbol {
                return c.ts_node_symbol(self.inner);
            }

            pub fn sExpr(self: Node) []u8 {
                return std.mem.sliceTo(c.ts_node_string(self.inner), 0);
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
        };

        pub const Tree = struct {
            inner: *c.TSTree,

            pub fn deinit(self: Tree) void {
                c.ts_tree_delete(self.inner);
            }

            pub fn root(self: Tree) Node {
                return Node{ .inner = c.ts_tree_root_node(self.inner) };
            }
        };
    };
}

pub const Point = struct {
    row: u32,
    column: u32,
};

pub const InputEdit = struct {
    start_byte: u32,
    old_end_byte: u32,
    new_end_byte: u32,
    start_point: Point,
    old_end_point: Point,
    new_end_point: Point,
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
