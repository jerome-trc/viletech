/// <reference types="tree-sitter-cli/dsl" />

module.exports = grammar({
    name: 'wadmerge',

    // https://docs.oracle.com/javase/8/docs/api/java/lang/Character.html#isWhitespace-char-
    // https://en.wikipedia.org/wiki/Whitespace_character#Unicode
    extras: $ => [
        /[ \t\v\f]/,
        /[\u001C-\u001F\u2000-\u2006\u2008-\u200a]/,
        /[\u0085\u1680\u2028\u2029\u205F\u3000]/,
    ],

    word: $ => $._ident,

    rules: {
        source_file: $ => seq(
            optional($._line),
            repeat(seq($._newline, optional($._line)))
        ),

        _line: $ => choice(
            $.clear_command,
            $.create_command,
            $.echo_command,
            $.end_command,
            $.line_comment,
        ),

        clear_command: $ =>
            seq(caseInsensitive('clear'), field('symbol', $.symbol), $._trailer),

        create_command: $ =>
            seq(
                caseInsensitive('create'),
                field('symbol', $.symbol),
                optional(field('iwad_qual', caseInsensitive('iwad'))),
                repeat($._trailer),
            ),

        echo_command: $ =>
            seq(
                caseInsensitive('echo'),
                token.immediate(/[ \t\u000B\u001C-\u001F]/),
                alias(repeat(/[^\s]+/), $.echo_text)
            ),

        end_command: $ =>
            seq(caseInsensitive('end'), repeat($._trailer)),

        symbol: $ =>
            choice($._string_literal, $._ident),

        _ident: _ =>
            /[^"\s]+/,

        _string_literal: _ =>
            token(
                seq(
                    '"',
                    repeat(choice(
                      /[^\\"]/,
                      /\\./
                    )),
                    '"'
                ), // i.e. string literal
            ),

        _trailer: _ =>
            /[^\s]+/,

        line_comment: _ => token(seq('#', /[^\r\n]+/)),

        _newline: _ => choice('\n', '\r\n', '\r'),
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
