/// @file
/// @brief A supplement to zacs.h for tying the C interface to the implementation.

#include "zacs.h"

#include "tarray.hpp"
#include "zstring.hpp"

#pragma once

#define LOCAL_SIZE 20
#define NUM_MAPVARS 128

// P-codes for ACS scripts
enum {
	/*  0*/ PCD_NOP,
	PCD_TERMINATE,
	PCD_SUSPEND,
	PCD_PUSHNUMBER,
	PCD_LSPEC1,
	PCD_LSPEC2,
	PCD_LSPEC3,
	PCD_LSPEC4,
	PCD_LSPEC5,
	PCD_LSPEC1DIRECT,
	/* 10*/ PCD_LSPEC2DIRECT,
	PCD_LSPEC3DIRECT,
	PCD_LSPEC4DIRECT,
	PCD_LSPEC5DIRECT,
	PCD_ADD,
	PCD_SUBTRACT,
	PCD_MULTIPLY,
	PCD_DIVIDE,
	PCD_MODULUS,
	PCD_EQ,
	/* 20*/ PCD_NE,
	PCD_LT,
	PCD_GT,
	PCD_LE,
	PCD_GE,
	PCD_ASSIGNSCRIPTVAR,
	PCD_ASSIGNMAPVAR,
	PCD_ASSIGNWORLDVAR,
	PCD_PUSHSCRIPTVAR,
	PCD_PUSHMAPVAR,
	/* 30*/ PCD_PUSHWORLDVAR,
	PCD_ADDSCRIPTVAR,
	PCD_ADDMAPVAR,
	PCD_ADDWORLDVAR,
	PCD_SUBSCRIPTVAR,
	PCD_SUBMAPVAR,
	PCD_SUBWORLDVAR,
	PCD_MULSCRIPTVAR,
	PCD_MULMAPVAR,
	PCD_MULWORLDVAR,
	/* 40*/ PCD_DIVSCRIPTVAR,
	PCD_DIVMAPVAR,
	PCD_DIVWORLDVAR,
	PCD_MODSCRIPTVAR,
	PCD_MODMAPVAR,
	PCD_MODWORLDVAR,
	PCD_INCSCRIPTVAR,
	PCD_INCMAPVAR,
	PCD_INCWORLDVAR,
	PCD_DECSCRIPTVAR,
	/* 50*/ PCD_DECMAPVAR,
	PCD_DECWORLDVAR,
	PCD_GOTO,
	PCD_IFGOTO,
	PCD_DROP,
	PCD_DELAY,
	PCD_DELAYDIRECT,
	PCD_RANDOM,
	PCD_RANDOMDIRECT,
	PCD_THINGCOUNT,
	/* 60*/ PCD_THINGCOUNTDIRECT,
	PCD_TAGWAIT,
	PCD_TAGWAITDIRECT,
	PCD_POLYWAIT,
	PCD_POLYWAITDIRECT,
	PCD_CHANGEFLOOR,
	PCD_CHANGEFLOORDIRECT,
	PCD_CHANGECEILING,
	PCD_CHANGECEILINGDIRECT,
	PCD_RESTART,
	/* 70*/ PCD_ANDLOGICAL,
	PCD_ORLOGICAL,
	PCD_ANDBITWISE,
	PCD_ORBITWISE,
	PCD_EORBITWISE,
	PCD_NEGATELOGICAL,
	PCD_LSHIFT,
	PCD_RSHIFT,
	PCD_UNARYMINUS,
	PCD_IFNOTGOTO,
	/* 80*/ PCD_LINESIDE,
	PCD_SCRIPTWAIT,
	PCD_SCRIPTWAITDIRECT,
	PCD_CLEARLINESPECIAL,
	PCD_CASEGOTO,
	PCD_BEGINPRINT,
	PCD_ENDPRINT,
	PCD_PRINTSTRING,
	PCD_PRINTNUMBER,
	PCD_PRINTCHARACTER,
	/* 90*/ PCD_PLAYERCOUNT,
	PCD_GAMETYPE,
	PCD_GAMESKILL,
	PCD_TIMER,
	PCD_SECTORSOUND,
	PCD_AMBIENTSOUND,
	PCD_SOUNDSEQUENCE,
	PCD_SETLINETEXTURE,
	PCD_SETLINEBLOCKING,
	PCD_SETLINESPECIAL,
	/*100*/ PCD_THINGSOUND,
	PCD_ENDPRINTBOLD, // [RH] End of Hexen p-codes
	PCD_ACTIVATORSOUND,
	PCD_LOCALAMBIENTSOUND,
	PCD_SETLINEMONSTERBLOCKING,
	PCD_PLAYERBLUESKULL, // [BC] Start of new [Skull Tag] pcodes
	PCD_PLAYERREDSKULL,
	PCD_PLAYERYELLOWSKULL,
	PCD_PLAYERMASTERSKULL,
	PCD_PLAYERBLUECARD,
	/*110*/ PCD_PLAYERREDCARD,
	PCD_PLAYERYELLOWCARD,
	PCD_PLAYERMASTERCARD,
	PCD_PLAYERBLACKSKULL,
	PCD_PLAYERSILVERSKULL,
	PCD_PLAYERGOLDSKULL,
	PCD_PLAYERBLACKCARD,
	PCD_PLAYERSILVERCARD,
	PCD_ISNETWORKGAME,
	PCD_PLAYERTEAM,
	/*120*/ PCD_PLAYERHEALTH,
	PCD_PLAYERARMORPOINTS,
	PCD_PLAYERFRAGS,
	PCD_PLAYEREXPERT,
	PCD_BLUETEAMCOUNT,
	PCD_REDTEAMCOUNT,
	PCD_BLUETEAMSCORE,
	PCD_REDTEAMSCORE,
	PCD_ISONEFLAGCTF,
	PCD_LSPEC6, // These are never used. They should probably
	/*130*/ PCD_LSPEC6DIRECT, // be given names like PCD_DUMMY.
	PCD_PRINTNAME,
	PCD_MUSICCHANGE,
	PCD_CONSOLECOMMANDDIRECT,
	PCD_CONSOLECOMMAND,
	PCD_SINGLEPLAYER, // [RH] End of Skull Tag p-codes
	PCD_FIXEDMUL,
	PCD_FIXEDDIV,
	PCD_SETGRAVITY,
	PCD_SETGRAVITYDIRECT,
	/*140*/ PCD_SETAIRCONTROL,
	PCD_SETAIRCONTROLDIRECT,
	PCD_CLEARINVENTORY,
	PCD_GIVEINVENTORY,
	PCD_GIVEINVENTORYDIRECT,
	PCD_TAKEINVENTORY,
	PCD_TAKEINVENTORYDIRECT,
	PCD_CHECKINVENTORY,
	PCD_CHECKINVENTORYDIRECT,
	PCD_SPAWN,
	/*150*/ PCD_SPAWNDIRECT,
	PCD_SPAWNSPOT,
	PCD_SPAWNSPOTDIRECT,
	PCD_SETMUSIC,
	PCD_SETMUSICDIRECT,
	PCD_LOCALSETMUSIC,
	PCD_LOCALSETMUSICDIRECT,
	PCD_PRINTFIXED,
	PCD_PRINTLOCALIZED,
	PCD_MOREHUDMESSAGE,
	/*160*/ PCD_OPTHUDMESSAGE,
	PCD_ENDHUDMESSAGE,
	PCD_ENDHUDMESSAGEBOLD,
	PCD_SETSTYLE,
	PCD_SETSTYLEDIRECT,
	PCD_SETFONT,
	PCD_SETFONTDIRECT,
	PCD_PUSHBYTE,
	PCD_LSPEC1DIRECTB,
	PCD_LSPEC2DIRECTB,
	/*170*/ PCD_LSPEC3DIRECTB,
	PCD_LSPEC4DIRECTB,
	PCD_LSPEC5DIRECTB,
	PCD_DELAYDIRECTB,
	PCD_RANDOMDIRECTB,
	PCD_PUSHBYTES,
	PCD_PUSH2BYTES,
	PCD_PUSH3BYTES,
	PCD_PUSH4BYTES,
	PCD_PUSH5BYTES,
	/*180*/ PCD_SETTHINGSPECIAL,
	PCD_ASSIGNGLOBALVAR,
	PCD_PUSHGLOBALVAR,
	PCD_ADDGLOBALVAR,
	PCD_SUBGLOBALVAR,
	PCD_MULGLOBALVAR,
	PCD_DIVGLOBALVAR,
	PCD_MODGLOBALVAR,
	PCD_INCGLOBALVAR,
	PCD_DECGLOBALVAR,
	/*190*/ PCD_FADETO,
	PCD_FADERANGE,
	PCD_CANCELFADE,
	PCD_PLAYMOVIE,
	PCD_SETFLOORTRIGGER,
	PCD_SETCEILINGTRIGGER,
	PCD_GETACTORX,
	PCD_GETACTORY,
	PCD_GETACTORZ,
	PCD_STARTTRANSLATION,
	/*200*/ PCD_TRANSLATIONRANGE1,
	PCD_TRANSLATIONRANGE2,
	PCD_ENDTRANSLATION,
	PCD_CALL,
	PCD_CALLDISCARD,
	PCD_RETURNVOID,
	PCD_RETURNVAL,
	PCD_PUSHMAPARRAY,
	PCD_ASSIGNMAPARRAY,
	PCD_ADDMAPARRAY,
	/*210*/ PCD_SUBMAPARRAY,
	PCD_MULMAPARRAY,
	PCD_DIVMAPARRAY,
	PCD_MODMAPARRAY,
	PCD_INCMAPARRAY,
	PCD_DECMAPARRAY,
	PCD_DUP,
	PCD_SWAP,
	PCD_WRITETOINI,
	PCD_GETFROMINI,
	/*220*/ PCD_SIN,
	PCD_COS,
	PCD_VECTORANGLE,
	PCD_CHECKWEAPON,
	PCD_SETWEAPON,
	PCD_TAGSTRING,
	PCD_PUSHWORLDARRAY,
	PCD_ASSIGNWORLDARRAY,
	PCD_ADDWORLDARRAY,
	PCD_SUBWORLDARRAY,
	/*230*/ PCD_MULWORLDARRAY,
	PCD_DIVWORLDARRAY,
	PCD_MODWORLDARRAY,
	PCD_INCWORLDARRAY,
	PCD_DECWORLDARRAY,
	PCD_PUSHGLOBALARRAY,
	PCD_ASSIGNGLOBALARRAY,
	PCD_ADDGLOBALARRAY,
	PCD_SUBGLOBALARRAY,
	PCD_MULGLOBALARRAY,
	/*240*/ PCD_DIVGLOBALARRAY,
	PCD_MODGLOBALARRAY,
	PCD_INCGLOBALARRAY,
	PCD_DECGLOBALARRAY,
	PCD_SETMARINEWEAPON,
	PCD_SETACTORPROPERTY,
	PCD_GETACTORPROPERTY,
	PCD_PLAYERNUMBER,
	PCD_ACTIVATORTID,
	PCD_SETMARINESPRITE,
	/*250*/ PCD_GETSCREENWIDTH,
	PCD_GETSCREENHEIGHT,
	PCD_THING_PROJECTILE2,
	PCD_STRLEN,
	PCD_SETHUDSIZE,
	PCD_GETCVAR,
	PCD_CASEGOTOSORTED,
	PCD_SETRESULTVALUE,
	PCD_GETLINEROWOFFSET,
	PCD_GETACTORFLOORZ,
	/*260*/ PCD_GETACTORANGLE,
	PCD_GETSECTORFLOORZ,
	PCD_GETSECTORCEILINGZ,
	PCD_LSPEC5RESULT,
	PCD_GETSIGILPIECES,
	PCD_GETLEVELINFO,
	PCD_CHANGESKY,
	PCD_PLAYERINGAME,
	PCD_PLAYERISBOT,
	PCD_SETCAMERATOTEXTURE,
	/*270*/ PCD_ENDLOG,
	PCD_GETAMMOCAPACITY,
	PCD_SETAMMOCAPACITY,
	PCD_PRINTMAPCHARARRAY, // [JB] start of new p-codes
	PCD_PRINTWORLDCHARARRAY,
	PCD_PRINTGLOBALCHARARRAY, // [JB] end of new p-codes
	PCD_SETACTORANGLE, // [GRB]
	PCD_GRABINPUT, // Unused but acc defines them
	PCD_SETMOUSEPOINTER, // "
	PCD_MOVEMOUSEPOINTER, // "
	/*280*/ PCD_SPAWNPROJECTILE,
	PCD_GETSECTORLIGHTLEVEL,
	PCD_GETACTORCEILINGZ,
	PCD_SETACTORPOSITION,
	PCD_CLEARACTORINVENTORY,
	PCD_GIVEACTORINVENTORY,
	PCD_TAKEACTORINVENTORY,
	PCD_CHECKACTORINVENTORY,
	PCD_THINGCOUNTNAME,
	PCD_SPAWNSPOTFACING,
	/*290*/ PCD_PLAYERCLASS, // [GRB]
	//[MW] start my p-codes
	PCD_ANDSCRIPTVAR,
	PCD_ANDMAPVAR,
	PCD_ANDWORLDVAR,
	PCD_ANDGLOBALVAR,
	PCD_ANDMAPARRAY,
	PCD_ANDWORLDARRAY,
	PCD_ANDGLOBALARRAY,
	PCD_EORSCRIPTVAR,
	PCD_EORMAPVAR,
	/*300*/ PCD_EORWORLDVAR,
	PCD_EORGLOBALVAR,
	PCD_EORMAPARRAY,
	PCD_EORWORLDARRAY,
	PCD_EORGLOBALARRAY,
	PCD_ORSCRIPTVAR,
	PCD_ORMAPVAR,
	PCD_ORWORLDVAR,
	PCD_ORGLOBALVAR,
	PCD_ORMAPARRAY,
	/*310*/ PCD_ORWORLDARRAY,
	PCD_ORGLOBALARRAY,
	PCD_LSSCRIPTVAR,
	PCD_LSMAPVAR,
	PCD_LSWORLDVAR,
	PCD_LSGLOBALVAR,
	PCD_LSMAPARRAY,
	PCD_LSWORLDARRAY,
	PCD_LSGLOBALARRAY,
	PCD_RSSCRIPTVAR,
	/*320*/ PCD_RSMAPVAR,
	PCD_RSWORLDVAR,
	PCD_RSGLOBALVAR,
	PCD_RSMAPARRAY,
	PCD_RSWORLDARRAY,
	PCD_RSGLOBALARRAY,
	//[MW] end my p-codes
	PCD_GETPLAYERINFO, // [GRB]
	PCD_CHANGELEVEL,
	PCD_SECTORDAMAGE,
	PCD_REPLACETEXTURES,
	/*330*/ PCD_NEGATEBINARY,
	PCD_GETACTORPITCH,
	PCD_SETACTORPITCH,
	PCD_PRINTBIND,
	PCD_SETACTORSTATE,
	PCD_THINGDAMAGE2,
	PCD_USEINVENTORY,
	PCD_USEACTORINVENTORY,
	PCD_CHECKACTORCEILINGTEXTURE,
	PCD_CHECKACTORFLOORTEXTURE,
	/*340*/ PCD_GETACTORLIGHTLEVEL,
	PCD_SETMUGSHOTSTATE,
	PCD_THINGCOUNTSECTOR,
	PCD_THINGCOUNTNAMESECTOR,
	PCD_CHECKPLAYERCAMERA, // [TN]
	PCD_MORPHACTOR, // [MH]
	PCD_UNMORPHACTOR, // [MH]
	PCD_GETPLAYERINPUT,
	PCD_CLASSIFYACTOR,
	PCD_PRINTBINARY,
	/*350*/ PCD_PRINTHEX,
	PCD_CALLFUNC,
	PCD_SAVESTRING, // [FDARI] create string (temporary)
	PCD_PRINTMAPCHRANGE, // [FDARI] output range (print part of array)
	PCD_PRINTWORLDCHRANGE,
	PCD_PRINTGLOBALCHRANGE,
	PCD_STRCPYTOMAPCHRANGE, // [FDARI] input range (copy string to all/part of array)
	PCD_STRCPYTOWORLDCHRANGE,
	PCD_STRCPYTOGLOBALCHRANGE,
	PCD_PUSHFUNCTION, // from Eternity
	/*360*/ PCD_CALLSTACK, // from Eternity
	PCD_SCRIPTWAITNAMED,
	PCD_TRANSLATIONRANGE3,
	PCD_GOTOSTACK,
	PCD_ASSIGNSCRIPTARRAY,
	PCD_PUSHSCRIPTARRAY,
	PCD_ADDSCRIPTARRAY,
	PCD_SUBSCRIPTARRAY,
	PCD_MULSCRIPTARRAY,
	PCD_DIVSCRIPTARRAY,
	/*370*/ PCD_MODSCRIPTARRAY,
	PCD_INCSCRIPTARRAY,
	PCD_DECSCRIPTARRAY,
	PCD_ANDSCRIPTARRAY,
	PCD_EORSCRIPTARRAY,
	PCD_ORSCRIPTARRAY,
	PCD_LSSCRIPTARRAY,
	PCD_RSSCRIPTARRAY,
	PCD_PRINTSCRIPTCHARARRAY,
	PCD_PRINTSCRIPTCHRANGE,
	/*380*/ PCD_STRCPYTOSCRIPTCHRANGE,
	PCD_LSPEC5EX,
	PCD_LSPEC5EXRESULT,
	PCD_TRANSLATIONRANGE4,
	PCD_TRANSLATIONRANGE5,

