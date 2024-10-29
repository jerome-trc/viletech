// re2zig --lang zig --api default -i $INPUT -o $OUTPUT
//
// A generated UDMF lexer.
// This is tracked as part of the Git repository so that building the rest of
// VileTech does not require a user to have the latest bleeding edge of re2zig.

// TODO: apply fuzzing when 0.14.0 lands, investigate places where runtime safety
// can be soundly disabled for performance gains.

%{
    re2c:define:YYCTYPE = u8;
    re2c:yyfill:enable = 0;
    re2c:eof = 0;
    re2c:api = custom;
    re2c:api:style = free-form;

    re2c:define:YYLESSTHAN = "self.yycursor >= self.yylimit";
    re2c:define:YYPEEK = "if (self.yycursor < self.yylimit) self.yyinput[self.yycursor] else 0";
    re2c:define:YYSKIP = "self.yycursor += 1;";
    re2c:define:YYBACKUP = "self.yymarker = self.yycursor;";
    re2c:define:YYRESTORE = "self.yycursor = self.yymarker;";
%}

const std = @import("std");
const udmf = @import("udmf.zig");

const Self = @This();

yyinput: []const u8,
yycursor: usize,
yymarker: usize,
yylimit: usize,
pos: usize,

pub fn init(textmap: []const u8) Self {
    return Self{
        .yyinput = textmap,
        .yycursor = 0,
        .yymarker = 0,
        .yylimit = textmap.len,
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

            * { return self.token(.unknown); }
            $ { return null; }
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
