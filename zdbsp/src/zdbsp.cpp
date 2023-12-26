/// @file
/// @brief Bridge between the C interface and the internal C++.

#include "zdbsp.h"

#include "common.hpp"
#include "processor.hpp"
#include "wad.hpp"

#define ZDBSP_VERSION "1.19"

zdbsp_WadReaderPtr zdbsp_wadreader_new(const uint8_t* bytes) {
	auto wad = std::make_unique<FWadReader>(bytes);
	return wad.release();
}

void zdbsp_wadreader_destroy(zdbsp_WadReaderPtr wad) {
	auto w = std::unique_ptr<FWadReader>(wad);
}

zdbsp_ProcessorPtr zdbsp_processor_new(zdbsp_WadReaderPtr wad, const zdbsp_ProcessConfig* const config) {
	auto p = std::make_unique<FProcessor>(*wad, 0);

	if (config != nullptr) {
		p->blockmap_mode = config->blockmap_mode;
		p->reject_mode = config->reject_mode;

		p->build_nodes = (config->flags & ZDBSP_PROCF_BUILDNODES);
		p->conform_nodes = (config->flags & ZDBSP_PROCF_CONFORMNODES);
		p->no_prune = (config->flags & ZDBSP_PROCF_NOPRUNE);
		p->check_poly_objs = (config->flags & ZDBSP_PROCF_CHECKPOLYOBJS);
		p->build_gl_nodes = (config->flags & ZDBSP_PROCF_BUILDGLNODES);
		p->gl_only = (config->flags & ZDBSP_PROCF_GLONLY);
		p->v5gl = (config->flags & ZDBSP_PROCF_V5GL);
		p->write_comments = (config->flags & ZDBSP_PROCF_WRITECOMMENTS);
		p->compress_nodes = (config->flags & ZDBSP_PROCF_COMPRESSNODES);
		p->compress_gl_nodes = (config->flags & ZDBSP_PROCF_COMPRESSGLNODES);
		p->force_compression = (config->flags & ZDBSP_PROCF_FORCECOMPRESSION);
	}

	return p.release();
}

void zdbsp_processor_run(zdbsp_ProcessorPtr p, const zdbsp_NodeConfig* const config) {
	p->Process(config);
}

size_t zdbsp_processor_nodesx_count(zdbsp_ProcessorPtr p) {
	return p->get_level().NumNodes;
}

void zdbsp_processor_nodes_foreach(zdbsp_ProcessorPtr p, void *ctx, zdbsp_NodeVisitor callback) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumNodes; ++i) {
		zdbsp_MapNode node = {0};
		node.x = LittleShort(level.Nodes[i].x >> 16);
		node.y = LittleShort(level.Nodes[i].y >> 16);

		callback(ctx, &node);
	}
}

void zdbsp_processor_nodesx_foreach(zdbsp_ProcessorPtr p, void *ctx, zdbsp_NodeExVisitor callback) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumNodes; ++i) {
		callback(ctx, &level.Nodes[i]);
	}
}

void zdbsp_processor_destroy(zdbsp_ProcessorPtr p) {
	auto w = std::unique_ptr<FProcessor>(p);
}
