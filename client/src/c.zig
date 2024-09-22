//! It's desirable to move as much C code as possible to Zig to minimize friction involved
//! in future changes (no need to write shims and C headers, et cetera), but per-file
//! is too coarse of a granularity to make the transition since:
//! - translating a file directly in a project with so much header pollutions creates
//! Zig output with multiple tens of thousands of lines of unused declarations
//! - some functions cannot be translated (goto, macros)
//! - some functions have multiple forms post-preprocessing, and so need manual fix-up
//! - some comments ought to be preserved

// The C-to-Zig translation pipeline is as follows:
//
// 1. Pass the file to `zig translate-c`.
// 2. From the output, extract the function and its static dependencies.
// (Remember to make the latter globally-visible and fix up C-side references.)
// 3. Move remaining dependencies from C to consts.zig, funcs.zig, structs.zig, and vars.zig.
// 4. Run demo-test suite to ensure no regressions have taken place.

pub const i_main = @import("c/i_main.c.zig");
