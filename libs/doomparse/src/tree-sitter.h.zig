//! This only needs to be tracked since otherwise compilation fails due to
//! `@compileError`s left by translate-c which the bindings don't even need.

const std = @import("std");

pub const TSStateId = u16;
pub const TSSymbol = u16;
pub const TSFieldId = u16;
pub const struct_TSLanguage = opaque {};
pub const TSLanguage = struct_TSLanguage;
pub const struct_TSParser = opaque {};
pub const TSParser = struct_TSParser;
pub const struct_TSTree = opaque {};
pub const TSTree = struct_TSTree;
pub const struct_TSQuery = opaque {};
pub const TSQuery = struct_TSQuery;
pub const struct_TSQueryCursor = opaque {};
pub const TSQueryCursor = struct_TSQueryCursor;
pub const struct_TSLookaheadIterator = opaque {};
pub const TSLookaheadIterator = struct_TSLookaheadIterator;
pub const TSInputEncodingUTF8: c_int = 0;
pub const TSInputEncodingUTF16: c_int = 1;
pub const enum_TSInputEncoding = c_uint;
pub const TSInputEncoding = enum_TSInputEncoding;
pub const TSSymbolTypeRegular: c_int = 0;
pub const TSSymbolTypeAnonymous: c_int = 1;
pub const TSSymbolTypeAuxiliary: c_int = 2;
pub const enum_TSSymbolType = c_uint;
pub const TSSymbolType = enum_TSSymbolType;
pub const struct_TSPoint = extern struct {
    row: u32 = std.mem.zeroes(u32),
    column: u32 = std.mem.zeroes(u32),
};
pub const TSPoint = struct_TSPoint;
pub const struct_TSRange = extern struct {
    start_point: TSPoint = std.mem.zeroes(TSPoint),
    end_point: TSPoint = std.mem.zeroes(TSPoint),
    start_byte: u32 = std.mem.zeroes(u32),
    end_byte: u32 = std.mem.zeroes(u32),
};
pub const TSRange = struct_TSRange;
pub const struct_TSInput = extern struct {
    payload: ?*anyopaque = std.mem.zeroes(?*anyopaque),
    read: ?*const fn (
        ?*anyopaque,
        u32,
        TSPoint,
        [*c]u32,
    ) callconv(.C) [*c]const u8 = std.mem.zeroes(
        ?*const fn (?*anyopaque, u32, TSPoint, [*c]u32) callconv(.C) [*c]const u8,
    ),
    encoding: TSInputEncoding = std.mem.zeroes(TSInputEncoding),
};
pub const TSInput = struct_TSInput;
pub const TSLogTypeParse: c_int = 0;
pub const TSLogTypeLex: c_int = 1;
pub const enum_TSLogType = c_uint;
pub const TSLogType = enum_TSLogType;
pub const struct_TSLogger = extern struct {
    payload: ?*anyopaque = std.mem.zeroes(?*anyopaque),
    log: ?*const fn (
        ?*anyopaque,
        TSLogType,
        [*c]const u8,
    ) callconv(.C) void = std.mem.zeroes(
        ?*const fn (?*anyopaque, TSLogType, [*c]const u8) callconv(.C) void,
    ),
};
pub const TSLogger = struct_TSLogger;
pub const struct_TSInputEdit = extern struct {
    start_byte: u32 = std.mem.zeroes(u32),
    old_end_byte: u32 = std.mem.zeroes(u32),
    new_end_byte: u32 = std.mem.zeroes(u32),
    start_point: TSPoint = std.mem.zeroes(TSPoint),
    old_end_point: TSPoint = std.mem.zeroes(TSPoint),
    new_end_point: TSPoint = std.mem.zeroes(TSPoint),
};
pub const TSInputEdit = struct_TSInputEdit;
pub const struct_TSNode = extern struct {
    context: [4]u32 = std.mem.zeroes([4]u32),
    id: ?*const anyopaque = std.mem.zeroes(?*const anyopaque),
    tree: ?*const TSTree = std.mem.zeroes(?*const TSTree),
};
pub const TSNode = struct_TSNode;
pub const struct_TSTreeCursor = extern struct {
    tree: ?*const anyopaque = std.mem.zeroes(?*const anyopaque),
    id: ?*const anyopaque = std.mem.zeroes(?*const anyopaque),
    context: [3]u32 = std.mem.zeroes([3]u32),
};
pub const TSTreeCursor = struct_TSTreeCursor;
pub const struct_TSQueryCapture = extern struct {
    node: TSNode = std.mem.zeroes(TSNode),
    index: u32 = std.mem.zeroes(u32),
};
pub const TSQueryCapture = struct_TSQueryCapture;
pub const TSQuantifierZero: c_int = 0;
pub const TSQuantifierZeroOrOne: c_int = 1;
pub const TSQuantifierZeroOrMore: c_int = 2;
pub const TSQuantifierOne: c_int = 3;
pub const TSQuantifierOneOrMore: c_int = 4;
pub const enum_TSQuantifier = c_uint;
pub const TSQuantifier = enum_TSQuantifier;
pub const struct_TSQueryMatch = extern struct {
    id: u32 = std.mem.zeroes(u32),
    pattern_index: u16 = std.mem.zeroes(u16),
    capture_count: u16 = std.mem.zeroes(u16),
    captures: [*c]const TSQueryCapture = std.mem.zeroes([*c]const TSQueryCapture),
};
pub const TSQueryMatch = struct_TSQueryMatch;
pub const TSQueryPredicateStepTypeDone: c_int = 0;
pub const TSQueryPredicateStepTypeCapture: c_int = 1;
pub const TSQueryPredicateStepTypeString: c_int = 2;
pub const enum_TSQueryPredicateStepType = c_uint;
pub const TSQueryPredicateStepType = enum_TSQueryPredicateStepType;
pub const struct_TSQueryPredicateStep = extern struct {
    type: TSQueryPredicateStepType = std.mem.zeroes(TSQueryPredicateStepType),
    value_id: u32 = std.mem.zeroes(u32),
};
pub const TSQueryPredicateStep = struct_TSQueryPredicateStep;
pub const TSQueryErrorNone: c_int = 0;
pub const TSQueryErrorSyntax: c_int = 1;
pub const TSQueryErrorNodeType: c_int = 2;
pub const TSQueryErrorField: c_int = 3;
pub const TSQueryErrorCapture: c_int = 4;
pub const TSQueryErrorStructure: c_int = 5;
pub const TSQueryErrorLanguage: c_int = 6;
pub const enum_TSQueryError = c_uint;
pub const TSQueryError = enum_TSQueryError;
pub extern fn ts_parser_new() ?*TSParser;
pub extern fn ts_parser_delete(self: ?*TSParser) void;
pub extern fn ts_parser_language(self: ?*const TSParser) ?*const TSLanguage;
pub extern fn ts_parser_set_language(self: ?*TSParser, language: ?*const TSLanguage) bool;
pub extern fn ts_parser_set_included_ranges(self: ?*TSParser, ranges: [*c]const TSRange, count: u32) bool;
pub extern fn ts_parser_included_ranges(self: ?*const TSParser, count: [*c]u32) [*c]const TSRange;
pub extern fn ts_parser_parse(self: ?*TSParser, old_tree: ?*const TSTree, input: TSInput) ?*TSTree;
pub extern fn ts_parser_parse_string(
    self: ?*TSParser,
    old_tree: ?*const TSTree,
    string: [*c]const u8,
    length: u32,
) ?*TSTree;
pub extern fn ts_parser_parse_string_encoding(
    self: ?*TSParser,
    old_tree: ?*const TSTree,
    string: [*c]const u8,
    length: u32,
    encoding: TSInputEncoding,
) ?*TSTree;
pub extern fn ts_parser_reset(self: ?*TSParser) void;
pub extern fn ts_parser_set_timeout_micros(self: ?*TSParser, timeout_micros: u64) void;
pub extern fn ts_parser_timeout_micros(self: ?*const TSParser) u64;
pub extern fn ts_parser_set_cancellation_flag(self: ?*TSParser, flag: [*c]const usize) void;
pub extern fn ts_parser_cancellation_flag(self: ?*const TSParser) [*c]const usize;
pub extern fn ts_parser_set_logger(self: ?*TSParser, logger: TSLogger) void;
pub extern fn ts_parser_logger(self: ?*const TSParser) TSLogger;
pub extern fn ts_parser_print_dot_graphs(self: ?*TSParser, fd: c_int) void;
pub extern fn ts_tree_copy(self: ?*const TSTree) ?*TSTree;
pub extern fn ts_tree_delete(self: ?*TSTree) void;
pub extern fn ts_tree_root_node(self: ?*const TSTree) TSNode;
pub extern fn ts_tree_root_node_with_offset(
    self: ?*const TSTree,
    offset_bytes: u32,
    offset_extent: TSPoint,
) TSNode;
pub extern fn ts_tree_language(self: ?*const TSTree) ?*const TSLanguage;
pub extern fn ts_tree_included_ranges(self: ?*const TSTree, length: [*c]u32) [*c]TSRange;
pub extern fn ts_tree_edit(self: ?*TSTree, edit: [*c]const TSInputEdit) void;
pub extern fn ts_tree_get_changed_ranges(
    old_tree: ?*const TSTree,
    new_tree: ?*const TSTree,
    length: [*c]u32,
) [*c]TSRange;
pub extern fn ts_tree_print_dot_graph(self: ?*const TSTree, file_descriptor: c_int) void;
pub extern fn ts_node_type(self: TSNode) [*c]const u8;
pub extern fn ts_node_symbol(self: TSNode) TSSymbol;
pub extern fn ts_node_language(self: TSNode) ?*const TSLanguage;
pub extern fn ts_node_grammar_type(self: TSNode) [*c]const u8;
pub extern fn ts_node_grammar_symbol(self: TSNode) TSSymbol;
pub extern fn ts_node_start_byte(self: TSNode) u32;
pub extern fn ts_node_start_point(self: TSNode) TSPoint;
pub extern fn ts_node_end_byte(self: TSNode) u32;
pub extern fn ts_node_end_point(self: TSNode) TSPoint;
pub extern fn ts_node_string(self: TSNode) [*c]u8;
pub extern fn ts_node_is_null(self: TSNode) bool;
pub extern fn ts_node_is_named(self: TSNode) bool;
pub extern fn ts_node_is_missing(self: TSNode) bool;
pub extern fn ts_node_is_extra(self: TSNode) bool;
pub extern fn ts_node_has_changes(self: TSNode) bool;
pub extern fn ts_node_has_error(self: TSNode) bool;
pub extern fn ts_node_is_error(self: TSNode) bool;
pub extern fn ts_node_parse_state(self: TSNode) TSStateId;
pub extern fn ts_node_next_parse_state(self: TSNode) TSStateId;
pub extern fn ts_node_parent(self: TSNode) TSNode;
pub extern fn ts_node_child_containing_descendant(self: TSNode, descendant: TSNode) TSNode;
pub extern fn ts_node_child(self: TSNode, child_index: u32) TSNode;
pub extern fn ts_node_field_name_for_child(self: TSNode, child_index: u32) [*c]const u8;
pub extern fn ts_node_child_count(self: TSNode) u32;
pub extern fn ts_node_named_child(self: TSNode, child_index: u32) TSNode;
pub extern fn ts_node_named_child_count(self: TSNode) u32;
pub extern fn ts_node_child_by_field_name(self: TSNode, name: [*c]const u8, name_length: u32) TSNode;
pub extern fn ts_node_child_by_field_id(self: TSNode, field_id: TSFieldId) TSNode;
pub extern fn ts_node_next_sibling(self: TSNode) TSNode;
pub extern fn ts_node_prev_sibling(self: TSNode) TSNode;
pub extern fn ts_node_next_named_sibling(self: TSNode) TSNode;
pub extern fn ts_node_prev_named_sibling(self: TSNode) TSNode;
pub extern fn ts_node_first_child_for_byte(self: TSNode, byte: u32) TSNode;
pub extern fn ts_node_first_named_child_for_byte(self: TSNode, byte: u32) TSNode;
pub extern fn ts_node_descendant_count(self: TSNode) u32;
pub extern fn ts_node_descendant_for_byte_range(self: TSNode, start: u32, end: u32) TSNode;
pub extern fn ts_node_descendant_for_point_range(self: TSNode, start: TSPoint, end: TSPoint) TSNode;
pub extern fn ts_node_named_descendant_for_byte_range(self: TSNode, start: u32, end: u32) TSNode;
pub extern fn ts_node_named_descendant_for_point_range(self: TSNode, start: TSPoint, end: TSPoint) TSNode;
pub extern fn ts_node_edit(self: [*c]TSNode, edit: [*c]const TSInputEdit) void;
pub extern fn ts_node_eq(self: TSNode, other: TSNode) bool;
pub extern fn ts_tree_cursor_new(node: TSNode) TSTreeCursor;
pub extern fn ts_tree_cursor_delete(self: [*c]TSTreeCursor) void;
pub extern fn ts_tree_cursor_reset(self: [*c]TSTreeCursor, node: TSNode) void;
pub extern fn ts_tree_cursor_reset_to(dst: [*c]TSTreeCursor, src: [*c]const TSTreeCursor) void;
pub extern fn ts_tree_cursor_current_node(self: [*c]const TSTreeCursor) TSNode;
pub extern fn ts_tree_cursor_current_field_name(self: [*c]const TSTreeCursor) [*c]const u8;
pub extern fn ts_tree_cursor_current_field_id(self: [*c]const TSTreeCursor) TSFieldId;
pub extern fn ts_tree_cursor_goto_parent(self: [*c]TSTreeCursor) bool;
pub extern fn ts_tree_cursor_goto_next_sibling(self: [*c]TSTreeCursor) bool;
pub extern fn ts_tree_cursor_goto_previous_sibling(self: [*c]TSTreeCursor) bool;
pub extern fn ts_tree_cursor_goto_first_child(self: [*c]TSTreeCursor) bool;
pub extern fn ts_tree_cursor_goto_last_child(self: [*c]TSTreeCursor) bool;
pub extern fn ts_tree_cursor_goto_descendant(self: [*c]TSTreeCursor, goal_descendant_index: u32) void;
pub extern fn ts_tree_cursor_current_descendant_index(self: [*c]const TSTreeCursor) u32;
pub extern fn ts_tree_cursor_current_depth(self: [*c]const TSTreeCursor) u32;
pub extern fn ts_tree_cursor_goto_first_child_for_byte(self: [*c]TSTreeCursor, goal_byte: u32) i64;
pub extern fn ts_tree_cursor_goto_first_child_for_point(self: [*c]TSTreeCursor, goal_point: TSPoint) i64;
pub extern fn ts_tree_cursor_copy(cursor: [*c]const TSTreeCursor) TSTreeCursor;
pub extern fn ts_query_new(
    language: ?*const TSLanguage,
    source: [*c]const u8,
    source_len: u32,
    error_offset: [*c]u32,
    error_type: [*c]TSQueryError,
) ?*TSQuery;
pub extern fn ts_query_delete(self: ?*TSQuery) void;
pub extern fn ts_query_pattern_count(self: ?*const TSQuery) u32;
pub extern fn ts_query_capture_count(self: ?*const TSQuery) u32;
pub extern fn ts_query_string_count(self: ?*const TSQuery) u32;
pub extern fn ts_query_start_byte_for_pattern(self: ?*const TSQuery, pattern_index: u32) u32;
pub extern fn ts_query_end_byte_for_pattern(self: ?*const TSQuery, pattern_index: u32) u32;
pub extern fn ts_query_predicates_for_pattern(
    self: ?*const TSQuery,
    pattern_index: u32,
    step_count: [*c]u32,
) [*c]const TSQueryPredicateStep;
pub extern fn ts_query_is_pattern_rooted(self: ?*const TSQuery, pattern_index: u32) bool;
pub extern fn ts_query_is_pattern_non_local(self: ?*const TSQuery, pattern_index: u32) bool;
pub extern fn ts_query_is_pattern_guaranteed_at_step(self: ?*const TSQuery, byte_offset: u32) bool;
pub extern fn ts_query_capture_name_for_id(self: ?*const TSQuery, index: u32, length: [*c]u32) [*c]const u8;
pub extern fn ts_query_capture_quantifier_for_id(
    self: ?*const TSQuery,
    pattern_index: u32,
    capture_index: u32,
) TSQuantifier;
pub extern fn ts_query_string_value_for_id(self: ?*const TSQuery, index: u32, length: [*c]u32) [*c]const u8;
pub extern fn ts_query_disable_capture(self: ?*TSQuery, name: [*c]const u8, length: u32) void;
pub extern fn ts_query_disable_pattern(self: ?*TSQuery, pattern_index: u32) void;
pub extern fn ts_query_cursor_new() ?*TSQueryCursor;
pub extern fn ts_query_cursor_delete(self: ?*TSQueryCursor) void;
pub extern fn ts_query_cursor_exec(self: ?*TSQueryCursor, query: ?*const TSQuery, node: TSNode) void;
pub extern fn ts_query_cursor_did_exceed_match_limit(self: ?*const TSQueryCursor) bool;
pub extern fn ts_query_cursor_match_limit(self: ?*const TSQueryCursor) u32;
pub extern fn ts_query_cursor_set_match_limit(self: ?*TSQueryCursor, limit: u32) void;
pub extern fn ts_query_cursor_set_byte_range(self: ?*TSQueryCursor, start_byte: u32, end_byte: u32) void;
pub extern fn ts_query_cursor_set_point_range(self: ?*TSQueryCursor, start_point: TSPoint, end_point: TSPoint) void;
pub extern fn ts_query_cursor_next_match(self: ?*TSQueryCursor, match: [*c]TSQueryMatch) bool;
pub extern fn ts_query_cursor_remove_match(self: ?*TSQueryCursor, match_id: u32) void;
pub extern fn ts_query_cursor_next_capture(
    self: ?*TSQueryCursor,
    match: [*c]TSQueryMatch,
    capture_index: [*c]u32,
) bool;
pub extern fn ts_query_cursor_set_max_start_depth(self: ?*TSQueryCursor, max_start_depth: u32) void;
pub extern fn ts_language_copy(self: ?*const TSLanguage) ?*const TSLanguage;
pub extern fn ts_language_delete(self: ?*const TSLanguage) void;
pub extern fn ts_language_symbol_count(self: ?*const TSLanguage) u32;
pub extern fn ts_language_state_count(self: ?*const TSLanguage) u32;
pub extern fn ts_language_symbol_name(self: ?*const TSLanguage, symbol: TSSymbol) [*c]const u8;
pub extern fn ts_language_symbol_for_name(
    self: ?*const TSLanguage,
    string: [*c]const u8,
    length: u32,
    is_named: bool,
) TSSymbol;
pub extern fn ts_language_field_count(self: ?*const TSLanguage) u32;
pub extern fn ts_language_field_name_for_id(
    self: ?*const TSLanguage,
    id: TSFieldId,
) [*c]const u8;
pub extern fn ts_language_field_id_for_name(
    self: ?*const TSLanguage,
    name: [*c]const u8,
    name_length: u32,
) TSFieldId;
pub extern fn ts_language_symbol_type(self: ?*const TSLanguage, symbol: TSSymbol) TSSymbolType;
pub extern fn ts_language_version(self: ?*const TSLanguage) u32;
pub extern fn ts_language_next_state(
    self: ?*const TSLanguage,
    state: TSStateId,
    symbol: TSSymbol,
) TSStateId;
pub extern fn ts_lookahead_iterator_new(self: ?*const TSLanguage, state: TSStateId) ?*TSLookaheadIterator;
pub extern fn ts_lookahead_iterator_delete(self: ?*TSLookaheadIterator) void;
pub extern fn ts_lookahead_iterator_reset_state(self: ?*TSLookaheadIterator, state: TSStateId) bool;
pub extern fn ts_lookahead_iterator_reset(self: ?*TSLookaheadIterator, language: ?*const TSLanguage, state: TSStateId) bool;
pub extern fn ts_lookahead_iterator_language(self: ?*const TSLookaheadIterator) ?*const TSLanguage;
pub extern fn ts_lookahead_iterator_next(self: ?*TSLookaheadIterator) bool;
pub extern fn ts_lookahead_iterator_current_symbol(self: ?*const TSLookaheadIterator) TSSymbol;
pub extern fn ts_lookahead_iterator_current_symbol_name(
    self: ?*const TSLookaheadIterator,
) [*c]const u8;
pub const struct_wasm_engine_t = opaque {};
pub const TSWasmEngine = struct_wasm_engine_t;
pub const struct_TSWasmStore = opaque {};
pub const TSWasmStore = struct_TSWasmStore;
pub const TSWasmErrorKindNone: c_int = 0;
pub const TSWasmErrorKindParse: c_int = 1;
pub const TSWasmErrorKindCompile: c_int = 2;
pub const TSWasmErrorKindInstantiate: c_int = 3;
pub const TSWasmErrorKindAllocate: c_int = 4;
pub const TSWasmErrorKind = c_uint;
pub const TSWasmError = extern struct {
    kind: TSWasmErrorKind = std.mem.zeroes(TSWasmErrorKind),
    message: [*c]u8 = std.mem.zeroes([*c]u8),
};
pub extern fn ts_wasm_store_new(engine: ?*TSWasmEngine, @"error": [*c]TSWasmError) ?*TSWasmStore;
pub extern fn ts_wasm_store_delete(?*TSWasmStore) void;
pub extern fn ts_wasm_store_load_language(
    ?*TSWasmStore,
    name: [*c]const u8,
    wasm: [*c]const u8,
    wasm_len: u32,
    @"error": [*c]TSWasmError,
) ?*const TSLanguage;
pub extern fn ts_wasm_store_language_count(?*const TSWasmStore) usize;
pub extern fn ts_language_is_wasm(?*const TSLanguage) bool;
pub extern fn ts_parser_set_wasm_store(?*TSParser, ?*TSWasmStore) void;
pub extern fn ts_parser_take_wasm_store(?*TSParser) ?*TSWasmStore;
pub extern fn ts_set_allocator(
    new_malloc: ?*const fn (usize) callconv(.C) ?*anyopaque,
    new_calloc: ?*const fn (usize, usize) callconv(.C) ?*anyopaque,
    new_realloc: ?*const fn (?*anyopaque, usize) callconv(.C) ?*anyopaque,
    new_free: ?*const fn (?*anyopaque) callconv(.C) void,
) void;

pub const TREE_SITTER_LANGUAGE_VERSION = @as(c_int, 14);
pub const TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION = @as(c_int, 13);