	/*381*/ PCODE_COMMAND_COUNT
};

// Some constants used by ACS scripts
enum {
	LINE_FRONT = 0,
	LINE_BACK = 1
};

enum {
	SIDE_FRONT = 0,
	SIDE_BACK = 1
};

enum {
	TEXTURE_TOP = 0,
	TEXTURE_MIDDLE = 1,
	TEXTURE_BOTTOM = 2
};

enum {
	GAME_SINGLE_PLAYER = 0,
	GAME_NET_COOPERATIVE = 1,
	GAME_NET_DEATHMATCH = 2,
	GAME_TITLE_MAP = 3
};

enum {
	CLASS_FIGHTER = 0,
	CLASS_CLERIC = 1,
	CLASS_MAGE = 2
};

enum {
	SKILL_VERY_EASY = 0,
	SKILL_EASY = 1,
	SKILL_NORMAL = 2,
	SKILL_HARD = 3,
	SKILL_VERY_HARD = 4
};

enum {
	BLOCK_NOTHING = 0,
	BLOCK_CREATURES = 1,
	BLOCK_EVERYTHING = 2,
	BLOCK_RAILING = 3,
	BLOCK_PLAYERS = 4
};

enum {
	LEVELINFO_PAR_TIME,
	LEVELINFO_CLUSTERNUM,
	LEVELINFO_LEVELNUM,
	LEVELINFO_TOTAL_SECRETS,
	LEVELINFO_FOUND_SECRETS,
	LEVELINFO_TOTAL_ITEMS,
	LEVELINFO_FOUND_ITEMS,
	LEVELINFO_TOTAL_MONSTERS,
	LEVELINFO_KILLED_MONSTERS,
	LEVELINFO_SUCK_TIME
};

