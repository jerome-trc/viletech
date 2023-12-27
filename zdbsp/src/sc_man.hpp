/// @file
/// @brief A subset of GZDoom's lexer.

#pragma once

#include <cstddef>

struct Scanner final {
	static constexpr size_t MAX_STRING_SIZE = 40960;

	char* string = NULL;
	int string_len = 0;
	int number = 0;
	double flnum = 0.0;
	int line = 0;
	bool end = false;
	bool crossed = false;
	bool file_scripts = false;
	bool string_quoted = false;
	char* scripts_dir = NULL;

	char* script_buf = NULL;
	char* script_ptr = NULL;
	char* script_end_ptr = NULL;
	char string_buf[MAX_STRING_SIZE];
	bool script_open = false;
	int script_size = 0;
	bool already_got = false;
	char* saved_script_ptr = NULL;
	int saved_script_line = 0;
	bool c_mode = false;

	void open(const char* name);
	void open_mem(const char* name, char* buffer, int size);
	void close();
	void set_c_mode(bool cmode);
	void save_pos();
	void restore_pos();
	bool get_string();
	void must_get_string();
	void must_get_string_name(const char* name);
	bool check_string(const char* name);
	bool get_number();
	void must_get_number();
	bool check_number();
	bool check_float();
	bool get_float();
	void must_get_float();
	void unget();
	bool compare(const char* text);
	int match_string(const char** strings);
	int must_match_string(const char** strings);
	void script_err(const char* message, ...);
	void prepare_script();
	void check_open();
};
