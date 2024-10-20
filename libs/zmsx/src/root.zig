//! # ZMSX
//!
//! A Zig wrapper around [ZMSX], a fork of (G)ZDoom's [ZMusic] library.
//!
//! [ZMSX]: https://github.com/jerome-trc/zmsx
//! [ZMusic]: https://github.com/ZDoom/ZMusic

const std = @import("std");

const c = @cImport(@cInclude("zmsx.h"));

test "smoke" {
    var out_count: c_int = 0;
    const devices = c.zmsx_get_midi_devices(&out_count);
    try std.testing.expect(devices != null);
    try std.testing.expect(out_count > 0);
}
