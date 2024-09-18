//! A module that the other parts of this repository can import to get access
//! to embedded files without having to keep those files in their sub-directories.

pub const viletech_png = @embedFile("viletech.png");
pub const viletech_svg = @embedFile("viletech.svg");