enum {
	PLAYERINFO_TEAM,
	PLAYERINFO_AIMDIST,
	PLAYERINFO_COLOR,
	PLAYERINFO_GENDER,
	PLAYERINFO_NEVERSWITCH,
	PLAYERINFO_MOVEBOB,
	PLAYERINFO_STILLBOB,
	PLAYERINFO_PLAYERCLASS,
	PLAYERINFO_FOV,
	PLAYERINFO_DESIREDFOV,
	PLAYERINFO_FVIEWBOB,
};

enum {
	NUM_WORLDVARS = 256,
	NUM_GLOBALVARS = 64
};

struct InitIntToZero {
	void Init(int& v) {
		v = 0;
	}
};

typedef TMap<int32_t, int32_t, THashTraits<int32_t>, InitIntToZero> FWorldGlobalArray;

// Type of elements count is unsigned int instead of size_t to match ACSStringPool interface
template<typename T, unsigned int N>
struct BoundsCheckingArray {
	T& operator[](const unsigned int index) {
		if (index >= N) {
#warning "TODO: handle this case ('Out of bounds access to local variables in ACS VM')"
		}

		return buffer[index];
	}

	T* Pointer() {
		return buffer;
	}
	unsigned int Size() const {
		return N;
	}

	void Fill(const T& value) {
		std::fill(std::begin(buffer), std::end(buffer), value);
	}

private:
	T buffer[N];
};

