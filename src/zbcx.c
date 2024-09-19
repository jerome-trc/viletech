/// @file
/// @brief Implements the public header.

#include "zbcx.h"

#include "cache/cache.h"
#include "codegen/phase.h"
#include "common.h"
#include "parse/phase.h"
#include "semantic/phase.h"
#include "task.h"

#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TAB_SIZE_MIN 1
#define TAB_SIZE_MAX 100

zbcx_Options zbcx_options_init(void) {
	zbcx_Options options = {0};
	// Default tab size for now is 4, since it's a common indentation size.
	options.tab_size = 4;
	options.write_asserts = true;
	options.cache.lifetime = -1;
	return options;
}

void zbcx_options_deinit(zbcx_Options* options) {
	zbcx_list_deinit(&options->includes);
	zbcx_list_deinit(&options->defines);
	zbcx_list_deinit(&options->library_links);
}

static const char* get_script_type_label(int type) {
	STATIC_ASSERT(SCRIPT_TYPE_NEXTFREENUMBER == SCRIPT_TYPE_REOPEN + 1);

	switch (type) {
	case SCRIPT_TYPE_CLOSED:
		return "closed";
	case SCRIPT_TYPE_OPEN:
		return "open";
	case SCRIPT_TYPE_RESPAWN:
		return "respawn";
	case SCRIPT_TYPE_DEATH:
		return "death";
	case SCRIPT_TYPE_ENTER:
		return "enter";
	case SCRIPT_TYPE_PICKUP:
		return "pickup";
	case SCRIPT_TYPE_BLUERETURN:
		return "bluereturn";
	case SCRIPT_TYPE_REDRETURN:
		return "redreturn";
	case SCRIPT_TYPE_WHITERETURN:
		return "whitereturn";
	case SCRIPT_TYPE_LIGHTNING:
		return "lightning";
	case SCRIPT_TYPE_UNLOADING:
		return "unloading";
	case SCRIPT_TYPE_DISCONNECT:
		return "disconnect";
	case SCRIPT_TYPE_RETURN:
		return "return";
	case SCRIPT_TYPE_EVENT:
		return "event";
	case SCRIPT_TYPE_KILL:
		return "kill";
	case SCRIPT_TYPE_REOPEN:
		return "reopen";
	default:
		return "";
	}
}

static void clear_cache(struct task* task, struct cache* cache) {
	if (cache) {
		cache_clear(cache);
	} else {
		t_diag(task, DIAG_ERR, "attempting to clear cache, but cache is not enabled");
		t_bail(task);
	}
}

static void preprocess(struct task* task) {
	struct parse parse;
	p_init(&parse, task, NULL);
	p_run(&parse);
}

static void compile_mainlib(struct task* task, struct cache* cache) {
	struct parse parse;
	p_init(&parse, task, cache);
	p_run(&parse);
	struct semantic semantic;
	s_init(&semantic, task);
	s_test(&semantic);
	struct codegen codegen;
	c_init(&codegen, task);
	c_publish(&codegen);
}

static void perform_selected_task(struct task* task, struct cache* cache) {
	if (task->options->cache.clear) {
		clear_cache(task, cache);
	} else if (task->options->preprocess) {
		preprocess(task);
	} else {
		compile_mainlib(task, cache);
	}
}

static void perform_task(struct task* task) {
	if (task->options->cache.enable) {
		struct cache cache;
		cache_init(&cache, task);
		cache_load(&cache);
		perform_selected_task(task, &cache);
		cache_close(&cache);
	} else {
		perform_selected_task(task, NULL);
	}
}

static bool perform_action(const zbcx_Options* options, jmp_buf* root_bail) {
	bool success = false;
	struct task task;
	t_init(&task, options, root_bail);
	jmp_buf bail;
	if (setjmp(bail) == 0) {
		task.bail = &bail;
		perform_task(&task);
		success = true;
	}
	task.bail = root_bail;
	t_deinit(&task);
	return success;
}

zbcx_Result zbcx_compile(const zbcx_Options* options) {
	mem_init(); // TODO: make this state local.
	jmp_buf bail;
	zbcx_Result res = zbcx_res_setjmpfail;

	if (setjmp(bail) == 0) {
		if (perform_action(options, &bail)) {
			res = zbcx_res_ok;
		}
	}

	mem_free_all();
	return res;
}
