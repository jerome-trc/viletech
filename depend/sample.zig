//! Embeds files from dependencies to use for automated testing.

pub const zbcx = struct {
    pub const stack = @embedFile("zbcx/test/stack.bcs");
    pub const zcommon = @embedFile("zbcx/lib/zcommon.bcs");
    pub const zcommon_h = @embedFile("zbcx/lib/zcommon.h.bcs");
};