// ACS variables with global scope
extern BoundsCheckingArray<int32_t, NUM_GLOBALVARS> ACS_GlobalVars;
extern BoundsCheckingArray<FWorldGlobalArray, NUM_GLOBALVARS> ACS_GlobalArrays;

#define LIBRARYID_MASK 0xFFF00000
#define LIBRARYID_SHIFT 20

// Global ACS string table
#define STRPOOL_LIBRARYID (INT_MAX >> LIBRARYID_SHIFT)
#define STRPOOL_LIBRARYID_OR (STRPOOL_LIBRARYID << LIBRARYID_SHIFT)

class ACSStringPool {
public:
	ACSStringPool();
	int AddString(const char* str);
	int AddString(FString& str);
	const char* GetString(int strnum);
	void LockString(int levelnum, int strnum);
	void UnlockAll();
	void MarkString(int strnum);
	void LockStringArray(int levelnum, const int* strnum, unsigned int count);
	void MarkStringArray(const int* strnum, unsigned int count);
	void MarkStringMap(const FWorldGlobalArray& array);
	void PurgeStrings();
	void Clear();
	void Dump(void* ctx, void (*callback)(void*, uint32_t lock_size, const char* str)) const;
	void UnlockForLevel(int level);

private:
	int FindString(const char* str, size_t len, unsigned int h, unsigned int bucketnum);
	int InsertString(FString& str, unsigned int h, unsigned int bucketnum);
	void FindFirstFreeEntry(unsigned int base);

