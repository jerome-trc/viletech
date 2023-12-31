/// @file
/// @brief UDMF map reading and writing.

/*

Copyright (C) 2009 Christoph Oelckers

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

#include <float.h>

#include "processor.hpp"
#include "sc_man.hpp"
#include "xs_Float.hpp"

/// Parses a 'key = value;' line of the TEXTMAP lump.
const char* FProcessor::ParseKey(const char*& value) {
	this->scanner.must_get_string();
	const char* key = stbuf.Copy(this->scanner.string);
	this->scanner.must_get_string_name("=");

	this->scanner.number = INT_MIN;
	this->scanner.flnum = DBL_MIN;

	if (!this->scanner.check_float()) {
		this->scanner.must_get_string();
	}

	value = stbuf.Copy(this->scanner.string);
	this->scanner.must_get_string_name(";");
	return key;
}

bool FProcessor::CheckKey(const char*& key, const char*& value) {
	this->scanner.save_pos();
	this->scanner.must_get_string();
	if (this->scanner.check_string("=")) {
		this->scanner.restore_pos();
		key = ParseKey(value);
		return true;
	}
	this->scanner.restore_pos();
	return false;
}

int FProcessor::CheckInt(const char* key) {
	if (this->scanner.number == INT_MIN) {
		this->scanner.script_err("Integer value expected for key '%s'", key);
	}

	return this->scanner.number;
}

double FProcessor::CheckFloat(const char* key) {
	if (this->scanner.flnum == DBL_MIN) {
		this->scanner.script_err("Floating point value expected for key '%s'", key);
	}

	return this->scanner.flnum;
}

znbx_I16F16 FProcessor::CheckFixed(const char* key) {
	double val = CheckFloat(key);
	if (val < -32768 || val > 32767) {
		this->scanner.script_err(
			"Fixed point value is out of range for key '%s'\n\t%.2f should be within "
			"[-32768,32767]",
			key, val / 65536
		);
	}
	return xs_Fix<16>::ToFix(val);
}

void FProcessor::ParseThing(IntThing* th) {
	this->scanner.must_get_string_name("{");
	while (!this->scanner.check_string("}")) {
		const char* value;
		const char* key = ParseKey(value);

		// The only properties we need from a thing are
		// x, y, angle and type.

		if (!stricmp(key, "x")) {
			th->x = CheckFixed(key);
		} else if (!stricmp(key, "y")) {
			th->y = CheckFixed(key);
		}
		if (!stricmp(key, "angle")) {
			th->angle = (short)CheckInt(key);
		}
		if (!stricmp(key, "type")) {
			th->type = (short)CheckInt(key);
		}

		// now store the key in its unprocessed form
		znbx_UdmfKey k = { key, value };
		th->props.Push(k);
	}
}

void FProcessor::ParseLinedef(IntLineDef* ld) {
	this->scanner.must_get_string_name("{");
	ld->v1 = ld->v2 = ld->sidenum[0] = ld->sidenum[1] = NO_INDEX;
	ld->special = 0;
	while (!this->scanner.check_string("}")) {
		const char* value;
		const char* key = ParseKey(value);

		if (!stricmp(key, "v1")) {
			ld->v1 = CheckInt(key);
			continue; // do not store in props
		} else if (!stricmp(key, "v2")) {
			ld->v2 = CheckInt(key);
			continue; // do not store in props
		} else if (is_extended && !stricmp(key, "special")) {
			ld->special = CheckInt(key);
		} else if (is_extended && !stricmp(key, "arg0")) {
			ld->args[0] = CheckInt(key);
		}
		if (!stricmp(key, "sidefront")) {
			ld->sidenum[0] = CheckInt(key);
			continue; // do not store in props
		} else if (!stricmp(key, "sideback")) {
			ld->sidenum[1] = CheckInt(key);
			continue; // do not store in props
		}

		// now store the key in its unprocessed form
		znbx_UdmfKey k = { key, value };
		ld->props.Push(k);
	}
}

void FProcessor::ParseSidedef(IntSideDef* sd) {
	this->scanner.must_get_string_name("{");
	sd->sector = NO_INDEX;
	while (!this->scanner.check_string("}")) {
		const char* value;
		const char* key = ParseKey(value);

		if (!stricmp(key, "sector")) {
			sd->sector = CheckInt(key);
			continue; // do not store in props
		}

		// now store the key in its unprocessed form
		znbx_UdmfKey k = { key, value };
		sd->props.Push(k);
	}
}

void FProcessor::ParseSector(IntSector* sec) {
	this->scanner.must_get_string_name("{");
	while (!this->scanner.check_string("}")) {
		const char* value;
		const char* key = ParseKey(value);

		// No specific sector properties are ever used by the node builder
		// so everything can go directly to the props array.

		// now store the key in its unprocessed form
		znbx_UdmfKey k = { key, value };
		sec->props.Push(k);
	}
}

void FProcessor::ParseVertex(znbx_VertexEx* vt, IntVertex* vtp) {
	vt->x = vt->y = 0;
	this->scanner.must_get_string_name("{");
	while (!this->scanner.check_string("}")) {
		const char* value;
		const char* key = ParseKey(value);

		if (!stricmp(key, "x")) {
			vt->x = CheckFixed(key);
		} else if (!stricmp(key, "y")) {
			vt->y = CheckFixed(key);
		}

		// now store the key in its unprocessed form
		znbx_UdmfKey k = { key, value };
		vtp->props.Push(k);
	}
}

/// Parses global map properties.
void FProcessor::ParseMapProperties() {
	const char *key, *value;

	// all global keys must come before the first map element.

	while (CheckKey(key, value)) {
		if (!stricmp(key, "namespace")) {
			// all unknown namespaces are assumed to be standard.
			is_extended = !stricmp(value, "\"ZDoom\"") || !stricmp(value, "\"Hexen\"") ||
					   !stricmp(value, "\"Vavoom\"");
		}

		// now store the key in its unprocessed form
		znbx_UdmfKey k = { key, value };
		Level.props.Push(k);
	}
}

void FProcessor::ParseTextMap(znbx_SliceU8 slice) {
	char* buffer;
	int32_t bufsz;
	TArray<znbx_VertexEx> Vertices;
	read_lump(slice, buffer, bufsz);
	this->scanner.open_mem("TEXTMAP", buffer, bufsz);

	this->scanner.set_c_mode(true);
	ParseMapProperties();

	while (this->scanner.get_string()) {
		if (this->scanner.compare("thing")) {
			IntThing* th = &Level.Things[Level.Things.Reserve(1)];
			ParseThing(th);
		} else if (this->scanner.compare("linedef")) {
			IntLineDef* ld = &Level.Lines[Level.Lines.Reserve(1)];
			ParseLinedef(ld);
		} else if (this->scanner.compare("sidedef")) {
			IntSideDef* sd = &Level.Sides[Level.Sides.Reserve(1)];
			ParseSidedef(sd);
		} else if (this->scanner.compare("sector")) {
			IntSector* sec = &Level.Sectors[Level.Sectors.Reserve(1)];
			ParseSector(sec);
		} else if (this->scanner.compare("vertex")) {
			znbx_VertexEx* vt = &Vertices[Vertices.Reserve(1)];
			IntVertex* vtp = &Level.VertexProps[Level.VertexProps.Reserve(1)];
			vt->index = Vertices.Size();
			ParseVertex(vt, vtp);
		}
	}

	Level.Vertices = new znbx_VertexEx[Vertices.Size()];
	Level.NumVertices = Vertices.Size();
	memcpy(Level.Vertices, &Vertices[0], Vertices.Size() * sizeof(znbx_VertexEx));
	this->scanner.close();
	delete[] buffer;
}

/// Write a property list.
void FProcessor::WriteProps(FWadWriter& out, TArray<znbx_UdmfKey>& props) {
	for (unsigned i = 0; i < props.Size(); i++) {
		out.AddToLump(props[i].key, (int)strlen(props[i].key));
		out.AddToLump(" = ", 3);
		out.AddToLump(props[i].value, (int)strlen(props[i].value));
		out.AddToLump(";\n", 2);
	}
}

void FProcessor::WriteIntProp(FWadWriter& out, const char* key, int value) {
	char buffer[20];

	out.AddToLump(key, (int)strlen(key));
	out.AddToLump(" = ", 3);
	sprintf(buffer, "%d;\n", value);
	out.AddToLump(buffer, (int)strlen(buffer));
}

void FProcessor::WriteThingUDMF(FWadWriter& out, IntThing* th, int num) {
	out.AddToLump("thing", 5);
	if (this->write_comments) {
		char buffer[32];
		int len = sprintf(buffer, " // %d", num);
		out.AddToLump(buffer, len);
	}
	out.AddToLump("\n{\n", 3);
	WriteProps(out, th->props);
	out.AddToLump("}\n\n", 3);
}

void FProcessor::WriteLinedefUDMF(FWadWriter& out, IntLineDef* ld, int num) {
	out.AddToLump("linedef", 7);
	if (this->write_comments) {
		char buffer[32];
		int len = sprintf(buffer, " // %d", num);
		out.AddToLump(buffer, len);
	}
	out.AddToLump("\n{\n", 3);
	WriteIntProp(out, "v1", ld->v1);
	WriteIntProp(out, "v2", ld->v2);
	if (ld->sidenum[0] != NO_INDEX)
		WriteIntProp(out, "sidefront", ld->sidenum[0]);
	if (ld->sidenum[1] != NO_INDEX)
		WriteIntProp(out, "sideback", ld->sidenum[1]);
	WriteProps(out, ld->props);
	out.AddToLump("}\n\n", 3);
}

void FProcessor::WriteSidedefUDMF(FWadWriter& out, IntSideDef* sd, int num) {
	out.AddToLump("sidedef", 7);
	if (this->write_comments) {
		char buffer[32];
		int len = sprintf(buffer, " // %d", num);
		out.AddToLump(buffer, len);
	}
	out.AddToLump("\n{\n", 3);
	WriteIntProp(out, "sector", sd->sector);
	WriteProps(out, sd->props);
	out.AddToLump("}\n\n", 3);
}

void FProcessor::WriteSectorUDMF(FWadWriter& out, IntSector* sec, int num) {
	out.AddToLump("sector", 6);
	if (this->write_comments) {
		char buffer[32];
		int len = sprintf(buffer, " // %d", num);
		out.AddToLump(buffer, len);
	}
	out.AddToLump("\n{\n", 3);
	WriteProps(out, sec->props);
	out.AddToLump("}\n\n", 3);
}

void FProcessor::WriteVertexUDMF(FWadWriter& out, IntVertex* vt, int num) {
	out.AddToLump("vertex", 6);
	if (this->write_comments) {
		char buffer[32];
		int len = sprintf(buffer, " // %d", num);
		out.AddToLump(buffer, len);
	}
	out.AddToLump("\n{\n", 3);
	WriteProps(out, vt->props);
	out.AddToLump("}\n\n", 3);
}

void FProcessor::WriteTextMap(FWadWriter& out) {
	out.StartWritingLump("TEXTMAP");
	WriteProps(out, Level.props);
	for (int i = 0; i < Level.NumThings(); i++) {
		WriteThingUDMF(out, &Level.Things[i], i);
	}

	for (int i = 0; i < Level.NumOrgVerts; i++) {
		znbx_VertexEx* vt = &Level.Vertices[i];
		if (vt->index <= 0) {
			// not valid!
			throw std::runtime_error("Invalid vertex data.");
		}
		WriteVertexUDMF(out, &Level.VertexProps[vt->index - 1], i);
	}

	for (int i = 0; i < Level.NumLines(); i++) {
		WriteLinedefUDMF(out, &Level.Lines[i], i);
	}

	for (int i = 0; i < Level.NumSides(); i++) {
		WriteSidedefUDMF(out, &Level.Sides[i], i);
	}

	for (int i = 0; i < Level.NumSectors(); i++) {
		WriteSectorUDMF(out, &Level.Sectors[i], i);
	}
}
