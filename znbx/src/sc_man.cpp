/// @file
/// @brief A subset of GZDoom's lexer.

/*

Copyright (C) 1996 Raven Software
Copyright (C) 2002-2006 Randy Heit

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

*/

#include <cassert>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>
#include <limits.h>

#include "sc_man.hpp"

#ifdef _MSC_VER
#pragma warning(disable : 4996)
#endif

static constexpr char ASCII_COMMENT = ';';
static constexpr char ASCII_QUOTE = 34;
static constexpr char C_COMMENT = '*';
static constexpr char CPP_COMMENT = '/';

void Scanner::open_mem([[maybe_unused]] const char* name, char* buffer, int len) {
	this->close();
	this->script_size = len;
	this->script_buf = buffer;
	this->prepare_script();
}

void Scanner::prepare_script() {
	this->script_ptr = this->script_buf;
	this->script_end_ptr = this->script_ptr + this->script_size;
	this->line = 1;
	this->end = false;
	this->script_open = true;
	this->string = this->string_buf;
	this->already_got = false;
	this->saved_script_ptr = NULL;
	this->c_mode = false;
}

void Scanner::close() {
	if (this->script_open) {
		this->script_buf = NULL;
		this->script_open = false;
	}
}

void Scanner::save_pos() {
this->check_open();

	if (this->end) {
		this->saved_script_ptr = NULL;
	} else {
		this->saved_script_ptr = this->script_ptr;
		this->saved_script_line = this->line;
	}
}

void Scanner::restore_pos() {
	if (this->saved_script_ptr) {
		this->script_ptr = this->saved_script_ptr;
		this->line = this->saved_script_line;
		this->end = false;
		this->already_got = false;
	}
}

/// Enables/disables C mode. In C mode, more characters are considered to
/// be whole words than in non-C mode.
void Scanner::set_c_mode(bool cmode) {
	this->c_mode = cmode;
}

bool Scanner::get_string() {
	char* text;
	bool foundToken;

this->check_open();
	if (this->already_got) {
		this->already_got = false;
		return true;
	}
	foundToken = false;
	this->crossed = false;
	this->string_quoted = false;
	if (this->script_ptr >= this->script_end_ptr) {
		this->end = true;
		return false;
	}
	while (foundToken == false) {
		while (this->script_ptr < this->script_end_ptr && *this->script_ptr <= ' ') {
			if (*this->script_ptr++ == '\n') {
				this->line++;
				this->crossed = true;
			}
		}
		if (this->script_ptr >= this->script_end_ptr) {
			this->end = true;
			return false;
		}
		if ((this->c_mode || *this->script_ptr != ASCII_COMMENT) &&
			!(this->script_ptr[0] == CPP_COMMENT && this->script_ptr < this->script_end_ptr - 1 &&
			  (this->script_ptr[1] == CPP_COMMENT || this->script_ptr[1] == C_COMMENT))) { // Found a token
			foundToken = true;
		} else { // Skip comment
			if (this->script_ptr[0] == CPP_COMMENT && this->script_ptr[1] == C_COMMENT) { // C comment
				while (this->script_ptr[0] != C_COMMENT || this->script_ptr[1] != CPP_COMMENT) {
					if (this->script_ptr[0] == '\n') {
						this->line++;
						this->crossed = true;
					}
					//					fputc(ScriptPtr[0], this->sc_Out);
					this->script_ptr++;
					if (this->script_ptr >= this->script_end_ptr - 1) {
						this->end = true;
						return false;
					}
					//					fputs("*/", this->sc_Out);
				}
				this->script_ptr += 2;
			} else { // C++ comment
				while (*this->script_ptr++ != '\n') {
					//					fputc(ScriptPtr[-1], this->sc_Out);
					if (this->script_ptr >= this->script_end_ptr) {
						this->end = true;
						return false;
					}
				}
				this->line++;
				this->crossed = true;
				//				fputc('\n', this->sc_Out);
			}
		}
	}
	text = this->string;
	if (*this->script_ptr == ASCII_QUOTE) { // Quoted string - return string including the quotes
		*text++ = *this->script_ptr++;
		this->string_quoted = true;
		while (*this->script_ptr != ASCII_QUOTE) {
			if (*this->script_ptr >= 0 && *this->script_ptr < ' ') {
				this->script_ptr++;
			} else if (*this->script_ptr == '\\') {
				// Add the backslash character and the following chararcter to the text.
				// We do not translate the escape sequence in any way, since the only
				// thing that will happen to this string is that it will be written back
				// out to disk. Basically, we just need this special case here so that
				// string reading won't terminate prematurely when a \" sequence is
				// used to embed a quote mark in the string.
				*text++ = *this->script_ptr++;
				*text++ = *this->script_ptr++;
			} else {
				*text++ = *this->script_ptr++;
			}
			if (this->script_ptr == this->script_end_ptr || text == &this->string[MAX_STRING_SIZE - 1]) {
				break;
			}
		}
		*text++ = '"';
		this->script_ptr++;
	} else { // Normal string
		static const char* stopchars;

		if (this->c_mode) {
			stopchars = "`~!@#$%^&*(){}[]/=\?+|;:<>,";

			// '-' can be its own token, or it can be part of a negative number
			if (*this->script_ptr == '-') {
				*text++ = '-';
				this->script_ptr++;
				if (this->script_ptr < this->script_end_ptr || (*this->script_ptr >= '0' && *this->script_ptr <= '9')) {
					goto grabtoken;
				}
				goto gottoken;
			}
		} else {
			stopchars = "{}|=";
		}
		if (strchr(stopchars, *this->script_ptr)) {
			*text++ = *this->script_ptr++;
		} else {
grabtoken:
			while ((*this->script_ptr > ' ') && (strchr(stopchars, *this->script_ptr) == NULL) &&
				   (this->c_mode || *this->script_ptr != ASCII_COMMENT) &&
				   !(this->script_ptr[0] == CPP_COMMENT && (this->script_ptr < this->script_end_ptr - 1) &&
					 (this->script_ptr[1] == CPP_COMMENT || this->script_ptr[1] == C_COMMENT))) {
				*text++ = *this->script_ptr++;
				if (this->script_ptr == this->script_end_ptr || text == &this->string[MAX_STRING_SIZE - 1]) {
					break;
				}
			}
		}
	}
gottoken:
	*text = 0;
	this->string_len = int(text - this->string);
	return true;
}