	enum {
		NUM_BUCKETS = 251
	};
	enum {
		FREE_ENTRY = 0xFFFFFFFE
	}; // Stored in PoolEntry's Next field
	enum {
		NO_ENTRY = 0xFFFFFFFF
	};
	enum {
		MIN_GC_SIZE = 100
	}; // Don't auto-collect until there are this many strings

	struct PoolEntry {
		FString Str;
		unsigned int Hash;
		unsigned int Next = FREE_ENTRY;
		bool Mark;
		TArray<int> Locks;

		void Lock(int levelnum);
		void Unlock(int levelnum);
	};

	TArray<PoolEntry> Pool;
	unsigned int PoolBuckets[NUM_BUCKETS];
	unsigned int FirstFreeEntry;
};
extern ACSStringPool GlobalACSStrings;

void P_CollectACSGlobalStrings();
void P_ClearACSVars(bool);

struct ACSProfileInfo {
	unsigned long long TotalInstr;
	unsigned int NumRuns;
	unsigned int MinInstrPerRun;
	unsigned int MaxInstrPerRun;

	ACSProfileInfo();
	void AddRun(unsigned int num_instr);
	void Reset();
};

struct ProfileCollector {
	ACSProfileInfo* ProfileData;
	class FBehavior* Module;
	int Index;
};

