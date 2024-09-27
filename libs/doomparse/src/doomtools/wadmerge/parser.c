#include "doomtools/wadmerge/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 15
#define LARGE_STATE_COUNT 3
#define SYMBOL_COUNT 20
#define ALIAS_COUNT 1
#define TOKEN_COUNT 12
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 0
#define MAX_ALIAS_SEQUENCE_LENGTH 3
#define PRODUCTION_ID_COUNT 2

enum {
  aux_sym_create_statement_token1 = 1,
  aux_sym_create_statement_token2 = 2,
  aux_sym_echo_statement_token1 = 3,
  aux_sym_echo_statement_token2 = 4,
  aux_sym_end_statement_token1 = 5,
  aux_sym_symbol_token1 = 6,
  aux_sym_symbol_token2 = 7,
  sym_line_comment = 8,
  anon_sym_LF = 9,
  anon_sym_CR_LF = 10,
  anon_sym_CR = 11,
  sym_source_file = 12,
  sym__line = 13,
  sym_create_statement = 14,
  sym_echo_statement = 15,
  sym_end_statement = 16,
  sym_symbol = 17,
  sym__newline = 18,
  aux_sym_source_file_repeat1 = 19,
  alias_sym_echo_text = 20,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [aux_sym_create_statement_token1] = "create",
  [aux_sym_create_statement_token2] = "create_statement_token2",
  [aux_sym_echo_statement_token1] = "echo",
  [aux_sym_echo_statement_token2] = "echo_statement_token2",
  [aux_sym_end_statement_token1] = "end",
  [aux_sym_symbol_token1] = "symbol_token1",
  [aux_sym_symbol_token2] = "symbol_token2",
  [sym_line_comment] = "line_comment",
  [anon_sym_LF] = "\n",
  [anon_sym_CR_LF] = "\r\n",
  [anon_sym_CR] = "\r",
  [sym_source_file] = "source_file",
  [sym__line] = "_line",
  [sym_create_statement] = "create_statement",
  [sym_echo_statement] = "echo_statement",
  [sym_end_statement] = "end_statement",
  [sym_symbol] = "symbol",
  [sym__newline] = "_newline",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [alias_sym_echo_text] = "echo_text",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [aux_sym_create_statement_token1] = aux_sym_create_statement_token1,
  [aux_sym_create_statement_token2] = aux_sym_create_statement_token2,
  [aux_sym_echo_statement_token1] = aux_sym_echo_statement_token1,
  [aux_sym_echo_statement_token2] = aux_sym_echo_statement_token2,
  [aux_sym_end_statement_token1] = aux_sym_end_statement_token1,
  [aux_sym_symbol_token1] = aux_sym_symbol_token1,
  [aux_sym_symbol_token2] = aux_sym_symbol_token2,
  [sym_line_comment] = sym_line_comment,
  [anon_sym_LF] = anon_sym_LF,
  [anon_sym_CR_LF] = anon_sym_CR_LF,
  [anon_sym_CR] = anon_sym_CR,
  [sym_source_file] = sym_source_file,
  [sym__line] = sym__line,
  [sym_create_statement] = sym_create_statement,
  [sym_echo_statement] = sym_echo_statement,
  [sym_end_statement] = sym_end_statement,
  [sym_symbol] = sym_symbol,
  [sym__newline] = sym__newline,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [alias_sym_echo_text] = alias_sym_echo_text,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [aux_sym_create_statement_token1] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_create_statement_token2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_echo_statement_token1] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_echo_statement_token2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_end_statement_token1] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_symbol_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_symbol_token2] = {
    .visible = false,
    .named = false,
  },
  [sym_line_comment] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_LF] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_CR_LF] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_CR] = {
    .visible = true,
    .named = false,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym__line] = {
    .visible = false,
    .named = true,
  },
  [sym_create_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_echo_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_end_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_symbol] = {
    .visible = true,
    .named = true,
  },
  [sym__newline] = {
    .visible = false,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [alias_sym_echo_text] = {
    .visible = true,
    .named = true,
  },
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
  [1] = {
    [2] = alias_sym_echo_text,
  },
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(15);
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead != 0) SKIP(0)
      END_STATE();
    case 1:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'A' ||
          lookahead == 'a') ADVANCE(6);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(3);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead == 'N' ||
          lookahead == 'n') ADVANCE(2);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 2:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'D' ||
          lookahead == 'd') ADVANCE(22);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 3:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead == 'H' ||
          lookahead == 'h') ADVANCE(4);
      if (lookahead == 'R' ||
          lookahead == 'r') ADVANCE(8);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 4:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead == 'O' ||
          lookahead == 'o') ADVANCE(19);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 5:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead == 'R' ||
          lookahead == 'r') ADVANCE(8);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 6:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead == 'T' ||
          lookahead == 't') ADVANCE(9);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 7:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 8:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(1);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 9:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(16);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 10:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(3);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead == 'N' ||
          lookahead == 'n') ADVANCE(2);
      if (lookahead != 0) SKIP(7)
      END_STATE();
    case 11:
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead != 0) ADVANCE(33);
      END_STATE();
    case 12:
      if (lookahead == '"') ADVANCE(13);
      if (lookahead == '\t' ||
          lookahead == ' ') SKIP(12)
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(25);
      END_STATE();
    case 13:
      if (lookahead == '"') ADVANCE(24);
      if (lookahead == '\\') ADVANCE(25);
      if (lookahead == '\t' ||
          lookahead == ' ') ADVANCE(13);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(26);
      END_STATE();
    case 14:
      if (eof) ADVANCE(15);
      if (lookahead == '\n') ADVANCE(37);
      if (lookahead == '\r') ADVANCE(39);
      if (lookahead != 0) SKIP(14)
      END_STATE();
    case 15:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 16:
      ACCEPT_TOKEN(aux_sym_create_statement_token1);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(3);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      if (lookahead == 'N' ||
          lookahead == 'n') ADVANCE(2);
      END_STATE();
    case 17:
      ACCEPT_TOKEN(aux_sym_create_statement_token1);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(29);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead == 'N' ||
          lookahead == 'n') ADVANCE(28);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 18:
      ACCEPT_TOKEN(aux_sym_create_statement_token2);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(18);
      END_STATE();
    case 19:
      ACCEPT_TOKEN(aux_sym_echo_statement_token1);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      END_STATE();
    case 20:
      ACCEPT_TOKEN(aux_sym_echo_statement_token1);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 21:
      ACCEPT_TOKEN(aux_sym_echo_statement_token2);
      if (lookahead == '\t' ||
          lookahead == ' ') ADVANCE(21);
      END_STATE();
    case 22:
      ACCEPT_TOKEN(aux_sym_end_statement_token1);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(5);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(10);
      END_STATE();
    case 23:
      ACCEPT_TOKEN(aux_sym_end_statement_token1);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 24:
      ACCEPT_TOKEN(aux_sym_symbol_token1);
      if (lookahead == '"') ADVANCE(24);
      if (lookahead == '\\') ADVANCE(25);
      if (lookahead == '\t' ||
          lookahead == ' ') ADVANCE(13);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(26);
      END_STATE();
    case 25:
      ACCEPT_TOKEN(aux_sym_symbol_token2);
      if (lookahead == '"') ADVANCE(13);
      if (lookahead != 0 &&
          lookahead != '\t' &&
          lookahead != '\n' &&
          lookahead != '\r' &&
          lookahead != ' ') ADVANCE(25);
      END_STATE();
    case 26:
      ACCEPT_TOKEN(aux_sym_symbol_token2);
      if (lookahead == '"') ADVANCE(24);
      if (lookahead == '\\') ADVANCE(25);
      if (lookahead == '\t' ||
          lookahead == ' ') ADVANCE(13);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(26);
      END_STATE();
    case 27:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'A' ||
          lookahead == 'a') ADVANCE(32);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(29);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead == 'N' ||
          lookahead == 'n') ADVANCE(28);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'D' ||
          lookahead == 'd') ADVANCE(23);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead == 'H' ||
          lookahead == 'h') ADVANCE(30);
      if (lookahead == 'R' ||
          lookahead == 'r') ADVANCE(34);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead == 'O' ||
          lookahead == 'o') ADVANCE(20);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead == 'R' ||
          lookahead == 'r') ADVANCE(34);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead == 'T' ||
          lookahead == 't') ADVANCE(35);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(27);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(31);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(17);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead == '#') ADVANCE(33);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(29);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(36);
      if (lookahead == 'N' ||
          lookahead == 'n') ADVANCE(28);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r') ADVANCE(33);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_LF);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_CR_LF);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_CR);
      if (lookahead == '\n') ADVANCE(38);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 14},
  [2] = {.lex_state = 0},
  [3] = {.lex_state = 14},
  [4] = {.lex_state = 14},
  [5] = {.lex_state = 14},
  [6] = {.lex_state = 14},
  [7] = {.lex_state = 14},
  [8] = {.lex_state = 14},
  [9] = {.lex_state = 12},
  [10] = {.lex_state = 0},
  [11] = {.lex_state = 21},
  [12] = {.lex_state = 18},
  [13] = {.lex_state = 18},
  [14] = {.lex_state = 18},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [aux_sym_create_statement_token1] = ACTIONS(1),
    [aux_sym_echo_statement_token1] = ACTIONS(1),
    [aux_sym_end_statement_token1] = ACTIONS(1),
    [sym_line_comment] = ACTIONS(1),
    [anon_sym_LF] = ACTIONS(1),
    [anon_sym_CR_LF] = ACTIONS(1),
    [anon_sym_CR] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(10),
    [sym__newline] = STATE(2),
    [aux_sym_source_file_repeat1] = STATE(3),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_LF] = ACTIONS(5),
    [anon_sym_CR_LF] = ACTIONS(5),
    [anon_sym_CR] = ACTIONS(7),
  },
  [2] = {
    [sym__line] = STATE(6),
    [sym_create_statement] = STATE(6),
    [sym_echo_statement] = STATE(6),
    [sym_end_statement] = STATE(6),
    [ts_builtin_sym_end] = ACTIONS(9),
    [aux_sym_create_statement_token1] = ACTIONS(11),
    [aux_sym_echo_statement_token1] = ACTIONS(13),
    [aux_sym_end_statement_token1] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(17),
    [anon_sym_LF] = ACTIONS(19),
    [anon_sym_CR_LF] = ACTIONS(19),
    [anon_sym_CR] = ACTIONS(19),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 5,
    ACTIONS(7), 1,
      anon_sym_CR,
    ACTIONS(21), 1,
      ts_builtin_sym_end,
    STATE(2), 1,
      sym__newline,
    STATE(4), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(5), 2,
      anon_sym_LF,
      anon_sym_CR_LF,
  [17] = 5,
    ACTIONS(23), 1,
      ts_builtin_sym_end,
    ACTIONS(28), 1,
      anon_sym_CR,
    STATE(2), 1,
      sym__newline,
    STATE(4), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(25), 2,
      anon_sym_LF,
      anon_sym_CR_LF,
  [34] = 2,
    ACTIONS(33), 1,
      anon_sym_CR,
    ACTIONS(31), 3,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_CR_LF,
  [43] = 2,
    ACTIONS(35), 1,
      anon_sym_CR,
    ACTIONS(23), 3,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_CR_LF,
  [52] = 2,
    ACTIONS(39), 1,
      anon_sym_CR,
    ACTIONS(37), 3,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_CR_LF,
  [61] = 2,
    ACTIONS(43), 1,
      anon_sym_CR,
    ACTIONS(41), 3,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_CR_LF,
  [70] = 2,
    STATE(13), 1,
      sym_symbol,
    ACTIONS(45), 2,
      aux_sym_symbol_token1,
      aux_sym_symbol_token2,
  [78] = 1,
    ACTIONS(47), 1,
      ts_builtin_sym_end,
  [82] = 1,
    ACTIONS(49), 1,
      aux_sym_echo_statement_token2,
  [86] = 1,
    ACTIONS(51), 1,
      aux_sym_create_statement_token2,
  [90] = 1,
    ACTIONS(53), 1,
      aux_sym_create_statement_token2,
  [94] = 1,
    ACTIONS(55), 1,
      aux_sym_create_statement_token2,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(3)] = 0,
  [SMALL_STATE(4)] = 17,
  [SMALL_STATE(5)] = 34,
  [SMALL_STATE(6)] = 43,
  [SMALL_STATE(7)] = 52,
  [SMALL_STATE(8)] = 61,
  [SMALL_STATE(9)] = 70,
  [SMALL_STATE(10)] = 78,
  [SMALL_STATE(11)] = 82,
  [SMALL_STATE(12)] = 86,
  [SMALL_STATE(13)] = 90,
  [SMALL_STATE(14)] = 94,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [9] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 1),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(9),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [17] = {.entry = {.count = 1, .reusable = false}}, SHIFT(6),
  [19] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 1),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1),
  [23] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2),
  [25] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(2),
  [28] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(2),
  [31] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_end_statement, 1),
  [33] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_end_statement, 1),
  [35] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2),
  [37] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_create_statement, 3),
  [39] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_create_statement, 3),
  [41] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_echo_statement, 3, .production_id = 1),
  [43] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_echo_statement, 3, .production_id = 1),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [47] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [49] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [51] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_symbol, 1),
  [53] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [55] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef _WIN32
#define extern __declspec(dllexport)
#endif

extern const TSLanguage *tree_sitter_wadmerge(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