void Scanner::must_get_string() {
	if (this->get_string() == false) {
		this->script_err("Missing string (unexpected end of file).");
	}
}

void Scanner::must_get_string_name(const char* name) {
	this->must_get_string();
	if (this->compare(name) == false) {
		this->script_err("Expected '%s', got '%s'.", name, this->string);
	}
}

/// Checks if the next token matches the specified string. Returns true if
/// it does. If it doesn't, it ungets it and returns false.
bool Scanner::check_string(const char* name) {
	if (this->get_string()) {
		if (this->compare(name)) {
			return true;
		}
		this->unget();
	}
	return false;
}

bool Scanner::get_number() {
	char* stopper;

	this->check_open();
	if (this->get_string()) {
		if (strcmp(this->string, "MAXINT") == 0) {
			this->number = INT_MAX;
		} else {
			this->number = strtol(this->string, &stopper, 0);

			if (*stopper != 0) {
				this->script_err("SC_GetNumber: Bad numeric constant \"%s\".", this->string);
			}
		}
		this->flnum = this->number;
		return true;
	} else {
		return false;
	}
}

void Scanner::must_get_number() {
	if (this->get_number() == false) {
		this->script_err("Missing integer (unexpected end of file).");
	}
}

/// Similar to SC_GetNumber but ungets the token if it isn't a number
/// and does not print an error.
bool Scanner::check_number() {
	char* stopper;

	// CheckOpen ();
	if (this->get_string()) {
		if (strcmp(this->string, "MAXINT") == 0) {
			this->number = INT_MAX;
		} else {
			this->number = strtol(this->string, &stopper, 0);

			if (*stopper != 0) {
				this->unget();
				return false;
			}
		}
		this->flnum = this->number;
		return true;
	} else {
		return false;
	}
}

/// [GRB] Same as SC_CheckNumber, only for floats.
bool Scanner::check_float() {
	char* stopper;

	// CheckOpen ();
	if (this->get_string()) {
		this->flnum = strtod(this->string, &stopper);
		this->number = (int)this->flnum;
		if (*stopper != 0) {
			this->unget();
			return false;
		}
		return true;
	} else {
		return false;
	}
}

bool Scanner::get_float() {
	char* stopper;

	this->check_open();
	if (this->get_string()) {
		this->flnum = strtod(this->string, &stopper);
		if (*stopper != 0) {
			this->script_err("SC_GetFloat: Bad numeric constant \"%s\".\n", this->string);
		}
		this->number = (int)this->flnum;
		return true;
	} else {
		return false;
	}
}

void Scanner::must_get_float() {
	if (this->get_float() == false) {
		this->script_err("Missing floating-point number (unexpected end of file).");
	}
}

/// Assumes there is a valid string in `this->sc_String`.
void Scanner::unget() {
	this->already_got = true;
}

/// Returns the index of the first match to this->sc_String from the passed
/// array of strings, or -1 if not found.
int Scanner::match_string(const char** strings) {
	int i;

	for (i = 0; *strings != NULL; i++) {
		if (this->compare(*strings++)) {
			return i;
		}
	}
	return -1;
}

int Scanner::must_match_string(const char** strings) {
	int i;

	i = Scanner::match_string(strings);
	if (i == -1) {
		this->script_err(NULL);
	}
	return i;
}

bool Scanner::compare(const char* text) {
#ifdef _MSC_VER
	return (_stricmp(text, this->sc_String) == 0);
#else
	return (strcasecmp(text, this->string) == 0);
#endif
}

void Scanner::script_err(const char* message, ...) {
	char composed[2048];
	if (message == NULL) {
		message = "Bad syntax.";
	}

	va_list arglist;
	va_start(arglist, message);
	vsprintf(composed, message, arglist);
	va_end(arglist);

	printf("Script error, line %d:\n%s\n", this->line, composed);
	exit(1);
}

void Scanner::check_open() {
	assert(this->script_open && "SC_ call before SC_Open().\n");
}