class ACSLocalVariables {
public:
	ACSLocalVariables(TArray<int32_t>& variables)
		: memory(&variables[0]), count(variables.Size()) { }

	void Reset(int32_t* const memory, const size_t count) {
		// TODO: pointer sanity check?
		// TODO: constraints on count?

		this->memory = memory;
		this->count = count;
	}

	int32_t& operator[](const size_t index) {
		if (index >= count) {
#warning "TODO: handle this case ('Out of bounds access to local variables in ACS VM')"
		}

		return memory[index];
	}

	const int32_t* GetPointer() const {
		return memory;
	}

private:
	int32_t* memory;
	size_t count;
};

struct ACSLocalArrayInfo {
	unsigned int Size;
	int Offset;
};

struct ACSLocalArrays {
	unsigned int Count;
	ACSLocalArrayInfo* Info;

	ACSLocalArrays() {
		Count = 0;
		Info = NULL;
	}
	~ACSLocalArrays() {
		if (Info != NULL) {
			delete[] Info;
			Info = NULL;
		}
	}

	// Bounds-checking Set and Get for local arrays
	void Set(ACSLocalVariables& locals, int arraynum, int arrayentry, int value) {
		if ((unsigned int)arraynum < Count && (unsigned int)arrayentry < Info[arraynum].Size) {
			locals[Info[arraynum].Offset + arrayentry] = value;
		}
	}
	int Get(ACSLocalVariables& locals, int arraynum, int arrayentry) {
		if ((unsigned int)arraynum < Count && (unsigned int)arrayentry < Info[arraynum].Size) {
			return locals[Info[arraynum].Offset + arrayentry];
		}
		return 0;
	}
};

