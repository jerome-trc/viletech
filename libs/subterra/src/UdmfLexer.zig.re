// re2zig --lang zig --api default -i $INPUT -o $OUTPUT
//
// A generated UDMF lexer.
// This is tracked as part of the Git repository so that building the rest of
// VileTech does not require a user to have the latest bleeding edge of re2zig.

%{
    re2c:define:YYCTYPE = u8;
    re2c:yyfill:enable = 0;

    re2c:define:YYCURSOR = "self.yycursor";
    re2c:define:YYINPUT = "self.yyinput";
    re2c:define:YYLIMIT = "self.yylimit";
    re2c:define:YYMARKER = "self.yymarker";
%}

const std = @import("std");
const udmf = @import("udmf.zig");

const Self = @This();

yyinput: [:0]const u8,
yycursor: usize,
yymarker: usize,
yylimit: usize,
pos: usize,

pub fn init(textmap: [:0]const u8) Self {
    return Self{
        .yyinput = textmap,
        .yycursor = 0,
        .yymarker = 0,
        .yylimit = textmap.len - 1,
        .pos = 0,
    };
}

pub fn next(self: *Self) ?udmf.Token {
    outer: while (true) {
        /*!re2c
            "{" { return self.token(.brace_l); }
            "}" { return self.token(.brace_r); }
            "=" { return self.token(.eq); }
            ";" { return self.token(.semicolon); }

            'false'     { return self.token(.kw_false); }
            'linedef'   { return self.token(.kw_linedef); }
            'namespace' { return self.token(.kw_namespace); }
            'sector'    { return self.token(.kw_sector); }
            'sidedef'   { return self.token(.kw_sidedef); }
            'thing'     { return self.token(.kw_thing); }
            'true'      { return self.token(.kw_true); }
            'vertex'    { return self.token(.kw_vertex); }

            [+-]?[0-9]+[.][0-9]*([eE][+-]?[0-9]+)? {
                return self.token(.lit_float);
            }

            [+-]?[1-9]+[0-9]* | [0][0-9]+ | [0][x][0-9A-Fa-f]+ {
                return self.token(.lit_int);
            }

            ["] ([^\x00"\\]*([\\][^\x00][^\x00"\\]*)*) ["] {
                return self.token(.lit_string);
            }

            [A-Za-z_]+[A-Za-z0-9_]* {
                return self.token(.ident);
            }

            [\001- ]+ {
                self.pos = self.yycursor;
                continue :outer;
            }

            "/*" ([^\x00*] | ("*" [^\x00/]))* "*/" {
                self.pos = self.yycursor;
                continue :outer;
            }

            "//" [^\x00\n]* "\n" {
                self.pos = self.yycursor;
                continue :outer;
            }

            * { return null; }
        */
    }
}

fn token(self: *Self, kind: udmf.Token.Kind) udmf.Token {
    const start = self.pos;
    self.pos = self.yycursor;

    return udmf.Token{
        .kind = kind,
        .start = start,
        .end = self.yycursor,
    };
}
