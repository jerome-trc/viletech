/// <reference types="tree-sitter-cli/dsl" />

module.exports = grammar({
    name: 'wadmerge',

    extras: $ => [/[^\r\n]/],

    rules: {
        source_file: $ => seq(repeat(seq($._newline, optional($._line)))),

        _line: $ => choice(
            $.create_statement,
            $.echo_statement,
            $.end_statement,
            $.line_comment,
        ),

        create_statement: $ =>
            seq(caseInsensitive('create'), $.symbol, /[^\r\n]*/),

        echo_statement: $ =>
            seq(
                caseInsensitive('echo'),
                token.immediate(repeat(/[ \t]/)),
                alias(/[^\r\n]*/, $.echo_text)
            ),

        end_statement: _ =>
            caseInsensitive('end'),

        symbol: $ =>
            choice(
                token(
                    seq(
                        '"',
                        repeat(/[^"\\\r\n]+/),
                        token.immediate('"')
                    ), // i.e. string literal
                ),
                /[^"\s]+/,
            ),

        line_comment: $ => token(seq('#', /[^\r\n]+/)),

        _newline: $ => choice('\n', '\r\n', '\r'),
    }
});

// Credit: https://github.com/stadelmanma/tree-sitter-fortran/blob/a9c79b2/grammar.js#L1125
function caseInsensitive(keyword, aliasAsWord = true) {
    let result = new RegExp(keyword
        .split('')
        .map(l => l !== l.toUpperCase() ? `[${l}${l.toUpperCase()}]` : l)
        .join('')
    );

    if (aliasAsWord) {
        result = alias(result, keyword);
    }

    return result
}