// The in-memory version
struct ScriptPtr {
	int Number;
	uint32_t Address;
	uint8_t Type;
	uint8_t ArgCount;
	uint16_t VarCount;
	uint16_t Flags;
	ACSLocalArrays LocalArrays;

	ACSProfileInfo ProfileData;
};

// The present ZDoom version
struct ScriptPtr3 {
	int16_t Number;
	uint8_t Type;
	uint8_t ArgCount;
	uint32_t Address;
};

// The intermediate ZDoom version
struct ScriptPtr1 {
	int16_t Number;
	uint16_t Type;
	uint32_t Address;
	uint32_t ArgCount;
};

// The old Hexen version
struct ScriptPtr2 {
	uint32_t Number; // Type is Number / 1000
	uint32_t Address;
	uint32_t ArgCount;
};

struct ScriptFlagsPtr {
	uint16_t Number;
	uint16_t Flags;
};

struct ScriptFunctionInFile {
	uint8_t ArgCount;
	uint8_t LocalCount;
	uint8_t HasReturnValue;
	uint8_t ImportNum;
	uint32_t Address;
};

struct ScriptFunction {
	uint8_t ArgCount;
	uint8_t HasReturnValue;
	uint8_t ImportNum;
	int LocalCount;
	uint32_t Address;
	ACSLocalArrays LocalArrays;
};

// Script types
enum {
	SCRIPT_Closed = 0,
	SCRIPT_Open = 1,
	SCRIPT_Respawn = 2,
	SCRIPT_Death = 3,
	SCRIPT_Enter = 4,
	SCRIPT_Pickup = 5,
	SCRIPT_BlueReturn = 6,
	SCRIPT_RedReturn = 7,
	SCRIPT_WhiteReturn = 8,
	SCRIPT_Lightning = 12,
	SCRIPT_Unloading = 13,
	SCRIPT_Disconnect = 14,
	SCRIPT_Return = 15,
	SCRIPT_Event = 16, // [BB]
	SCRIPT_Kill = 17, // [JM]
	SCRIPT_Reopen = 18, // [Nash]
};

// Script flags
enum {
	SCRIPTF_Net = 0x0001 // Safe to "puke" in multiplayer
};

enum ACSFormat {
	ACS_Old,
	ACS_Enhanced,
	ACS_LittleEnhanced,
	ACS_Unknown
};

class FBehavior {
public:
	FBehavior();
	~FBehavior();
	bool Init(FBehaviorContainer& ctr, const zacs_SliceU8 slice, zacs_ModuleLoader mloader);

	bool IsGood();
	uint8_t* FindChunk(uint32_t id) const;
	uint8_t* NextChunk(uint8_t* chunk) const;
	const ScriptPtr* FindScript(int number) const;

	uint32_t PC2Ofs(int* pc) const {
		return (uint32_t)((uint8_t*)pc - Data);
	}

	int* Ofs2PC(uint32_t ofs) const {
		return (int*)(Data + ofs);
	}

	int* Jump2PC(uint32_t jumpPoint) const {
		return Ofs2PC(JumpPoints[jumpPoint]);
	}

