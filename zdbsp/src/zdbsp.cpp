/// @file
/// @brief Bridge between the C interface and the internal C++.

#include "zdbsp.h"

#include "common.hpp"
#include "processor.hpp"
#include "wad.hpp"

// Upstream ZDBSP aliased `DWORD` to `unsigned long` on win32 and `uint32_t`
// everywhere else. These check that using fixed-width types everywhere is sound.
static_assert(sizeof(unsigned char) == sizeof(uint8_t));
static_assert(sizeof(unsigned short) == sizeof(uint16_t));
static_assert(sizeof(unsigned int) == sizeof(uint32_t));

zdbsp_WadReaderPtr zdbsp_wadreader_new(const uint8_t* bytes) {
	auto wad = std::make_unique<FWadReader>(bytes);
	return wad.release();
}

void zdbsp_wadreader_destroy(zdbsp_WadReaderPtr wad) {
	auto w = std::unique_ptr<FWadReader>(wad);
}

zdbsp_ProcessorPtr zdbsp_processor_new(
	zdbsp_WadReaderPtr wad, const zdbsp_ProcessConfig* const config
) {
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

void zdbsp_processor_nodes_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor callback) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumNodes; ++i) {
		zdbsp_NodeRaw node = {};
		node.x = LittleShort(level.Nodes[i].x >> 16);
		node.y = LittleShort(level.Nodes[i].y >> 16);
		node.dx = LittleShort(level.Nodes[i].dx >> 16);
		node.dy = LittleShort(level.Nodes[i].dy >> 16);

		for (size_t ii = 0; ii < 2; ++ii) {
			for (size_t iii = 0; iii < 4; ++iii) {
				node.bbox[ii][iii] = LittleShort(level.Nodes[i].bbox[ii][iii]);
			}
		}

		auto o = reinterpret_cast<int16_t*>(&node.children[0]);

		for (size_t ii = 0; ii < 2; ++ii) {
			uint32_t child = level.Nodes[i].children[ii];

			if (child & NFX_SUBSECTOR) {
				*o++ = LittleShort(uint16_t(child - (NFX_SUBSECTOR + NF_SUBSECTOR)));
			} else {
				*o++ = LittleShort((uint16_t)child);
			}
		}

		// uint32_t child0 = level.Nodes[i].children[0];

		// if (child0 & NFX_SUBSECTOR) {
		// 	node.children[0] = LittleShort(uint16_t(child0 - (NFX_SUBSECTOR + NF_SUBSECTOR)));
		// } else {
		// 	node.children[0] = LittleShort((uint16_t)child0);
		// }

		// uint32_t child1 = level.Nodes[i].children[1];

		// if (child0 & NFX_SUBSECTOR) {
		// 	node.children[1] = LittleShort(uint16_t(child1 - (NFX_SUBSECTOR + NF_SUBSECTOR)));
		// } else {
		// 	node.children[1] = LittleShort((uint16_t)child1);
		// }

		callback(ctx, &node);
	}
}

void zdbsp_processor_nodesx_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExVisitor callback) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumNodes; ++i) {
		callback(ctx, &level.Nodes[i]);
	}
}

void zdbsp_processor_glnodes_foreach(
	zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLNodes; ++i) {
		callback(ctx, &level.GLNodes[i]);
	}
}

void zdbsp_processor_destroy(zdbsp_ProcessorPtr p) {
	auto w = std::unique_ptr<FProcessor>(p);
}
