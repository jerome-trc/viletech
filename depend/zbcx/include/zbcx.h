#ifndef ZBCX_H_
#define ZBCX_H_

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>

#define ZBCX_DIAG_NONE 0
#define ZBCX_DIAG_FILE 0x1
#define ZBCX_DIAG_LINE 0x2
#define ZBCX_DIAG_COLUMN 0x4
#define ZBCX_DIAG_WARN 0x8
#define ZBCX_DIAG_ERR 0x10
#define ZBCX_DIAG_SYNTAX 0x20
#define ZBCX_DIAG_INTERNAL 0x40
#define ZBCX_DIAG_NOTE 0x80
#define ZBCX_DIAG_POS (ZBCX_DIAG_FILE | ZBCX_DIAG_LINE | ZBCX_DIAG_COLUMN)
#define ZBCX_DIAG_POS_ERR (ZBCX_DIAG_POS | ZBCX_DIAG_ERR)

#ifdef __cplusplus
extern "C" {
#endif

typedef enum _zbcx_Result {
	zbcx_res_ok,
	zbcx_res_nullsrc,
	zbcx_res_setjmpfail,
} zbcx_Result;

typedef struct _zbcx_Pos {
    int line;
    int column;
    int file_id;
} zbcx_Pos;

typedef struct _zbcx_IoVtable {
	/// A generic counterpart to libc's `fclose`.
	int (*close)(void* state);
	/// A generic counterpart to libc's `ferror`.
	int (*error)(void* state);
	/// A generic counterpart to libc's `fseek`.
	int (*seek)(void* state, long offset, int whence);
	/// A generic counterpart to libc's `fread`.
	unsigned long (*read)(void* dest, size_t size, size_t n, void* state);
	/// A generic counterpart to libc's `fwrite`.
	unsigned long (*write)(void* src, size_t size, size_t n, void* state);
} zbcx_IoVtable;

typedef struct _zbcx_Io {
	void* state;
	const zbcx_IoVtable* vtable;
} zbcx_Io;

typedef struct _zbcx_ListLink {
	struct _zbcx_ListLink* next;
	void* data;
} zbcx_ListLink;

typedef struct _zbcx_List {
	zbcx_ListLink* head;
	zbcx_ListLink* tail;
	int size;
} zbcx_List;

typedef struct _zbcx_ListIter {
	zbcx_ListLink* prev;
	zbcx_ListLink* link;
} zbcx_ListIter;

void zbcx_list_init(zbcx_List* list);
int zbcx_list_size(zbcx_List* list);
void* zbcx_list_head(zbcx_List* list);
void* zbcx_list_tail(zbcx_List* list);
void zbcx_list_append(zbcx_List*, void* data);
void zbcx_list_prepend(zbcx_List*, void* data);
void zbcx_list_iterate(const zbcx_List* list, zbcx_ListIter* iter);
bool zbcx_list_end(zbcx_ListIter* iter);
void zbcx_list_next(zbcx_ListIter* iter);
void* zbcx_list_data(zbcx_ListIter* iter);
void zbcx_list_insert_after(zbcx_List* list, zbcx_ListIter* iter, void* data);
void zbcx_list_insert_before(zbcx_List* list, zbcx_ListIter* iter, void* data);
/// Updates the data at the specified node and returns the old data.
void* zbcx_list_replace(zbcx_List* list, zbcx_ListIter* iter, void* data);
void zbcx_list_merge(zbcx_List* receiver, zbcx_List* giver);
/// Removes the first node of the list and returns the data of the removed node.
void* zbcx_list_shift(zbcx_List* list);
void zbcx_list_deinit(zbcx_List* list);

typedef struct _zbcx_Options {
	void* context;

	/// This is a path which gets passed to `zbcx_Options.fopen`.
	const char* source_file;

	zbcx_List includes;
	zbcx_List defines;
	zbcx_List library_links;
	int tab_size;
	bool acc_err;
	bool acc_stats;
	bool one_column;
	bool preprocess;
	bool write_asserts;
	bool slade_mode;

	/// The given `va_list` should not be freed by the caller.
	void (*diag)(void* context, int flags, va_list* args);

	char* (*realpath)(void* context, const char* path);

	bool (*fexists)(void* context, const char* path);

	/// A generic counterpart to libc's `fopen`.
	/// If the returned IO struct has a null `vtable`,
	/// this is considered equivalent to `fopen` returning a null `FILE*`.
	zbcx_Io (*fopen)(void* context, const char* filename, const char* modes);

	/// Where the finalized bytecode object will be written.
	zbcx_Io output;

	struct {
    	const char* dir_path;
    	int lifetime;
    	bool enable;
    	bool print;
    	bool clear;
	} cache;
} zbcx_Options;

zbcx_Options zbcx_options_init(void);
void zbcx_options_deinit(zbcx_Options*);

zbcx_Result zbcx_compile(const zbcx_Options*);

#ifdef __cplusplus
}
#endif

#endif // ifndef ZBCX_H_