	ACSFormat GetFormat() const {
		return Format;
	}

	ScriptFunction* GetFunction(int funcnum, FBehavior*& module) const;
	int GetArrayVal(int arraynum, int index) const;
	void SetArrayVal(int arraynum, int index, int value);
	inline bool CopyStringToArray(int arraynum, int index, int maxLength, const char* string);

	int FindFunctionName(const char* funcname) const;
	int FindMapVarName(const char* varname) const;
	int FindMapArray(const char* arrayname) const;

	int GetLibraryID() const {
		return LibraryID;
	}

	int* GetScriptAddress(const ScriptPtr* ptr) const {
		return (int*)(ptr->Address + Data);
	}

	int GetScriptIndex(const ScriptPtr* ptr) const {
		ptrdiff_t index = ptr - Scripts;
		return index >= NumScripts ? -1 : (int)index;
	}

	ScriptPtr* GetScriptPtr(int index) const {
		return index >= 0 && index < NumScripts ? &Scripts[index] : NULL;
	}

	int GetDataSize() const {
		return DataSize;
	}

	const char* GetModuleName() const {
		return ModuleName;
	}

	ACSProfileInfo* GetFunctionProfileData(int index) {
		return index >= 0 && index < NumFunctions ? &FunctionProfileData[index] : NULL;
	}

	ACSProfileInfo* GetFunctionProfileData(ScriptFunction* func) {
		return GetFunctionProfileData((int)(func - (ScriptFunction*)Functions));
	}

	const char* LookupString(uint32_t index, bool forprint = false) const;

	BoundsCheckingArray<int32_t*, NUM_MAPVARS> MapVars;

private:
	struct ArrayInfo;

	uint8_t* Data;
	uint8_t* Chunks;
	ScriptPtr* Scripts;
	ScriptFunction* Functions;
	ACSProfileInfo* FunctionProfileData;
	ArrayInfo* ArrayStore;
	ArrayInfo** Arrays;

	ACSFormat Format;
	int DataSize;
	int NumScripts;
	int NumFunctions;
	int NumArrays;
	int NumTotalArrays;
	uint32_t StringTable;
	uint32_t LibraryID;
	bool ShouldLocalize;

	int32_t MapVarStore[NUM_MAPVARS];
	TArray<FBehavior*> Imports;
	char ModuleName[9];
	TArray<int> JumpPoints;

	void LoadScriptsDirectory();

	static int SortScripts(const void* a, const void* b);
	void UnencryptStrings();
	void UnescapeStringTable(uint8_t* chunkstart, uint8_t* datastart, bool haspadding);
	int FindStringInChunk(uint32_t* chunk, const char* varname) const;

	void MarkMapVarStrings() const;
	void LockMapVarStrings(int levelnum) const;

	friend struct FBehaviorContainer;
};

struct FBehaviorContainer {
	TArray<FBehavior*> StaticModules;

	FBehaviorContainer() = default;

	FBehavior* LoadModule(const zacs_SliceU8 slice, zacs_ModuleLoader mloader);

	void LoadDefaultModules();
	void UnloadModules();
	bool CheckAllGood();
	FBehavior* GetModule(int lib);
	void MarkLevelVarStrings();
	void LockLevelVarStrings(int levelnum);
	void UnlockLevelVarStrings(int levelnum);

	const ScriptPtr* FindScript(int script, FBehavior*& module);
	const char* LookupString(uint32_t index, bool forprint = false);
	void ArrangeScriptProfiles(TArray<ProfileCollector>& profiles);
	void ArrangeFunctionProfiles(TArray<ProfileCollector>& profiles);
};

class DLevelScript;

class DACSThinker {
public:
	void Construct() { }
	~DACSThinker();

	void Tick();

	typedef TMap<int, DLevelScript*> ScriptMap;
	ScriptMap RunningScripts; // Array of all synchronous scripts

	void DumpScriptStatus();

private:
	DLevelScript* LastScript = nullptr;
	DLevelScript* Scripts = nullptr; // List of all running scripts

	friend class DLevelScript;
	friend class FBehavior;
	friend struct FBehaviorContainer;
};
