/// @file
/// @brief Reads wad files, builds nodes, and saves new wad files.

/*

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

#include "processor.hpp"
#include "blockmapbuilder.hpp"
#include "wad.hpp"
#include "znbx.h"
#include <cstring>

enum {
	// Thing numbers used in Hexen maps
	PO_HEX_ANCHOR_TYPE = 3000,
	PO_HEX_SPAWN_TYPE,
	PO_HEX_SPAWNCRUSH_TYPE,

	// Thing numbers used in Doom and Heretic maps
	PO_ANCHOR_TYPE = 9300,
	PO_SPAWN_TYPE,
	PO_SPAWNCRUSH_TYPE,
	PO_SPAWNHURT_TYPE
};

FLevel::FLevel() {
	memset(this, 0, sizeof(*this));
}

FLevel::~FLevel() {
	if (Vertices)
		delete[] Vertices;
	if (Subsectors)
		delete[] Subsectors;
	if (Segs)
		delete[] Segs;
	if (Nodes)
		delete[] Nodes;
	if (Blockmap)
		delete[] Blockmap;
	if (Reject)
		delete[] Reject;
	if (GLSubsectors)
		delete[] GLSubsectors;
	if (GLSegs)
		delete[] GLSegs;
	if (GLNodes)
		delete[] GLNodes;
	if (GLPVS)
		delete[] GLPVS;
	if (OrgSectorMap)
		delete[] OrgSectorMap;
}

FProcessor::FProcessor(znbx_Level level, bool extended) {
	this->is_udmf = false;

	if (!extended) {
		this->is_extended = false;

		this->load_things(level.things);
		this->load_vertices(level.vertices);
		this->load_lines(level.linedefs);
		this->load_sides(level.sidedefs);
		this->load_sectors(level.sectors);
	} else {
		this->is_extended = true;

		this->load_things_ext(level.things);
		this->load_vertices(level.vertices);
		this->load_lines_ext(level.linedefs);
		this->load_sides(level.sidedefs);
		this->load_sectors(level.sectors);
	}

	strcpy(this->level_name, level.name);
	this->finish_load();
}

FProcessor::FProcessor(znbx_LevelUdmf level) {
	this->is_udmf = true;
	strcpy(this->level_name, level.name);
	this->ParseTextMap(level.textmap);
	this->finish_load();
}

void FProcessor::load_things(znbx_SliceU8 slice) {
	znbx_ThingRaw* mt;
	int32_t thing_count;
	read_lump(slice, mt, thing_count);

	this->Level.Things.Resize(thing_count);

	for (uint32_t i = 0; i < thing_count; ++i) {
		this->Level.Things[i].x = little_short(mt[i].x) << FRACBITS;
		this->Level.Things[i].y = little_short(mt[i].y) << FRACBITS;
		this->Level.Things[i].angle = little_short(mt[i].angle);
		this->Level.Things[i].type = little_short(mt[i].type);
		this->Level.Things[i].flags = little_short(mt[i].flags);
		this->Level.Things[i].z = 0;
		this->Level.Things[i].special = 0;
		this->Level.Things[i].args[0] = 0;
		this->Level.Things[i].args[1] = 0;
		this->Level.Things[i].args[2] = 0;
		this->Level.Things[i].args[3] = 0;
		this->Level.Things[i].args[4] = 0;
	}

	delete[] mt;
}

void FProcessor::load_things_ext(znbx_SliceU8 slice) {
	znbx_Thing2* things;
	int32_t thing_count;
	read_lump(slice, things, thing_count);

	this->Level.Things.Resize(thing_count);

	for (uint32_t i = 0; i < thing_count; ++i) {
		this->Level.Things[i].thingid = things[i].thing_id;
		this->Level.Things[i].x = little_short(things[i].x) << FRACBITS;
		this->Level.Things[i].y = little_short(things[i].y) << FRACBITS;
		this->Level.Things[i].z = little_short(things[i].z);
		this->Level.Things[i].angle = little_short(things[i].angle);
		this->Level.Things[i].type = little_short(things[i].type);
		this->Level.Things[i].flags = little_short(things[i].flags);
		this->Level.Things[i].special = things[i].special;
		this->Level.Things[i].args[0] = things[i].args[0];
		this->Level.Things[i].args[1] = things[i].args[1];
		this->Level.Things[i].args[2] = things[i].args[2];
		this->Level.Things[i].args[3] = things[i].args[3];
		this->Level.Things[i].args[4] = things[i].args[4];
	}

	delete[] things;
}

void FProcessor::finish_load() {
	if (this->Level.NumLines() == 0 || this->Level.NumVertices == 0 ||
		this->Level.NumSides() == 0 || this->Level.NumSectors() == 0) {
		return;
	}

	{
		// Removing extra vertices is done by the node builder.
		this->Level.remove_extra_lines();

		if (!this->no_prune) {
			this->Level.RemoveExtraSides();
			this->Level.remove_extra_sectors();
		}

		if (this->build_nodes) {
			get_poly_spots();
		}

		this->Level.find_map_bounds();
	}
}

void FProcessor::load_lines(znbx_SliceU8 slice) {
	int32_t line_count;
	MapLineDef* data;
	read_lump<MapLineDef>(slice, data, line_count);

	this->Level.Lines.Resize(line_count);

	for (uint32_t i = 0; i < line_count; ++i) {
		this->Level.Lines[i].v1 = little_short(data[i].v1);
		this->Level.Lines[i].v2 = little_short(data[i].v2);
		this->Level.Lines[i].flags = little_short(data[i].flags);
		this->Level.Lines[i].sidenum[0] = little_short(data[i].sidenum[0]);
		this->Level.Lines[i].sidenum[1] = little_short(data[i].sidenum[1]);

		if (this->Level.Lines[i].sidenum[0] == NO_MAP_INDEX)
			this->Level.Lines[i].sidenum[0] = NO_INDEX;
		if (this->Level.Lines[i].sidenum[1] == NO_MAP_INDEX)
			this->Level.Lines[i].sidenum[1] = NO_INDEX;

		// Store the special and tag in the args array so we don't lose them
		this->Level.Lines[i].special = 0;
		this->Level.Lines[i].args[0] = little_short(data[i].special);
		this->Level.Lines[i].args[1] = little_short(data[i].tag);
	}

	delete[] data;
}

void FProcessor::load_lines_ext(znbx_SliceU8 slice) {
	int32_t line_count;
	MapLineDef2* data;

	read_lump<MapLineDef2>(slice, data, line_count);

	this->Level.Lines.Resize(line_count);

	for (uint32_t i = 0; i < line_count; ++i) {
		this->Level.Lines[i].special = data[i].special;
		this->Level.Lines[i].args[0] = data[i].args[0];
		this->Level.Lines[i].args[1] = data[i].args[1];
		this->Level.Lines[i].args[2] = data[i].args[2];
		this->Level.Lines[i].args[3] = data[i].args[3];
		this->Level.Lines[i].args[4] = data[i].args[4];
		this->Level.Lines[i].v1 = little_short(data[i].v1);
		this->Level.Lines[i].v2 = little_short(data[i].v2);
		this->Level.Lines[i].flags = little_short(data[i].flags);
		this->Level.Lines[i].sidenum[0] = little_short(data[i].sidenum[0]);
		this->Level.Lines[i].sidenum[1] = little_short(data[i].sidenum[1]);

		if (this->Level.Lines[i].sidenum[0] == NO_MAP_INDEX)
			this->Level.Lines[i].sidenum[0] = NO_INDEX;
		if (this->Level.Lines[i].sidenum[1] == NO_MAP_INDEX)
			this->Level.Lines[i].sidenum[1] = NO_INDEX;
	}

	delete[] data;
}

void FProcessor::load_vertices(znbx_SliceU8 slice) {
	znbx_VertexRaw* data;
	read_lump(slice, data, this->Level.NumVertices);
	this->Level.Vertices = new znbx_VertexEx[this->Level.NumVertices];

	for (int i = 0; i < this->Level.NumVertices; ++i) {
		this->Level.Vertices[i].x = little_short(data[i].x) << FRACBITS;
		this->Level.Vertices[i].y = little_short(data[i].y) << FRACBITS;
		this->Level.Vertices[i].index = 0; // we don't need this value for non-UDMF maps
	}
}

void FProcessor::load_sides(znbx_SliceU8 slice) {
	MapSideDef* data;
	int32_t side_count;
	read_lump(slice, data, side_count);

	this->Level.Sides.Resize(side_count);

	for (uint32_t i = 0; i < side_count; ++i) {
		this->Level.Sides[i].textureoffset = data[i].textureoffset;
		this->Level.Sides[i].rowoffset = data[i].rowoffset;
		memcpy(this->Level.Sides[i].toptexture, data[i].toptexture, 8);
		memcpy(this->Level.Sides[i].bottomtexture, data[i].bottomtexture, 8);
		memcpy(this->Level.Sides[i].midtexture, data[i].midtexture, 8);

		this->Level.Sides[i].sector = little_short(data[i].sector);

		if (this->Level.Sides[i].sector == NO_MAP_INDEX)
			this->Level.Sides[i].sector = NO_INDEX;
	}

	delete[] data;
}

void FProcessor::load_sectors(znbx_SliceU8 slice) {
	MapSector* data;
	int32_t sector_count;
	read_lump(slice, data, sector_count);
	this->Level.Sectors.Resize(sector_count);

	for (int i = 0; i < sector_count; ++i) {
		this->Level.Sectors[i].data = data[i];
	}
}

void FLevel::find_map_bounds() {
	znbx_I16F16 minx, maxx, miny, maxy;

	minx = maxx = Vertices[0].x;
	miny = maxy = Vertices[0].y;

	for (int i = 1; i < NumVertices; ++i) {
		if (Vertices[i].x < minx)
			minx = Vertices[i].x;
		else if (Vertices[i].x > maxx)
			maxx = Vertices[i].x;
		if (Vertices[i].y < miny)
			miny = Vertices[i].y;
		else if (Vertices[i].y > maxy)
			maxy = Vertices[i].y;
	}

	MinX = minx;
	MinY = miny;
	MaxX = maxx;
	MaxY = maxy;
}

void FLevel::remove_extra_lines() {
	uint32_t i, newNumLines;

	// Extra lines are those with 0 length. Collision detection against
	// one of those could cause a divide by 0, so it's best to remove them.

	for (i = newNumLines = 0; i < NumLines(); ++i) {
		if (Vertices[Lines[i].v1].x != Vertices[Lines[i].v2].x ||
			Vertices[Lines[i].v1].y != Vertices[Lines[i].v2].y) {
			if (i != newNumLines) {
				Lines[newNumLines] = Lines[i];
			}
			++newNumLines;
		}
	}
	if (newNumLines < NumLines()) {
		uint32_t diff = NumLines() - newNumLines;

		printf("   Removed %d line%s with 0 length.\n", diff, diff > 1 ? "s" : "");
	}
	Lines.Resize(newNumLines);
}

void FLevel::RemoveExtraSides() {
	uint8_t* used;
	int* remap;
	uint32_t i, newNumSides;

	// Extra sides are those that aren't referenced by any lines.
	// They just waste space, so get rid of them.
	uint32_t NumSides = this->NumSides();

	used = new uint8_t[NumSides];
	memset(used, 0, NumSides * sizeof(*used));
	remap = new int[NumSides];

	// Mark all used sides
	for (i = 0; i < NumLines(); ++i) {
		if (Lines[i].sidenum[0] != NO_INDEX) {
			used[Lines[i].sidenum[0]] = 1;
		} else {
			printf("   Line %d needs a front sidedef before it will run with ZDoom.\n", i);
		}
		if (Lines[i].sidenum[1] != NO_INDEX) {
			used[Lines[i].sidenum[1]] = 1;
		}
	}

	// Shift out any unused sides
	for (i = newNumSides = 0; i < NumSides; ++i) {
		if (used[i]) {
			if (i != newNumSides) {
				Sides[newNumSides] = Sides[i];
			}
			remap[i] = newNumSides++;
		} else {
			remap[i] = NO_INDEX;
		}
	}

	if (newNumSides < NumSides) {
		int diff = NumSides - newNumSides;

		printf("   Removed %d unused sidedef%s.\n", diff, diff > 1 ? "s" : "");
		Sides.Resize(newNumSides);

		// Renumber side references in lines
		for (i = 0; i < NumLines(); ++i) {
			if (Lines[i].sidenum[0] != NO_INDEX) {
				Lines[i].sidenum[0] = remap[Lines[i].sidenum[0]];
			}
			if (Lines[i].sidenum[1] != NO_INDEX) {
				Lines[i].sidenum[1] = remap[Lines[i].sidenum[1]];
			}
		}
	}
	delete[] used;
	delete[] remap;
}

void FLevel::remove_extra_sectors() {
	uint8_t* used;
	uint32_t* remap;
	int i, newNumSectors;

	// Extra sectors are those that aren't referenced by any sides.
	// They just waste space, so get rid of them.

	NumOrgSectors = this->NumSectors();
	used = new uint8_t[this->NumSectors()];
	memset(used, 0, this->NumSectors() * sizeof(*used));
	remap = new uint32_t[this->NumSectors()];

	// Mark all used sectors
	for (i = 0; i < this->NumSides(); ++i) {
		if ((uint32_t)Sides[i].sector != NO_INDEX) {
			used[Sides[i].sector] = 1;
		} else {
			printf("   Sidedef %d needs a front sector before it will run with ZDoom.\n", i);
		}
	}

	// Shift out any unused sides
	for (i = newNumSectors = 0; i < NumSectors(); ++i) {
		if (used[i]) {
			if (i != newNumSectors) {
				Sectors[newNumSectors] = Sectors[i];
			}
			remap[i] = newNumSectors++;
		} else {
			remap[i] = NO_INDEX;
		}
	}

	if (newNumSectors < this->NumSectors()) {
		int diff = this->NumSectors() - newNumSectors;
		printf("   Removed %d unused sector%s.\n", diff, diff > 1 ? "s" : "");

		// Renumber sector references in sides
		for (i = 0; i < this->NumSides(); ++i) {
			if ((uint32_t)Sides[i].sector != NO_INDEX) {
				Sides[i].sector = remap[Sides[i].sector];
			}
		}
		// Make a reverse map for fixing reject lumps
		OrgSectorMap = new uint32_t[newNumSectors];
		for (i = 0; i < this->NumSectors(); ++i) {
			if (remap[i] != NO_INDEX) {
				OrgSectorMap[remap[i]] = i;
			}
		}

		Sectors.Resize(newNumSectors);
	}

	delete[] used;
	delete[] remap;
}

void FProcessor::get_poly_spots() {
	if (is_extended && this->check_poly_objs) {
		int spot1, spot2, anchor, i;

		// Determine if this is a Hexen map by looking for things of type 3000
		// Only Hexen maps use them, and they are the polyobject anchors
		for (i = 0; i < Level.NumThings(); ++i) {
			if (Level.Things[i].type == PO_HEX_ANCHOR_TYPE) {
				break;
			}
		}

		if (i < Level.NumThings()) {
			spot1 = PO_HEX_SPAWN_TYPE;
			spot2 = PO_HEX_SPAWNCRUSH_TYPE;
			anchor = PO_HEX_ANCHOR_TYPE;
		} else {
			spot1 = PO_SPAWN_TYPE;
			spot2 = PO_SPAWNCRUSH_TYPE;
			anchor = PO_ANCHOR_TYPE;
		}

		for (i = 0; i < Level.NumThings(); ++i) {
			if (Level.Things[i].type == spot1 || Level.Things[i].type == spot2 ||
				Level.Things[i].type == PO_SPAWNHURT_TYPE || Level.Things[i].type == anchor) {
				FNodeBuilder::FPolyStart newvert;
				newvert.x = Level.Things[i].x;
				newvert.y = Level.Things[i].y;
				newvert.polynum = Level.Things[i].angle;
				if (Level.Things[i].type == anchor) {
					poly_anchors.Push(newvert);
				} else {
					poly_starts.Push(newvert);
				}
			}
		}
	}
}

void FProcessor::Process(const znbx_NodeConfig* const config) {
	if (Level.NumLines() == 0 || Level.NumSides() == 0 || Level.NumSectors() == 0 ||
		Level.NumVertices == 0) {
		return;
	}

	if (this->build_nodes) {
		FNodeBuilder* builder = NULL;

		// ZDoom's UDMF spec requires compressed GL nodes.
		// No other UDMF spec has defined anything regarding nodes yet.
		if (is_udmf) {
			this->build_gl_nodes = true;
			this->conform_nodes = false;
			this->gl_only = true;
			this->compress_gl_nodes = true;
		}

		try {
			builder = new FNodeBuilder(
				this->Level, this->poly_starts, this->poly_anchors, this->level_name,
				this->build_gl_nodes
			);

			if (builder == NULL) {
				throw std::runtime_error("   Not enough memory to build nodes!");
			}

			if (config != nullptr) {
				builder->aa_pref = config->aa_preference;
				builder->max_segs = config->max_segs;
				builder->split_cost = config->split_cost;

				if (builder->aa_pref < 1) {
					builder->aa_pref = 1;
				}

				if (builder->max_segs < 3) {
					builder->max_segs = 3;
				}

				if (builder->split_cost < 1) {
					// 1 means to add no extra weight at all.
					builder->split_cost = 1;
				}
			}

			delete[] Level.Vertices;
			builder->GetVertices(Level.Vertices, Level.NumVertices);

			if (this->conform_nodes) {
				// When the nodes are "conformed", the normal and GL nodes use the same
				// basic information. This creates normal nodes that are less "good" than
				// possible, but it makes it easier to compare the two sets of nodes to
				// determine the correctness of the GL nodes.
				builder->GetNodes(
					Level.Nodes, Level.NumNodes, Level.Segs, Level.NumSegs, Level.Subsectors,
					Level.NumSubsectors
				);
				builder->GetVertices(Level.GLVertices, Level.NumGLVertices);
				builder->GetGLNodes(
					Level.GLNodes, Level.NumGLNodes, Level.GLSegs, Level.NumGLSegs,
					Level.GLSubsectors, Level.NumGLSubsectors
				);
			} else {
				if (this->build_gl_nodes) {
					builder->GetVertices(Level.GLVertices, Level.NumGLVertices);
					builder->GetGLNodes(
						Level.GLNodes, Level.NumGLNodes, Level.GLSegs, Level.NumGLSegs,
						Level.GLSubsectors, Level.NumGLSubsectors
					);

					if (!this->gl_only) {
						// Now repeat the process to obtain regular nodes.
						delete builder;

						builder = new FNodeBuilder(
							Level, poly_starts, poly_anchors, this->level_name, false
						);

						if (builder == NULL) {
							throw std::runtime_error("   Not enough memory to build regular nodes!"
							);
						}

						if (config != nullptr) {
							builder->aa_pref = config->aa_preference;
							builder->max_segs = config->max_segs;
							builder->split_cost = config->split_cost;
						}

						delete[] Level.Vertices;
						builder->GetVertices(Level.Vertices, Level.NumVertices);
					}
				}
				if (!this->gl_only) {
					builder->GetNodes(
						Level.Nodes, Level.NumNodes, Level.Segs, Level.NumSegs, Level.Subsectors,
						Level.NumSubsectors
					);
				}
			}
			delete builder;
			builder = NULL;
		} catch (...) {
			if (builder != NULL) {
				delete builder;
			}
			throw;
		}
	}

	if (!is_udmf) {
		FBlockmapBuilder bbuilder(Level);
		uint16_t* blocks = bbuilder.GetBlockmap(Level.BlockmapSize);
		Level.Blockmap = new uint16_t[Level.BlockmapSize];
		memcpy(Level.Blockmap, blocks, Level.BlockmapSize * sizeof(uint16_t));

		Level.RejectSize = (Level.NumSectors() * Level.NumSectors() + 7) / 8;
		Level.Reject = NULL;

		switch (this->reject_mode) {
		case ZNBX_ERM_REBUILD:
#if 0
			FRejectBuilder reject(Level);
			Level.Reject = reject.GetReject();
#endif
			printf("   Rebuilding the reject is unsupported.\n");
			// Intentional fall-through
		case ZNBX_ERM_DONTTOUCH: {
#warning "TODO: handle this case"
		} break;

		case ZNBX_ERM_CREATE0:
			break;

		case ZNBX_ERM_CREATEZEROES:
			Level.Reject = new uint8_t[Level.RejectSize];
			memset(Level.Reject, 0, Level.RejectSize);
			break;
		}
	}

	this->node_version = ZNBX_NODEVERS_UNKNOWN;

	if (this->Level.GLNodes != nullptr && this->Level.NumGLNodes > 0) {
		bool frac_splitters =
			this->CheckForFracSplitters(this->Level.GLNodes, this->Level.NumGLNodes);

		if (frac_splitters) {
			this->node_version = ZNBX_NODEVERS_3;
		} else if (this->Level.NumLines() < 65'535) {
			this->node_version = ZNBX_NODEVERS_1;
		} else {
			this->node_version = ZNBX_NODEVERS_2;
		}
	}
}

znbx_NodeVersion FProcessor::get_node_version() const {
	return this->node_version;
}

uint8_t* FProcessor::fix_reject(const uint8_t* oldreject) {
	int x, y, ox, oy, pnum, opnum;
	int rejectSize = (Level.NumSectors() * Level.NumSectors() + 7) / 8;
	uint8_t* newreject = new uint8_t[rejectSize];

	memset(newreject, 0, rejectSize);

	for (y = 0; y < Level.NumSectors(); ++y) {
		oy = Level.OrgSectorMap[y];
		for (x = 0; x < Level.NumSectors(); ++x) {
			ox = Level.OrgSectorMap[x];
			pnum = y * Level.NumSectors() + x;
			opnum = oy * Level.NumSectors() + ox;

			if (oldreject[opnum >> 3] & (1 << (opnum & 7))) {
				newreject[pnum >> 3] |= 1 << (pnum & 7);
			}
		}
	}
	return newreject;
}

znbx_NodeEx* FProcessor::NodesToEx(const znbx_NodeRaw* nodes, int count) {
	if (count == 0) {
		return NULL;
	}

	znbx_NodeEx* Nodes = new znbx_NodeEx[Level.NumNodes];
	int x;

	for (x = 0; x < count; ++x) {
		uint16_t child;
		int i;

		for (i = 0; i < 4 + 2 * 4; ++i) {
			*((uint16_t*)&Nodes[x] + i) = little_short(*((uint16_t*)&nodes[x] + i));
		}
		for (i = 0; i < 2; ++i) {
			child = little_short(nodes[x].children[i]);
			if (child & NF_SUBSECTOR) {
				Nodes[x].children[i] = child + (NFX_SUBSECTOR - NF_SUBSECTOR);
			} else {
				Nodes[x].children[i] = child;
			}
		}
	}
	return Nodes;
}

znbx_SubsectorEx* FProcessor::SubsectorsToEx(const znbx_SubsectorRaw* ssec, int count) {
	if (count == 0) {
		return NULL;
	}

	znbx_SubsectorEx* out = new znbx_SubsectorEx[Level.NumSubsectors];
	int x;

	for (x = 0; x < count; ++x) {
		out[x].num_lines = little_short(ssec[x].num_lines);
		out[x].first_line = little_short(ssec[x].first_line);
	}

	return out;
}

znbx_SegGlEx* FProcessor::SegGLsToEx(const znbx_SegGl* segs, int count) {
	if (count == 0) {
		return NULL;
	}

	znbx_SegGlEx* out = new znbx_SegGlEx[count];
	int x;

	for (x = 0; x < count; ++x) {
		out[x].v1 = little_short(segs[x].v1);
		out[x].v2 = little_short(segs[x].v2);
		out[x].linedef = little_short(segs[x].linedef);
		out[x].side = little_short(segs[x].side);
		out[x].partner = little_short(segs[x].partner);
	}

	return out;
}

void FProcessor::WriteVertices(FWadWriter& out, int count) {
	int i;
	znbx_VertexEx* vertdata = Level.Vertices;

	short* verts = new short[count * 2];

	for (i = 0; i < count; ++i) {
		verts[i * 2] = little_short(vertdata[i].x >> FRACBITS);
		verts[i * 2 + 1] = little_short(vertdata[i].y >> FRACBITS);
	}
	out.WriteLump("VERTEXES", verts, sizeof(*verts) * count * 2);
	delete[] verts;

	if (count >= 32768) {
		printf("   VERTEXES is past the normal limit. (%d vertices)\n", count);
	}
}

void FProcessor::WriteLines(FWadWriter& out) {
	int i;

	if (is_extended) {
		MapLineDef2* Lines = new MapLineDef2[Level.NumLines()];
		for (i = 0; i < Level.NumLines(); ++i) {
			Lines[i].special = Level.Lines[i].special;
			Lines[i].args[0] = Level.Lines[i].args[0];
			Lines[i].args[1] = Level.Lines[i].args[1];
			Lines[i].args[2] = Level.Lines[i].args[2];
			Lines[i].args[3] = Level.Lines[i].args[3];
			Lines[i].args[4] = Level.Lines[i].args[4];
			Lines[i].v1 = little_short(uint16_t(Level.Lines[i].v1));
			Lines[i].v2 = little_short(uint16_t(Level.Lines[i].v2));
			Lines[i].flags = little_short(uint16_t(Level.Lines[i].flags));
			Lines[i].sidenum[0] = little_short(uint16_t(Level.Lines[i].sidenum[0]));
			Lines[i].sidenum[1] = little_short(uint16_t(Level.Lines[i].sidenum[1]));
		}
		out.WriteLump("LINEDEFS", Lines, Level.NumLines() * sizeof(*Lines));
		delete[] Lines;
	} else {
		MapLineDef* ld = new MapLineDef[Level.NumLines()];

		for (i = 0; i < Level.NumLines(); ++i) {
			ld[i].v1 = little_short(uint16_t(Level.Lines[i].v1));
			ld[i].v2 = little_short(uint16_t(Level.Lines[i].v2));
			ld[i].flags = little_short(uint16_t(Level.Lines[i].flags));
			ld[i].sidenum[0] = little_short(uint16_t(Level.Lines[i].sidenum[0]));
			ld[i].sidenum[1] = little_short(uint16_t(Level.Lines[i].sidenum[1]));
			ld[i].special = little_short(uint16_t(Level.Lines[i].args[0]));
			ld[i].tag = little_short(uint16_t(Level.Lines[i].args[1]));
		}
		out.WriteLump("LINEDEFS", ld, Level.NumLines() * sizeof(*ld));
		delete[] ld;
	}
}

void FProcessor::WriteSides(FWadWriter& out) {
	int i;
	MapSideDef* Sides = new MapSideDef[Level.NumSides()];

	for (i = 0; i < Level.NumSides(); ++i) {
		Sides[i].textureoffset = Level.Sides[i].textureoffset;
		Sides[i].rowoffset = Level.Sides[i].rowoffset;
		memcpy(Sides[i].toptexture, Level.Sides[i].toptexture, 8);
		memcpy(Sides[i].bottomtexture, Level.Sides[i].bottomtexture, 8);
		memcpy(Sides[i].midtexture, Level.Sides[i].midtexture, 8);
		Sides[i].sector = little_short(Level.Sides[i].sector);
	}
	out.WriteLump("SIDEDEFS", Sides, Level.NumSides() * sizeof(*Sides));
	delete[] Sides;
}

void FProcessor::WriteSectors(FWadWriter& out) {
	int i;
	MapSector* Sectors = new MapSector[Level.NumSectors()];

	for (i = 0; i < Level.NumSectors(); ++i) {
		Sectors[i] = Level.Sectors[i].data;
	}

	out.WriteLump("SECTORS", Sectors, Level.NumSectors() * sizeof(*Sectors));
}

void FProcessor::WriteSegs(FWadWriter& out) {
	int i;
	znbx_SegRaw* segdata;

	assert(Level.NumVertices < 65536);

	segdata = new znbx_SegRaw[Level.NumSegs];

	for (i = 0; i < Level.NumSegs; ++i) {
		segdata[i].v1 = little_short(uint16_t(Level.Segs[i].v1));
		segdata[i].v2 = little_short(uint16_t(Level.Segs[i].v2));
		segdata[i].angle = little_short(Level.Segs[i].angle);
		segdata[i].linedef = little_short(Level.Segs[i].linedef);
		segdata[i].side = little_short(Level.Segs[i].side);
		segdata[i].offset = little_short(Level.Segs[i].offset);
	}
	out.WriteLump("SEGS", segdata, sizeof(*segdata) * Level.NumSegs);

	if (Level.NumSegs >= 65536) {
		printf("   SEGS is too big for any port. (%d segs)\n", Level.NumSegs);
	} else if (Level.NumSegs >= 32768) {
		printf("   SEGS is too big for vanilla Doom and some ports. (%d segs)\n", Level.NumSegs);
	}
}

void FProcessor::WriteSSectors(FWadWriter& out) const {
	WriteSSectors2(out, "SSECTORS", Level.Subsectors, Level.NumSubsectors);
}

void FProcessor::WriteSSectors2(
	FWadWriter& out, const char* name, const znbx_SubsectorEx* subs, int count
) const {
	int i;
	znbx_SubsectorRaw* ssec;

	ssec = new znbx_SubsectorRaw[count];

	for (i = 0; i < count; ++i) {
		ssec[i].first_line = little_short((uint16_t)subs[i].first_line);
		ssec[i].num_lines = little_short((uint16_t)subs[i].num_lines);
	}
	out.WriteLump(name, ssec, sizeof(*ssec) * count);
	delete[] ssec;

	if (count >= 65536) {
		printf("   %s is too big. (%d subsectors)\n", name, count);
	}
}

void FProcessor::WriteSSectors5(
	FWadWriter& out, const char* name, const znbx_SubsectorEx* subs, int count
) const {
	int i;
	znbx_SubsectorEx* ssec;

	ssec = new znbx_SubsectorEx[count];

	for (i = 0; i < count; ++i) {
		ssec[i].first_line = little_long(subs[i].first_line);
		ssec[i].num_lines = little_long(subs[i].num_lines);
	}
	out.WriteLump(name, ssec, sizeof(*ssec) * count);
	delete[] ssec;
}

void FProcessor::WriteNodes(FWadWriter& out) const {
	WriteNodes2(out, "NODES", Level.Nodes, Level.NumNodes);
}

void FProcessor::WriteNodes2(
	FWadWriter& out, const char* name, const znbx_NodeEx* zaNodes, int count
) const {
	int i, j;
	short *onodes, *nodes;

	nodes = onodes = new short[count * sizeof(znbx_NodeRaw) / 2];

	for (i = 0; i < count; ++i) {
		nodes[0] = little_short(zaNodes[i].x >> 16);
		nodes[1] = little_short(zaNodes[i].y >> 16);
		nodes[2] = little_short(zaNodes[i].dx >> 16);
		nodes[3] = little_short(zaNodes[i].dy >> 16);
		nodes += 4;
		const short* inodes = (short*)&zaNodes[i].bbox[0][0];
		for (j = 0; j < 2 * 4; ++j) {
			nodes[j] = little_short(inodes[j]);
		}
		nodes += j;
		for (j = 0; j < 2; ++j) {
			uint32_t child = zaNodes[i].children[j];
			if (child & NFX_SUBSECTOR) {
				*nodes++ = little_short(uint16_t(child - (NFX_SUBSECTOR + NF_SUBSECTOR)));
			} else {
				*nodes++ = little_short((uint16_t)child);
			}
		}
	}
	out.WriteLump(name, onodes, count * sizeof(znbx_NodeRaw));
	delete[] onodes;

	if (count >= 32768) {
		printf("   %s is too big. (%d nodes)\n", name, count);
	}
}

void FProcessor::WriteNodes5(
	FWadWriter& out, const char* name, const znbx_NodeEx* zaNodes, int count
) const {
	int i, j;
	znbx_NodeExO* const nodes = new znbx_NodeExO[count * sizeof(znbx_NodeEx)];

	for (i = 0; i < count; ++i) {
		const short* inodes = &zaNodes[i].bbox[0][0];
		short* coord = &nodes[i].bbox[0][0];
		for (j = 0; j < 2 * 4; ++j) {
			coord[j] = little_short(inodes[j]);
		}
		nodes[i].x = little_short(zaNodes[i].x >> 16);
		nodes[i].y = little_short(zaNodes[i].y >> 16);
		nodes[i].dx = little_short(zaNodes[i].dx >> 16);
		nodes[i].dy = little_short(zaNodes[i].dy >> 16);
		for (j = 0; j < 2; ++j) {
			nodes[i].children[j] = little_long(zaNodes[i].children[j]);
		}
	}
	out.WriteLump(name, nodes, count * sizeof(znbx_NodeEx));
	delete[] nodes;
}

void FProcessor::WriteBlockmap(FWadWriter& out) {
	if (this->blockmap_mode == ZNBX_EBM_CREATE0) {
		out.CreateLabel("BLOCKMAP");
		return;
	}

	size_t i, count;
	uint16_t* blocks;

	count = Level.BlockmapSize;
	blocks = Level.Blockmap;

	for (i = 0; i < count; ++i) {
		blocks[i] = little_short(blocks[i]);
	}
	out.WriteLump("BLOCKMAP", blocks, int(sizeof(*blocks) * count));

#ifdef BLOCK_TEST
	FILE* f = fopen("blockmap.lm2", "wb");
	if (f) {
		fwrite(blocks, count, sizeof(*blocks), f);
		fclose(f);
	}
#endif

	for (i = 0; i < count; ++i) {
		blocks[i] = little_short(blocks[i]);
	}

	if (count >= 65536) {
		printf("   BLOCKMAP is so big that ports will have to recreate it.\n"
			   "   Vanilla Doom cannot handle it at all. If this map is for ZDoom 2+,\n"
			   "   you should use the -b switch to save space in the wad.\n");
	} else if (count >= 32768) {
		printf("   BLOCKMAP is too big for vanilla Doom.\n");
	}
}

void FProcessor::WriteReject(FWadWriter& out) {
	if (this->reject_mode == ZNBX_ERM_CREATE0 || Level.Reject == NULL) {
		out.CreateLabel("REJECT");
	} else {
		out.WriteLump("REJECT", Level.Reject, Level.RejectSize);
	}
}

void FProcessor::WriteGLVertices(FWadWriter& out, bool v5) {
	int i, count = (Level.NumGLVertices - Level.NumOrgVerts);
	znbx_VertexEx* vertdata = Level.GLVertices + Level.NumOrgVerts;

	znbx_I16F16* verts = new znbx_I16F16[count * 2 + 1];
	char* magic = (char*)verts;
	magic[0] = 'g';
	magic[1] = 'N';
	magic[2] = 'd';
	magic[3] = v5 ? '5' : '2';

	for (i = 0; i < count; ++i) {
		verts[i * 2 + 1] = little_short(vertdata[i].x);
		verts[i * 2 + 2] = little_short(vertdata[i].y);
	}
	out.WriteLump("GL_VERT", verts, sizeof(*verts) * (count * 2 + 1));
	delete[] verts;

	if (count > 65536) {
		printf("   GL_VERT is too big. (%d GL vertices)\n", count / 2);
	}
}

void FProcessor::WriteGLSegs(FWadWriter& out, bool v5) {
	if (v5) {
		WriteGLSegs5(out);
		return;
	}
	int i, count;
	znbx_SegGl* segdata;

	count = Level.NumGLSegs;
	segdata = new znbx_SegGl[count];

	for (i = 0; i < count; ++i) {
		if (Level.GLSegs[i].v1 < (uint32_t)Level.NumOrgVerts) {
			segdata[i].v1 = little_short((uint16_t)Level.GLSegs[i].v1);
		} else {
			segdata[i].v1 =
				little_short(0x8000 | (uint16_t)(Level.GLSegs[i].v1 - Level.NumOrgVerts));
		}
		if (Level.GLSegs[i].v2 < (uint32_t)Level.NumOrgVerts) {
			segdata[i].v2 = (uint16_t)little_short(Level.GLSegs[i].v2);
		} else {
			segdata[i].v2 =
				little_short(0x8000 | (uint16_t)(Level.GLSegs[i].v2 - Level.NumOrgVerts));
		}
		segdata[i].linedef = little_short((uint16_t)Level.GLSegs[i].linedef);
		segdata[i].side = little_short(Level.GLSegs[i].side);
		segdata[i].partner = little_short((uint16_t)Level.GLSegs[i].partner);
	}
	out.WriteLump("GL_SEGS", segdata, sizeof(znbx_SegGl) * count);
	delete[] segdata;

	if (count >= 65536) {
		printf("   GL_SEGS is too big for any port. (%d GL segs)\n", count);
	} else if (count >= 32768) {
		printf("   GL_SEGS is too big for some ports. (%d GL segs)\n", count);
	}
}

void FProcessor::WriteGLSegs5(FWadWriter& out) {
	int i, count;
	znbx_SegGlEx* segdata;

	count = Level.NumGLSegs;
	segdata = new znbx_SegGlEx[count];

	for (i = 0; i < count; ++i) {
		if (Level.GLSegs[i].v1 < (uint32_t)Level.NumOrgVerts) {
			segdata[i].v1 = little_long(Level.GLSegs[i].v1);
		} else {
			segdata[i].v1 = little_long(0x80000000u | ((int)Level.GLSegs[i].v1 - Level.NumOrgVerts));
		}
		if (Level.GLSegs[i].v2 < (uint32_t)Level.NumOrgVerts) {
			segdata[i].v2 = little_long(Level.GLSegs[i].v2);
		} else {
			segdata[i].v2 = little_long(0x80000000u | ((int)Level.GLSegs[i].v2 - Level.NumOrgVerts));
		}
		segdata[i].linedef = little_short(Level.GLSegs[i].linedef);
		segdata[i].side = little_short(Level.GLSegs[i].side);
		segdata[i].partner = little_long(Level.GLSegs[i].partner);
	}
	out.WriteLump("GL_SEGS", segdata, sizeof(znbx_SegGlEx) * count);
	delete[] segdata;
}

void FProcessor::WriteGLSSect(FWadWriter& out, bool v5) {
	if (!v5) {
		WriteSSectors2(out, "GL_SSECT", Level.GLSubsectors, Level.NumGLSubsectors);
	} else {
		WriteSSectors5(out, "GL_SSECT", Level.GLSubsectors, Level.NumGLSubsectors);
	}
}

void FProcessor::WriteGLNodes(FWadWriter& out, bool v5) {
	if (!v5) {
		WriteNodes2(out, "GL_NODES", Level.GLNodes, Level.NumGLNodes);
	} else {
		WriteNodes5(out, "GL_NODES", Level.GLNodes, Level.NumGLNodes);
	}
}

void FProcessor::WriteBSPZ(FWadWriter& out, const char* label) {
	ZLibOut zout(out);

	if (!this->compress_nodes) {
		printf("   Nodes are so big that compression has been forced.\n");
	}

	out.StartWritingLump(label);
	out.AddToLump("ZNOD", 4);
	WriteVerticesZ(
		zout, &Level.Vertices[Level.NumOrgVerts], Level.NumOrgVerts,
		Level.NumVertices - Level.NumOrgVerts
	);
	WriteSubsectorsZ(zout, Level.Subsectors, Level.NumSubsectors);
	WriteSegsZ(zout, Level.Segs, Level.NumSegs);
	WriteNodesZ(zout, Level.Nodes, Level.NumNodes, 1);
}

void FProcessor::WriteGLBSPZ(FWadWriter& out, const char* label) {
	ZLibOut zout(out);
	bool fracsplitters = CheckForFracSplitters(Level.GLNodes, Level.NumGLNodes);
	int nodever;

	if (!this->compress_gl_nodes) {
		printf("   GL Nodes are so big that compression has been forced.\n");
	}

	out.StartWritingLump(label);
	if (fracsplitters) {
		out.AddToLump("ZGL3", 4);
		nodever = 3;
	} else if (Level.NumLines() < 65535) {
		out.AddToLump("ZGLN", 4);
		nodever = 1;
	} else {
		out.AddToLump("ZGL2", 4);
		nodever = 2;
	}
	WriteVerticesZ(
		zout, &Level.GLVertices[Level.NumOrgVerts], Level.NumOrgVerts,
		Level.NumGLVertices - Level.NumOrgVerts
	);
	WriteSubsectorsZ(zout, Level.GLSubsectors, Level.NumGLSubsectors);
	WriteGLSegsZ(zout, Level.GLSegs, Level.NumGLSegs, nodever);
	WriteNodesZ(zout, Level.GLNodes, Level.NumGLNodes, nodever);
}

void FProcessor::WriteVerticesZ(
	ZLibOut& out, const znbx_VertexEx* verts, int orgverts, int newverts
) {
	out << (uint32_t)orgverts << (uint32_t)newverts;

	for (int i = 0; i < newverts; ++i) {
		out << verts[i].x << verts[i].y;
	}
}

void FProcessor::WriteSubsectorsZ(ZLibOut& out, const znbx_SubsectorEx* subs, int numsubs) {
	out << (uint32_t)numsubs;

	for (int i = 0; i < numsubs; ++i) {
		out << (uint32_t)subs[i].num_lines;
	}
}

void FProcessor::WriteSegsZ(ZLibOut& out, const znbx_SegEx* segs, int numsegs) {
	out << (uint32_t)numsegs;

	for (int i = 0; i < numsegs; ++i) {
		out << (uint32_t)segs[i].v1 << (uint32_t)segs[i].v2 << (uint16_t)segs[i].linedef
			<< (uint8_t)segs[i].side;
	}
}

void FProcessor::WriteGLSegsZ(ZLibOut& out, const znbx_SegGlEx* segs, int numsegs, int nodever) {
	out << (uint32_t)numsegs;

	if (nodever < 2) {
		for (int i = 0; i < numsegs; ++i) {
			out << (uint32_t)segs[i].v1 << (uint32_t)segs[i].partner << (uint16_t)segs[i].linedef
				<< (uint8_t)segs[i].side;
		}
	} else {
		for (int i = 0; i < numsegs; ++i) {
			out << (uint32_t)segs[i].v1 << (uint32_t)segs[i].partner << (uint32_t)segs[i].linedef
				<< (uint8_t)segs[i].side;
		}
	}
}

void FProcessor::WriteNodesZ(ZLibOut& out, const znbx_NodeEx* nodes, int numnodes, int nodever) {
	out << (uint32_t)numnodes;

	for (int i = 0; i < numnodes; ++i) {
		if (nodever < 3) {
			out << (int16_t)(nodes[i].x >> 16) << (int16_t)(nodes[i].y >> 16)
				<< (int16_t)(nodes[i].dx >> 16) << (int16_t)(nodes[i].dy >> 16);
		} else {
			out << (uint32_t)nodes[i].x << (uint32_t)nodes[i].y << (uint32_t)nodes[i].dx
				<< (uint32_t)nodes[i].dy;
		}
		for (int j = 0; j < 2; ++j) {
			for (int k = 0; k < 4; ++k) {
				out << (int16_t)nodes[i].bbox[j][k];
			}
		}
		out << (uint32_t)nodes[i].children[0] << (uint32_t)nodes[i].children[1];
	}
}

void FProcessor::WriteBSPX(FWadWriter& out, const char* label) {
	if (!this->compress_nodes) {
		printf("   Nodes are so big that extended format has been forced.\n");
	}

	out.StartWritingLump(label);
	out.AddToLump("XNOD", 4);
	WriteVerticesX(
		out, &Level.Vertices[Level.NumOrgVerts], Level.NumOrgVerts,
		Level.NumVertices - Level.NumOrgVerts
	);
	WriteSubsectorsX(out, Level.Subsectors, Level.NumSubsectors);
	WriteSegsX(out, Level.Segs, Level.NumSegs);
	WriteNodesX(out, Level.Nodes, Level.NumNodes, 1);
}

void FProcessor::WriteGLBSPX(FWadWriter& out, const char* label) {
	bool fracsplitters = CheckForFracSplitters(Level.GLNodes, Level.NumGLNodes);
	int nodever;

	if (!this->compress_gl_nodes) {
		printf("   GL Nodes are so big that extended format has been forced.\n");
	}

	out.StartWritingLump(label);
	if (fracsplitters) {
		out.AddToLump("XGL3", 4);
		nodever = 3;
	} else if (Level.NumLines() < 65535) {
		out.AddToLump("XGLN", 4);
		nodever = 1;
	} else {
		out.AddToLump("XGL2", 4);
		nodever = 2;
	}
	WriteVerticesX(
		out, &Level.GLVertices[Level.NumOrgVerts], Level.NumOrgVerts,
		Level.NumGLVertices - Level.NumOrgVerts
	);
	WriteSubsectorsX(out, Level.GLSubsectors, Level.NumGLSubsectors);
	WriteGLSegsX(out, Level.GLSegs, Level.NumGLSegs, nodever);
	WriteNodesX(out, Level.GLNodes, Level.NumGLNodes, nodever);
}

void FProcessor::WriteVerticesX(
	FWadWriter& out, const znbx_VertexEx* verts, int orgverts, int newverts
) {
	out << (uint32_t)orgverts << (uint32_t)newverts;

	for (int i = 0; i < newverts; ++i) {
		out << verts[i].x << verts[i].y;
	}
}

void FProcessor::WriteSubsectorsX(FWadWriter& out, const znbx_SubsectorEx* subs, int numsubs) {
	out << (uint32_t)numsubs;

	for (int i = 0; i < numsubs; ++i) {
		out << (uint32_t)subs[i].num_lines;
	}
}

void FProcessor::WriteSegsX(FWadWriter& out, const znbx_SegEx* segs, int numsegs) {
	out << (uint32_t)numsegs;

	for (int i = 0; i < numsegs; ++i) {
		out << (uint32_t)segs[i].v1 << (uint32_t)segs[i].v2 << (uint16_t)segs[i].linedef
			<< (uint8_t)segs[i].side;
	}
}

void FProcessor::WriteGLSegsX(
	FWadWriter& out, const znbx_SegGlEx* segs, int numsegs, int nodever
) {
	out << (uint32_t)numsegs;

	if (nodever < 2) {
		for (int i = 0; i < numsegs; ++i) {
			out << (uint32_t)segs[i].v1 << (uint32_t)segs[i].partner << (uint16_t)segs[i].linedef
				<< (uint8_t)segs[i].side;
		}
	} else {
		for (int i = 0; i < numsegs; ++i) {
			out << (uint32_t)segs[i].v1 << (uint32_t)segs[i].partner << (uint32_t)segs[i].linedef
				<< (uint8_t)segs[i].side;
		}
	}
}

void FProcessor::WriteNodesX(
	FWadWriter& out, const znbx_NodeEx* nodes, int numnodes, int nodever
) {
	out << (uint32_t)numnodes;

	for (int i = 0; i < numnodes; ++i) {
		if (nodever < 3) {
			out << (int16_t)(nodes[i].x >> 16) << (int16_t)(nodes[i].y >> 16)
				<< (int16_t)(nodes[i].dx >> 16) << (int16_t)(nodes[i].dy >> 16);
		} else {
			out << (uint32_t)nodes[i].x << (uint32_t)nodes[i].y << (uint32_t)nodes[i].dx
				<< (uint32_t)nodes[i].dy;
		}
		for (int j = 0; j < 2; ++j) {
			for (int k = 0; k < 4; ++k) {
				out << (int16_t)nodes[i].bbox[j][k];
			}
		}
		out << (uint32_t)nodes[i].children[0] << (uint32_t)nodes[i].children[1];
	}
}

bool FProcessor::CheckForFracSplitters(const znbx_NodeEx* nodes, int numnodes) const {
	for (int i = 0; i < numnodes; ++i) {
		if (0 != ((nodes[i].x | nodes[i].y | nodes[i].dx | nodes[i].dy) & 0x0000FFFF)) {
			return true;
		}
	}
	return false;
}

// zlib lump writer ---------------------------------------------------------

ZLibOut::ZLibOut(FWadWriter& out) : Out(out) {
	int err;

	Stream.next_in = Z_NULL;
	Stream.avail_in = 0;
	Stream.zalloc = Z_NULL;
	Stream.zfree = Z_NULL;
	err = deflateInit(&Stream, 9);

	if (err != Z_OK) {
		throw std::runtime_error("Could not initialize deflate buffer.");
	}

	Stream.next_out = Buffer;
	Stream.avail_out = BUFFER_SIZE;
}

ZLibOut::~ZLibOut() {
	int err;

	for (;;) {
		err = deflate(&Stream, Z_FINISH);
		if (err != Z_OK) {
			break;
		}
		if (Stream.avail_out == 0) {
			Out.AddToLump(Buffer, BUFFER_SIZE);
			Stream.next_out = Buffer;
			Stream.avail_out = BUFFER_SIZE;
		}
	}
	deflateEnd(&Stream);
	if (err != Z_STREAM_END) {
		throw std::runtime_error("Error deflating data.");
	}
	Out.AddToLump(Buffer, BUFFER_SIZE - Stream.avail_out);
}

void ZLibOut::Write(uint8_t* data, int len) {
	int err;

	Stream.next_in = data;
	Stream.avail_in = len;
	err = deflate(&Stream, 0);
	while (Stream.avail_out == 0 && err == Z_OK) {
		Out.AddToLump(Buffer, BUFFER_SIZE);
		Stream.next_out = Buffer;
		Stream.avail_out = BUFFER_SIZE;
		if (Stream.avail_in != 0) {
			err = deflate(&Stream, 0);
		}
	}
	if (err != Z_OK) {
		throw std::runtime_error("Error deflating data.");
	}
}

ZLibOut& ZLibOut::operator<<(uint8_t val) {
	Write(&val, 1);
	return *this;
}

ZLibOut& ZLibOut::operator<<(uint16_t val) {
	val = little_short(val);
	Write((uint8_t*)&val, 2);
	return *this;
}

ZLibOut& ZLibOut::operator<<(int16_t val) {
	val = little_short(val);
	Write((uint8_t*)&val, 2);
	return *this;
}

ZLibOut& ZLibOut::operator<<(uint32_t val) {
	val = little_long(val);
	Write((uint8_t*)&val, 4);
	return *this;
}

ZLibOut& ZLibOut::operator<<(znbx_I16F16 val) {
	val = little_long(val);
	Write((uint8_t*)&val, 4);
	return *this;
}
