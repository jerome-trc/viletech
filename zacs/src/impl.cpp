/*

Copyright 1998-2012 Randy Heit
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions
are met:

1. Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the distribution.
3. The name of the author may not be used to endorse or promote products
   derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT,
INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

[RH] This code at one time made lots of little-endian assumptions.
I think it should be fine on big-endian machines now, but I have no
real way to test it.

*/

#include <assert.h>
#include <climits>

#include "zacs.hpp"

#include "common.hpp"
#include "name.hpp"

// [RH] I imagine this much stack space is probably overkill, but it could
// potentially get used with recursive functions.
#define STACK_SIZE 4096

// HUD message flags
#define HUDMSG_LOG					(0x80000000)
#define HUDMSG_COLORSTRING			(0x40000000)
#define HUDMSG_ADDBLEND				(0x20000000)
#define HUDMSG_ALPHA				(0x10000000)
#define HUDMSG_NOWRAP				(0x08000000)

// HUD message layers; these are not flags
#define HUDMSG_LAYER_SHIFT			12
#define HUDMSG_LAYER_MASK			(0x0000F000)
// See HUDMSGLayer enumerations in sbar.h

// HUD message visibility flags
#define HUDMSG_VISIBILITY_SHIFT		16
#define HUDMSG_VISIBILITY_MASK		(0x00070000)
// See HUDMSG visibility enumerations in sbar.h

// LineAttack flags
#define FHF_NORANDOMPUFFZ	1
#define FHF_NOIMPACTDECAL	2

// SpawnDecal flags
#define SDF_ABSANGLE		1
#define SDF_PERMANENT		2
#define SDF_FIXED_ZOFF		4
#define SDF_FIXED_DISTANCE	8

// GetArmorInfo
enum
{
	ARMORINFO_CLASSNAME,
	ARMORINFO_SAVEAMOUNT,
	ARMORINFO_SAVEPERCENT,
	ARMORINFO_MAXABSORB,
	ARMORINFO_MAXFULLABSORB,
	ARMORINFO_ACTUALSAVEAMOUNT,
};

// PickActor
// [JP] I've renamed these flags to something else to avoid confusion with the other PAF_ flags
enum
{
//	PAF_FORCETID,
//	PAF_RETURNTID
	PICKAF_FORCETID = 1,
	PICKAF_RETURNTID = 2,
};

// ACS specific conversion functions to and from fixed point.
// These should be used to convert from and to sctipt variables
// so that there is a clear distinction between leftover fixed point code
// and genuinely needed conversions.

struct FBehavior::ArrayInfo
{
	uint32_t ArraySize;
	int32_t *Elements;
};

//============================================================================
//
// uallong
//
// Read a possibly unaligned four-byte little endian integer from memory.
//
//============================================================================

#if defined(_M_IX86) || defined(_M_X64) || defined(__i386__) || defined(__x86_64__)
inline int uallong(const int &foo)
{
	return foo;
}
#else
inline int uallong(const int &foo)
{
	const unsigned char *bar = (const unsigned char *)&foo;
	return bar[0] | (bar[1] << 8) | (bar[2] << 16) | (bar[3] << 24);
}
#endif

//============================================================================
//
// Global and world variables
//
//============================================================================

// ACS variables with world scope
static BoundsCheckingArray<int32_t, NUM_WORLDVARS> ACS_WorldVars;
static BoundsCheckingArray<FWorldGlobalArray, NUM_WORLDVARS> ACS_WorldArrays;

// ACS variables with global scope
BoundsCheckingArray<int32_t, NUM_GLOBALVARS> ACS_GlobalVars;
BoundsCheckingArray<FWorldGlobalArray, NUM_GLOBALVARS> ACS_GlobalArrays;

//----------------------------------------------------------------------------
//
// ACS stack manager
//
// This is needed so that the garbage collector has access to all active
// script stacks
//
//----------------------------------------------------------------------------

using FACSStackMemory = BoundsCheckingArray<int32_t, STACK_SIZE>;

struct FACSStack
{
	FACSStackMemory buffer;
	int sp;
	FACSStack *next;
	FACSStack *prev;
	static FACSStack *head;

	FACSStack();
	~FACSStack();
};

FACSStack *FACSStack::head;

FACSStack::FACSStack()
{
	sp = 0;
	next = head;
	prev = NULL;
	head = this;
}

FACSStack::~FACSStack()
{
	if (next != NULL) next->prev = prev;
	if (prev == NULL)
	{
		head = next;
	}
	else
	{
		prev->next = next;
	}
}

//----------------------------------------------------------------------------
//
// Global ACS strings (Formerly known as On the fly strings)
//
// This special string table is part of the global state. Programmatically
// generated strings (e.g. those returned by strparam) are stored here.
// PCD_TAGSTRING also now stores strings in this table instead of simply
// tagging strings with their library ID.
//
// Identical strings map to identical string identifiers.
//
// When the string table needs to grow to hold more strings, a garbage
// collection is first attempted to see if more room can be made to store
// strings without growing. A string is considered in use if any value
// in any of these variable blocks contains a valid ID in the global string
// table:
//   * The active area of the ACS stack
//   * All running scripts' local variables
//   * All map variables
//   * All world variables
//   * All global variables
// It's not important whether or not they are really used as strings, only
// that they might be. A string is also considered in use if its lock count
// is non-zero, even if none of the above variable blocks referenced it.
//
// To keep track of local and map variables for nonresident maps in a hub,
// when a map's state is archived, all strings found in its local and map
// variables are locked. When a map is revisited in a hub, all strings found
// in its local and map variables are unlocked. Locking and unlocking are
// cumulative operations.
//
// What this all means is that:
//   * Strings returned by strparam last indefinitely. No longer do they
//     disappear at the end of the tic they were generated.
//   * You can pass library strings around freely without having to worry
//     about always having the same libraries loaded in the same order on
//     every map that needs to use those strings.
//
//----------------------------------------------------------------------------

ACSStringPool GlobalACSStrings;

void ACSStringPool::PoolEntry::Lock(int levelnum)
{
	if (Locks.Find(levelnum) == Locks.Size())
	{
		Locks.Push(levelnum);
	}
}

void ACSStringPool::PoolEntry::Unlock(int levelnum)
{
	auto ndx = Locks.Find(levelnum);
	if (ndx < Locks.Size())
	{
		Locks.Delete(ndx);
	}
}

ACSStringPool::ACSStringPool()
{
	memset(PoolBuckets, 0xFF, sizeof(PoolBuckets));
	FirstFreeEntry = 0;
}

//============================================================================
//
// ACSStringPool :: Clear
//
// Remove all strings from the pool.
//
//============================================================================

void ACSStringPool::Clear()
{
	Pool.Clear();
	memset(PoolBuckets, 0xFF, sizeof(PoolBuckets));
	FirstFreeEntry = 0;
}

//============================================================================
//
// ACSStringPool :: AddString
//
// Returns a valid string identifier (including library ID) or -1 if we ran
// out of room. Identical strings will return identical values.
//
//============================================================================

int ACSStringPool::AddString(const char *str)
{
	if (str == nullptr) str = "";
	size_t len = strlen(str);
	unsigned int h = SuperFastHash(str, len);
	unsigned int bucketnum = h % NUM_BUCKETS;
	int i = FindString(str, len, h, bucketnum);
	if (i >= 0)
	{
		return i | STRPOOL_LIBRARYID_OR;
	}
	FString fstr(str);
	return InsertString(fstr, h, bucketnum);
}

int ACSStringPool::AddString(FString &str)
{
	unsigned int h = SuperFastHash(str.GetChars(), str.Len());
	unsigned int bucketnum = h % NUM_BUCKETS;
	int i = FindString(str.GetChars(), str.Len(), h, bucketnum);
	if (i >= 0)
	{
		return i | STRPOOL_LIBRARYID_OR;
	}
	return InsertString(str, h, bucketnum);
}

//============================================================================
//
// ACSStringPool :: GetString
//
//============================================================================

const char *ACSStringPool::GetString(int strnum)
{
	assert((strnum & LIBRARYID_MASK) == STRPOOL_LIBRARYID_OR);
	strnum &= ~LIBRARYID_MASK;
	if ((unsigned)strnum < Pool.Size() && Pool[strnum].Next != FREE_ENTRY)
	{
		return Pool[strnum].Str.GetChars();
	}
	return NULL;
}

//============================================================================
//
// ACSStringPool :: LockString
//
// Prevents this string from being purged.
//
//============================================================================

void ACSStringPool::LockString(int levelnum, int strnum)
{
	assert((strnum & LIBRARYID_MASK) == STRPOOL_LIBRARYID_OR);
	strnum &= ~LIBRARYID_MASK;
	assert((unsigned)strnum < Pool.Size());
	Pool[strnum].Lock(levelnum);
}

//============================================================================
//
// ACSStringPool :: MarkString
//
// Prevent this string from being purged during the next call to PurgeStrings.
// This does not carry over to subsequent calls of PurgeStrings.
//
//============================================================================

void ACSStringPool::MarkString(int strnum)
{
	assert((strnum & LIBRARYID_MASK) == STRPOOL_LIBRARYID_OR);
	strnum &= ~LIBRARYID_MASK;
	assert((unsigned)strnum < Pool.Size());
	Pool[strnum].Mark = true;
}

//============================================================================
//
// ACSStringPool :: LockStringArray
//
// Prevents several strings from being purged. Entries not in this pool will
// be silently ignored. The idea here is to pass this function a block of
// ACS variables. Everything that looks like it might be a string in the pool
// is locked, even if it's not actually used as such. It's better to keep
// more strings than we need than to throw away ones we do need.
//
//============================================================================

void ACSStringPool::LockStringArray(int levelnum, const int *strnum, unsigned int count)
{
	for (unsigned int i = 0; i < count; ++i)
	{
		int num = strnum[i];
		if ((num & LIBRARYID_MASK) == STRPOOL_LIBRARYID_OR)
		{
			num &= ~LIBRARYID_MASK;
			if ((unsigned)num < Pool.Size())
			{
				Pool[num].Lock(levelnum);
			}
		}
	}
}

//============================================================================
//
// ACSStringPool :: MarkStringArray
//
// Array version of MarkString.
//
//============================================================================

void ACSStringPool::MarkStringArray(const int *strnum, unsigned int count)
{
	for (unsigned int i = 0; i < count; ++i)
	{
		int num = strnum[i];
		if ((num & LIBRARYID_MASK) == STRPOOL_LIBRARYID_OR)
		{
			num &= ~LIBRARYID_MASK;
			if ((unsigned)num < Pool.Size())
			{
				Pool[num].Mark = true;
			}
		}
	}
}

//============================================================================
//
// ACSStringPool :: MarkStringMap
//
// World/global variables version of MarkString.
//
//============================================================================

void ACSStringPool::MarkStringMap(const FWorldGlobalArray &aray)
{
	FWorldGlobalArray::ConstIterator it(aray);
	FWorldGlobalArray::ConstPair *pair;

	while (it.NextPair(pair))
	{
		int num = pair->Value;
		if ((num & LIBRARYID_MASK) == STRPOOL_LIBRARYID_OR)
		{
			num &= ~LIBRARYID_MASK;
			if ((unsigned)num < Pool.Size())
			{
				Pool[num].Mark |= true;
			}
		}
	}
}

//============================================================================
//
// ACSStringPool :: UnlockAll
//
// Resets every entry's lock count to 0. Used when doing a partial reset of
// ACS state such as travelling to a new hub.
//
//============================================================================

void ACSStringPool::UnlockAll()
{
	for (unsigned int i = 0; i < Pool.Size(); ++i)
	{
		Pool[i].Mark = false;
		Pool[i].Locks.Clear();
	}
}

//============================================================================
//
// ACSStringPool :: PurgeStrings
//
// Remove all unlocked strings from the pool.
//
//============================================================================

void ACSStringPool::PurgeStrings()
{
	// Clear the hash buckets. We'll rebuild them as we decide what strings
	// to keep and which to toss.
	memset(PoolBuckets, 0xFF, sizeof(PoolBuckets));
	size_t usedcount = 0, freedcount = 0;
	for (unsigned int i = 0; i < Pool.Size(); ++i)
	{
		PoolEntry *entry = &Pool[i];
		if (entry->Next != FREE_ENTRY)
		{
			if (entry->Locks.Size() == 0 && !entry->Mark)
			{
				freedcount++;
				// Mark this entry as free.
				entry->Next = FREE_ENTRY;
				if (i < FirstFreeEntry)
				{
					FirstFreeEntry = i;
				}
				// And free the string.
				entry->Str = "";
			}
			else
			{
				usedcount++;
				// Rehash this entry.
				unsigned int h = entry->Hash % NUM_BUCKETS;
				entry->Next = PoolBuckets[h];
				PoolBuckets[h] = i;
				// Remove MarkString's mark.
				entry->Mark = false;
			}
		}
	}
}

//============================================================================
//
// ACSStringPool :: FindString
//
// Finds a string in the pool. Does not include the library ID in the returned
// value. Returns -1 if the string does not exist in the pool.
//
//============================================================================

int ACSStringPool::FindString(const char *str, size_t len, unsigned int h, unsigned int bucketnum)
{
	unsigned int i = PoolBuckets[bucketnum];
	while (i != NO_ENTRY)
	{
		PoolEntry *entry = &Pool[i];
		assert(entry->Next != FREE_ENTRY);
		if (entry->Hash == h && entry->Str.Len() == len &&
			memcmp(entry->Str.GetChars(), str, len) == 0)
		{
			return i;
		}
		i = entry->Next;
	}
	return -1;
}

//============================================================================
//
// ACSStringPool :: InsertString
//
// Inserts a new string into the pool.
//
//============================================================================

int ACSStringPool::InsertString(FString &str, unsigned int h, unsigned int bucketnum)
{
	unsigned int index = FirstFreeEntry;
	if (index >= MIN_GC_SIZE && index == Pool.Max())
	{ // We will need to grow the array. Try a garbage collection first.
		P_CollectACSGlobalStrings();
		index = FirstFreeEntry;
	}
	if (FirstFreeEntry >= STRPOOL_LIBRARYID_OR)
	{ // If we go any higher, we'll collide with the library ID marker.
		return -1;
	}
	if (index == Pool.Size())
	{ // There were no free entries; make a new one.
		Pool.Reserve(MIN_GC_SIZE);
		FirstFreeEntry++;
	}
	else
	{ // Scan for the next free entry
		FindFirstFreeEntry(FirstFreeEntry + 1);
	}
	PoolEntry *entry = &Pool[index];
	entry->Str = str;
	entry->Hash = h;
	entry->Next = PoolBuckets[bucketnum];
	entry->Mark = false;
	entry->Locks.Clear();
	PoolBuckets[bucketnum] = index;
	return index | STRPOOL_LIBRARYID_OR;
}

//============================================================================
//
// ACSStringPool :: FindFirstFreeEntry
//
// Finds the first free entry, starting at base.
//
//============================================================================

void ACSStringPool::FindFirstFreeEntry(unsigned base)
{
	while (base < Pool.Size() && Pool[base].Next != FREE_ENTRY)
	{
		base++;
	}
	FirstFreeEntry = base;
}

//============================================================================
//
// ACSStringPool :: Dump
//
// Lists all strings in the pool.
//
//============================================================================

void ACSStringPool::Dump(void* ctx, void(*callback)(void*, uint32_t lock_size, const char* str)) const
{
	for (unsigned int i = 0; i < Pool.Size(); ++i)
	{
		if (Pool[i].Next != FREE_ENTRY)
		{
			callback(ctx, Pool[i].Locks.Size(), Pool[i].Str.GetChars());
		}
	}

#warning "TODO: API function for returning `FirstFreeEntry`"
}

void ACSStringPool::UnlockForLevel(int lnum)
{
	for (unsigned int i = 0; i < Pool.Size(); ++i)
	{
		if (Pool[i].Next != FREE_ENTRY)
		{
			auto ndx = Pool[i].Locks.Find(lnum);
			if (ndx < Pool[i].Locks.Size())
			{
				Pool[i].Locks.Delete(ndx);
			}
		}
	}
}

//============================================================================
//
// P_MarkWorldVarStrings
//
//============================================================================

void P_MarkWorldVarStrings()
{
	GlobalACSStrings.MarkStringArray(ACS_WorldVars.Pointer(), ACS_WorldVars.Size());
	for (size_t i = 0; i < ACS_WorldArrays.Size(); ++i)
	{
		GlobalACSStrings.MarkStringMap(ACS_WorldArrays.Pointer()[i]);
	}
}

//============================================================================
//
// P_MarkGlobalVarStrings
//
//============================================================================

void P_MarkGlobalVarStrings()
{
	GlobalACSStrings.MarkStringArray(ACS_GlobalVars.Pointer(), ACS_GlobalVars.Size());
	for (size_t i = 0; i < ACS_GlobalArrays.Size(); ++i)
	{
		GlobalACSStrings.MarkStringMap(ACS_GlobalArrays.Pointer()[i]);
	}
}

//============================================================================
//
// P_CollectACSGlobalStrings
//
// Garbage collect ACS global strings.
//
//============================================================================

void P_CollectACSGlobalStrings()
{
	for (FACSStack *stack = FACSStack::head; stack != NULL; stack = stack->next)
	{
		const int32_t sp = stack->sp;

		if (0 == sp)
		{
			continue;
		}
		else if (sp < 0 && sp >= STACK_SIZE)
		{
#warning "TODO: handle this case ('Corrupted stack pointer in ACS VM')"
		}
		else
		{
			GlobalACSStrings.MarkStringArray(&stack->buffer[0], sp);
		}
	}

	P_MarkWorldVarStrings();
	P_MarkGlobalVarStrings();
	GlobalACSStrings.PurgeStrings();
}

#ifdef _DEBUG
CCMD(acsgc)
{
	P_CollectACSGlobalStrings();
}
CCMD(globstr)
{
	GlobalACSStrings.Dump();
}
#endif

//============================================================================
//
// ScriptPresentation
//
// Returns a presentable version of the script number.
//
//============================================================================

static FString ScriptPresentation(int script)
{
	FString out = "script ";

	if (script < 0)
	{
		FName scrname = FName(ENamedName(-script));

		if (scrname.IsValidName())
		{
			out << '"' << scrname.GetChars() << '"';
			return out;
		}
	}
	out.AppendFormat("%d", script);
	return out;
}

//============================================================================
//
// P_ClearACSVars
//
//============================================================================

void P_ClearACSVars(bool alsoglobal)
{
	int i;

	ACS_WorldVars.Fill(0);
	for (i = 0; i < NUM_WORLDVARS; ++i)
	{
		ACS_WorldArrays[i].Clear ();
	}
	if (alsoglobal)
	{
		ACS_GlobalVars.Fill(0);
		for (i = 0; i < NUM_GLOBALVARS; ++i)
		{
			ACS_GlobalArrays[i].Clear ();
		}
		// Since we cleared all ACS variables, we know nothing refers to them
		// anymore.
		GlobalACSStrings.Clear();
	}
	else
	{
		// Purge any strings that aren't referenced by global variables, since
		// they're the only possible references left.
		P_MarkGlobalVarStrings();
		GlobalACSStrings.PurgeStrings();
	}
}

//---- ACS lump manager ----//

FBehavior *FBehaviorContainer::LoadModule(const zacs_SliceU8 slice, zacs_ModuleLoader mloader)
{
	FBehavior * behavior = new FBehavior();

	if (behavior->Init(*this, slice, mloader))
	{
		return behavior;
	}
	else
	{
		delete behavior;
		#warning "TODO: handle this case properly"
		return NULL;
	}
}

bool FBehaviorContainer::CheckAllGood ()
{
	for (unsigned int i = 0; i < StaticModules.Size(); ++i)
	{
		if (!StaticModules[i]->IsGood())
		{
			return false;
		}
	}
	return true;
}

void FBehaviorContainer::UnloadModules ()
{
	for (unsigned int i = StaticModules.Size(); i-- > 0; )
	{
		delete StaticModules[i];
	}
	StaticModules.Clear ();
}

FBehavior *FBehaviorContainer::GetModule (int lib)
{
	if ((size_t)lib >= StaticModules.Size())
	{
		return NULL;
	}
	return StaticModules[lib];
}

void FBehaviorContainer::MarkLevelVarStrings()
{
	// Mark map variables.
	for (uint32_t modnum = 0; modnum < StaticModules.Size(); ++modnum)
	{
		StaticModules[modnum]->MarkMapVarStrings();
	}

#warning "TODO: lock running scripts' local variables"
}

void FBehaviorContainer::LockLevelVarStrings(int levelnum)
{
	// Lock map variables.
	for (uint32_t modnum = 0; modnum < StaticModules.Size(); ++modnum)
	{
		StaticModules[modnum]->LockMapVarStrings(levelnum);
	}

#warning "TODO: lock running scripts' local variables"
}

void FBehaviorContainer::UnlockLevelVarStrings(int levelnum)
{
	GlobalACSStrings.UnlockForLevel(levelnum);
}

void FBehavior::MarkMapVarStrings() const
{
	GlobalACSStrings.MarkStringArray(MapVarStore, NUM_MAPVARS);

	for (int i = 0; i < NumArrays; ++i)
	{
		GlobalACSStrings.MarkStringArray(ArrayStore[i].Elements, ArrayStore[i].ArraySize);
	}
}

void FBehavior::LockMapVarStrings(int levelnum) const
{
	GlobalACSStrings.LockStringArray(levelnum, MapVarStore, NUM_MAPVARS);

	for (int i = 0; i < NumArrays; ++i)
	{
		GlobalACSStrings.LockStringArray(levelnum, ArrayStore[i].Elements, ArrayStore[i].ArraySize);
	}
}

static int ParseLocalArrayChunk(void *chunk, ACSLocalArrays *arrays, int offset)
{
	unsigned count = little_short(static_cast<unsigned short>(((unsigned *)chunk)[1] - 2)) / 4;
	int *sizes = (int *)((uint8_t *)chunk + 10);
	arrays->Count = count;
	if (count > 0)
	{
		ACSLocalArrayInfo *info = new ACSLocalArrayInfo[count];
		arrays->Info = info;
		for (unsigned i = 0; i < count; ++i)
		{
			info[i].Size = little_long(sizes[i]);
			info[i].Offset = offset;
			offset += info[i].Size;
		}
	}
	// Return the new local variable size, with space for the arrays
	return offset;
}

FBehavior::FBehavior()
{
	NumScripts = 0;
	NumFunctions = 0;
	NumArrays = 0;
	NumTotalArrays = 0;
	Scripts = NULL;
	Functions = NULL;
	Arrays = NULL;
	ArrayStore = NULL;
	Chunks = NULL;
	Data = NULL;
	Format = ACS_Unknown;
	memset (MapVarStore, 0, sizeof(MapVarStore));
	ModuleName[0] = 0;
	FunctionProfileData = NULL;
}

#warning "TODO: add a `const char* name` parameter here"

bool FBehavior::Init(FBehaviorContainer& ctr, const zacs_SliceU8 slice, zacs_ModuleLoader mloader)
{
	if (slice.ptr == nullptr) {
		return false;
	}

	// Any behaviors smaller than 32 bytes cannot possibly contain anything useful.
	// (16 bytes for a completely empty behavior + 12 bytes for one script header
	//  + 4 bytes for PCD_TERMINATE for an old-style object. A new-style object
	// has 24 bytes if it is completely empty. An empty SPTR chunk adds 8 bytes.)
	if (slice.len < 32) {
		return false;
	}

	if (slice.ptr[0] != 'A' || slice.ptr[1] != 'C' || slice.ptr[2] != 'S') {
		return false;
	}

	switch (slice.ptr[3])
	{
	case 0:
		Format = ACS_Old;
		break;
	case 'E':
		Format = ACS_Enhanced;
		break;
	case 'e':
		Format = ACS_LittleEnhanced;
		break;
	default:
		return false;
	}

	uint8_t *object;
	int i;

	object = new uint8_t[slice.len];
	memcpy(object, slice.ptr, slice.len);

    LibraryID = ctr.StaticModules.Push(this) << LIBRARYID_SHIFT;

	strcpy(ModuleName, "BEHAVIOR");
	Data = object;
	DataSize = slice.len;

	if (Format == ACS_Old)
	{
		uint32_t dirofs = little_long(((uint32_t *)object)[1]);
		uint32_t pretag = ((uint32_t *)(object + dirofs))[-1];

		Chunks = object + slice.len;
		// Check for redesigned ACSE/ACSe
		if (dirofs >= 6*4 &&
			(pretag == MAKE_ID('A','C','S','e') ||
			 pretag == MAKE_ID('A','C','S','E')))
		{
			Format = (pretag == MAKE_ID('A','C','S','e')) ? ACS_LittleEnhanced : ACS_Enhanced;
			Chunks = object + little_long(((uint32_t *)(object + dirofs))[-2]);
			// Forget about the compatibility cruft at the end of the lump
			DataSize = little_long(((uint32_t *)object)[1]) - 8;
		}

		ShouldLocalize = false;
	}
	else
	{
		Chunks = object + little_long(((uint32_t *)object)[1]);
	}

	LoadScriptsDirectory ();

	if (Format == ACS_Old)
	{
		StringTable = little_long(((uint32_t *)Data)[1]);
		StringTable += little_long(((uint32_t *)(Data + StringTable))[0]) * 12 + 4;
		UnescapeStringTable(Data + StringTable, Data, false);

#warning "TODO: allow passing in a configuration struct for this"

#if 0
		// If this is an original Hexen BEHAVIOR, set up some localization info for it. Original Hexen BEHAVIORs are always in the old format.
		if ((Level->flags2 & LEVEL2_HEXENHACK) && gameinfo.gametype == GAME_Hexen && lumpnum == -1 && reallumpnum > 0)
		{
			int fileno = fileSystem.GetFileContainer(reallumpnum);
			const char * filename = fileSystem.GetResourceFileName(fileno);

			if (!stricmp(filename, "HEXEN.WAD") || !stricmp(filename, "HEXDD.WAD"))
			{
				ShouldLocalize = true;
			}
		}
#endif
	}
	else
	{
		UnencryptStrings ();
		uint8_t *strings = FindChunk (MAKE_ID('S','T','R','L'));
		if (strings != NULL)
		{
			StringTable = uint32_t(strings - Data + 8);
			UnescapeStringTable(strings + 8, NULL, true);
		}
		else
		{
			StringTable = 0;
		}
	}

	if (Format == ACS_Old)
	{
		// Do initialization for old-style behavior lumps
		for (i = 0; i < NUM_MAPVARS; ++i)
		{
			MapVars[i] = &MapVarStore[i];
		}
	}
	else
	{
		uint32_t *chunk;

		// Load functions
		uint8_t *funcs;
		Functions = NULL;
		funcs = FindChunk (MAKE_ID('F','U','N','C'));
		if (funcs != NULL)
		{
			NumFunctions = little_long(((uint32_t *)funcs)[1]) / 8;
			funcs += 8;
			FunctionProfileData = new ACSProfileInfo[NumFunctions];
			Functions = new ScriptFunction[NumFunctions];
			for (i = 0; i < NumFunctions; ++i)
			{
				ScriptFunctionInFile *funcf = &((ScriptFunctionInFile *)funcs)[i];
				ScriptFunction *funcm = &Functions[i];
				funcm->ArgCount = funcf->ArgCount;
				funcm->HasReturnValue = funcf->HasReturnValue;
				funcm->ImportNum = funcf->ImportNum;
				funcm->LocalCount = funcf->LocalCount;
				funcm->Address = little_long(funcf->Address);
			}
		}

		// Load local arrays for functions
		if (NumFunctions > 0)
		{
			for (chunk = (uint32_t *)FindChunk(MAKE_ID('F','A','R','Y')); chunk != NULL; chunk = (uint32_t *)NextChunk((uint8_t *)chunk))
			{
				int size = little_long(chunk[1]);
				if (size >= 6)
				{
					unsigned int func_num = little_short(((uint16_t *)chunk)[4]);
					if (func_num < (unsigned int)NumFunctions)
					{
						ScriptFunction *func = &Functions[func_num];
						// Unlike scripts, functions do not include their arg count in their local count.
						func->LocalCount = ParseLocalArrayChunk(chunk, &func->LocalArrays, func->LocalCount + func->ArgCount) - func->ArgCount;
					}
				}
			}
		}

		// Load JUMP points
		chunk = (uint32_t *)FindChunk (MAKE_ID('J','U','M','P'));
		if (chunk != NULL)
		{
			for (i = 0;i < (int)little_long(chunk[1]);i += 4)
				JumpPoints.Push(little_long(chunk[2 + i/4]));
		}

		// Initialize this object's map variables
		memset (MapVarStore, 0, sizeof(MapVarStore));
		chunk = (uint32_t *)FindChunk (MAKE_ID('M','I','N','I'));
		while (chunk != NULL)
		{
			int numvars = little_long(chunk[1])/4 - 1;
			int firstvar = little_long(chunk[2]);
			for (i = 0; i < numvars; ++i)
			{
				MapVarStore[i+firstvar] = little_long(chunk[3+i]);
			}
			chunk = (uint32_t *)NextChunk ((uint8_t *)chunk);
		}

		// Initialize this object's map variable pointers to defaults. They can be changed
		// later once the imported modules are loaded.
		for (i = 0; i < NUM_MAPVARS; ++i)
		{
			MapVars[i] = &MapVarStore[i];
		}

		// Create arrays for this module
		chunk = (uint32_t *)FindChunk (MAKE_ID('A','R','A','Y'));
		if (chunk != NULL)
		{
			NumArrays = little_long(chunk[1])/8;
			ArrayStore = new ArrayInfo[NumArrays];
			memset (ArrayStore, 0, sizeof(*Arrays)*NumArrays);
			for (i = 0; i < NumArrays; ++i)
			{
				MapVarStore[little_long(chunk[2+i*2])] = i;
				ArrayStore[i].ArraySize = little_long(chunk[3+i*2]);
				ArrayStore[i].Elements = new int32_t[ArrayStore[i].ArraySize];
				memset(ArrayStore[i].Elements, 0, ArrayStore[i].ArraySize*sizeof(uint32_t));
			}
		}

		// Initialize arrays for this module
		chunk = (uint32_t *)FindChunk (MAKE_ID('A','I','N','I'));
		while (chunk != NULL)
		{
			int arraynum = MapVarStore[little_long(chunk[2])];
			if ((unsigned)arraynum < (unsigned)NumArrays)
			{
				// Use unsigned iterator here to avoid issue with GCC 4.9/5.x
				// optimizer. Might be some undefined behavior in this code,
				// but I don't know what it is.
				unsigned int initsize = std::min<unsigned int> (ArrayStore[arraynum].ArraySize, (little_long(chunk[1])-4)/4);
				int32_t *elems = ArrayStore[arraynum].Elements;
				for (unsigned int j = 0; j < initsize; ++j)
				{
					elems[j] = little_long(chunk[3+j]);
				}
			}
			chunk = (uint32_t *)NextChunk((uint8_t *)chunk);
		}

		// Start setting up array pointers
		NumTotalArrays = NumArrays;
		chunk = (uint32_t *)FindChunk (MAKE_ID('A','I','M','P'));
		if (chunk != NULL)
		{
			NumTotalArrays += little_long(chunk[2]);
		}
		if (NumTotalArrays != 0)
		{
			Arrays = new ArrayInfo *[NumTotalArrays];
			for (i = 0; i < NumArrays; ++i)
			{
				Arrays[i] = &ArrayStore[i];
			}
		}

		// Tag the library ID to any map variables that are initialized with strings
		if (LibraryID != 0)
		{
			chunk = (uint32_t *)FindChunk (MAKE_ID('M','S','T','R'));
			if (chunk != NULL)
			{
				for (uint32_t i = 0; i < little_long(chunk[1])/4; ++i)
				{
					const char *str = LookupString(MapVarStore[little_long(chunk[i+2])]);
					if (str != NULL)
					{
						MapVarStore[little_long(chunk[i+2])] = GlobalACSStrings.AddString(str);
					}
				}
			}

			chunk = (uint32_t *)FindChunk (MAKE_ID('A','S','T','R'));
			if (chunk != NULL)
			{
				for (uint32_t i = 0; i < little_long(chunk[1])/4; ++i)
				{
					int arraynum = MapVarStore[little_long(chunk[i+2])];
					if ((unsigned)arraynum < (unsigned)NumArrays)
					{
						int32_t *elems = ArrayStore[arraynum].Elements;
						for (int j = ArrayStore[arraynum].ArraySize; j > 0; --j, ++elems)
						{
//							*elems |= LibraryID;
							const char *str = LookupString(*elems);
							if (str != NULL)
							{
								*elems = GlobalACSStrings.AddString(str);
							}
						}
					}
				}
			}

			// [BL] Newer version of ASTR for structure aware compilers although we only have one array per chunk
			chunk = (uint32_t *)FindChunk (MAKE_ID('A','T','A','G'));
			while (chunk != NULL)
			{
				const uint8_t* chunkData = (const uint8_t*)(chunk + 2);
				// First byte is version, it should be 0
				if(*chunkData++ == 0)
				{
					int arraynum = MapVarStore[uallong(little_long(*(const int*)(chunkData)))];
					chunkData += 4;
					if ((unsigned)arraynum < (unsigned)NumArrays)
					{
						int32_t *elems = ArrayStore[arraynum].Elements;
						// Ending zeros may be left out.
						for (int j = std::min(little_long(chunk[1])-5, ArrayStore[arraynum].ArraySize); j > 0; --j, ++elems, ++chunkData)
						{
							// For ATAG, a value of 0 = Integer, 1 = String, 2 = FunctionPtr
							// Our implementation uses the same tags for both String and FunctionPtr
							if (*chunkData == 2)
							{
								*elems |= LibraryID;
							}
							else if (*chunkData == 1)
							{
								const char *str = LookupString(*elems);
								if (str != NULL)
								{
									*elems = GlobalACSStrings.AddString(str);
								}
							}
						}
					}
				}

				chunk = (uint32_t *)NextChunk ((uint8_t *)chunk);
			}
		}

		// Load required libraries.
		if (NULL != (chunk = (uint32_t *)FindChunk (MAKE_ID('L','O','A','D'))))
		{
			const char *const parse = (char *)&chunk[2];
			uint32_t i;

			for (i = 0; i < little_long(chunk[1]); )
			{
				if (parse[i])
				{
					FBehavior *beh = mloader.callback(mloader.ctx, &parse[i]);

					if (beh != nullptr) {
						Imports.Push (beh);
					} else {
#warning "TODO: handle this case"

#if 0
						Printf (TEXTCOLOR_RED "Could not find ACS library %s.\n", &parse[i]);
#endif
					}

					do {;} while (parse[++i]);
				}

				++i;
			}

			// Go through each imported module in order and resolve all imported functions
			// and map variables.
			for (i = 0; i < Imports.Size(); ++i)
			{
				FBehavior *lib = Imports[i];
				int j;

				if (lib == NULL)
					continue;

				// Resolve functions
				chunk = (uint32_t *)FindChunk(MAKE_ID('F','N','A','M'));
				for (j = 0; j < NumFunctions; ++j)
				{
					ScriptFunction *func = &((ScriptFunction *)Functions)[j];
					if (func->Address == 0 && func->ImportNum == 0)
					{
						int libfunc = lib->FindFunctionName ((char *)(chunk + 2) + little_long(chunk[3+j]));
						if (libfunc >= 0)
						{
							ScriptFunction *realfunc = &((ScriptFunction *)lib->Functions)[libfunc];
							// Make sure that the library really defines this function. It might simply
							// be importing it itself.
							if (realfunc->Address != 0 && realfunc->ImportNum == 0)
							{
								func->Address = libfunc;
								func->ImportNum = i+1;
								if (realfunc->ArgCount != func->ArgCount)
								{
#warning "TODO: handle this case"

#if 0
									Printf (TEXTCOLOR_ORANGE "Function %s in %s has %d arguments. %s expects it to have %d.\n",
										(char *)(chunk + 2) + little_long(chunk[3+j]), lib->ModuleName, realfunc->ArgCount,
										ModuleName, func->ArgCount);
#endif
									Format = ACS_Unknown;
								}
								// The next two properties do not affect code compatibility, so it is
								// okay for them to be different in the imported module than they are
								// in this one, as long as we make sure to use the real values.
								func->LocalCount = little_long(realfunc->LocalCount);
								func->HasReturnValue = realfunc->HasReturnValue;
							}
						}
					}
				}

				// Resolve map variables
				chunk = (uint32_t *)FindChunk(MAKE_ID('M','I','M','P'));
				if (chunk != NULL)
				{
					char *parse = (char *)&chunk[2];
					for (uint32_t j = 0; j < little_long(chunk[1]); )
					{
						uint32_t varNum = little_long(*(uint32_t *)&parse[j]);
						j += 4;
						int impNum = lib->FindMapVarName (&parse[j]);
						if (impNum >= 0)
						{
							MapVars[varNum] = &lib->MapVarStore[impNum];
						}
						do {;} while (parse[++j]);
						++j;
					}
				}

				// Resolve arrays
				if (NumTotalArrays > NumArrays)
				{
					chunk = (uint32_t *)FindChunk(MAKE_ID('A','I','M','P'));
					char *parse = (char *)&chunk[3];
					for (uint32_t j = 0; j < little_long(chunk[2]); ++j)
					{
						uint32_t varNum = little_long(*(uint32_t *)parse);
						parse += 4;
						uint32_t expectedSize = little_long(*(uint32_t *)parse);
						parse += 4;
						int impNum = lib->FindMapArray (parse);
						if (impNum >= 0)
						{
							Arrays[NumArrays + j] = &lib->ArrayStore[impNum];
							MapVarStore[varNum] = NumArrays + j;
							if (lib->ArrayStore[impNum].ArraySize != expectedSize)
							{
								Format = ACS_Unknown;

								#warning "TODO: handle this case"

#if 0
								Printf (TEXTCOLOR_ORANGE "The array %s in %s has %u elements, but %s expects it to only have %u.\n",
									parse, lib->ModuleName, lib->ArrayStore[impNum].ArraySize,
									ModuleName, expectedSize);
#endif
							}
						}
						do {;} while (*++parse);
						++parse;
					}
				}
			}
		}
	}

	return true;
}

FBehavior::~FBehavior ()
{
	if (Scripts != NULL)
	{
		delete[] Scripts;
		Scripts = NULL;
	}
	if (Arrays != NULL)
	{
		delete[] Arrays;
		Arrays = NULL;
	}
	if (ArrayStore != NULL)
	{
		for (int i = 0; i < NumArrays; ++i)
		{
			if (ArrayStore[i].Elements != NULL)
			{
				delete[] ArrayStore[i].Elements;
				ArrayStore[i].Elements = NULL;
			}
		}
		delete[] ArrayStore;
		ArrayStore = NULL;
	}
	if (Functions != NULL)
	{
		delete[] Functions;
		Functions = NULL;
	}
	if (FunctionProfileData != NULL)
	{
		delete[] FunctionProfileData;
		FunctionProfileData = NULL;
	}
	if (Data != NULL)
	{
		delete[] Data;
		Data = NULL;
	}
}

void FBehavior::LoadScriptsDirectory ()
{
	union
	{
		uint8_t *b;
		uint32_t *dw;
		uint16_t *w;
		int16_t *sw;
		ScriptPtr2 *po;		// Old
		ScriptPtr1 *pi;		// Intermediate
		ScriptPtr3 *pe;		// LittleEnhanced
	} scripts;
	int i, max;

	NumScripts = 0;
	Scripts = NULL;

	// Load the main script directory
	switch (Format)
	{
	case ACS_Old:
		scripts.dw = (uint32_t *)(Data + little_long(((uint32_t *)Data)[1]));
		NumScripts = little_long(scripts.dw[0]);
		if (NumScripts != 0)
		{
			scripts.dw++;

			Scripts = new ScriptPtr[NumScripts];

			for (i = 0; i < NumScripts; ++i)
			{
				ScriptPtr2 *ptr1 = &scripts.po[i];
				ScriptPtr  *ptr2 = &Scripts[i];

				ptr2->Number = little_long(ptr1->Number) % 1000;
				ptr2->Type = little_long(ptr1->Number) / 1000;
				ptr2->ArgCount = little_long(ptr1->ArgCount);
				ptr2->Address = little_long(ptr1->Address);
			}
		}
		break;

	case ACS_Enhanced:
	case ACS_LittleEnhanced:
		scripts.b = FindChunk (MAKE_ID('S','P','T','R'));
		if (scripts.b == NULL)
		{
			// There are no scripts!
		}
		else if (*(uint32_t *)Data != MAKE_ID('A','C','S',0))
		{
			NumScripts = little_long(scripts.dw[1]) / 12;
			Scripts = new ScriptPtr[NumScripts];
			scripts.dw += 2;

			for (i = 0; i < NumScripts; ++i)
			{
				ScriptPtr1 *ptr1 = &scripts.pi[i];
				ScriptPtr  *ptr2 = &Scripts[i];

				ptr2->Number = little_short(ptr1->Number);
				ptr2->Type = uint8_t(little_short(ptr1->Type));
				ptr2->ArgCount = little_long(ptr1->ArgCount);
				ptr2->Address = little_long(ptr1->Address);
			}
		}
		else
		{
			NumScripts = little_long(scripts.dw[1]) / 8;
			Scripts = new ScriptPtr[NumScripts];
			scripts.dw += 2;

			for (i = 0; i < NumScripts; ++i)
			{
				ScriptPtr3 *ptr1 = &scripts.pe[i];
				ScriptPtr  *ptr2 = &Scripts[i];

				ptr2->Number = little_short(ptr1->Number);
				ptr2->Type = ptr1->Type;
				ptr2->ArgCount = ptr1->ArgCount;
				ptr2->Address = little_long(ptr1->Address);
			}
		}
		break;

	default:
		break;
	}

// [EP] Clang 3.5.0 optimizer miscompiles this function and causes random
// crashes in the program. This is fixed in 3.5.1 onwards.
#if defined(__clang__) && __clang_major__ == 3 && __clang_minor__ == 5 && __clang_patchlevel__ == 0
	asm("" : "+g" (NumScripts));
#endif
	for (i = 0; i < NumScripts; ++i)
	{
		Scripts[i].Flags = 0;
		Scripts[i].VarCount = LOCAL_SIZE;
	}

	// Sort scripts, so we can use a binary search to find them
	if (NumScripts > 1)
	{
		qsort (Scripts, NumScripts, sizeof(ScriptPtr), SortScripts);
		// Check for duplicates because ACC originally did not enforce
		// script number uniqueness across different script types. We
		// only need to do this for old format lumps, because the ACCs
		// that produce new format lumps won't let you do this.
		if (Format == ACS_Old)
		{
			for (i = 0; i < NumScripts - 1; ++i)
			{
				if (Scripts[i].Number == Scripts[i+1].Number)
				{
#warning "TODO: handle this case"

#if 0
					Printf(TEXTCOLOR_ORANGE "%s appears more than once.\n",
						ScriptPresentation(Scripts[i].Number).GetChars());
#endif

					// Make the closed version the first one.
					if (Scripts[i+1].Type == SCRIPT_Closed)
					{
						std::swap(Scripts[i], Scripts[i+1]);
					}
				}
			}
		}
	}

	if (Format == ACS_Old)
		return;

	// Load script flags
	scripts.b = FindChunk (MAKE_ID('S','F','L','G'));
	if (scripts.dw != NULL)
	{
		max = little_long(scripts.dw[1]) / 4;
		scripts.dw += 2;
		for (i = max; i > 0; --i, scripts.w += 2)
		{
			ScriptPtr *ptr = const_cast<ScriptPtr *>(FindScript (little_short(scripts.sw[0])));
			if (ptr != NULL)
			{
				ptr->Flags = little_short(scripts.w[1]);
			}
		}
	}

	// Load script var counts. (Only recorded for scripts that use more than LOCAL_SIZE variables.)
	scripts.b = FindChunk (MAKE_ID('S','V','C','T'));
	if (scripts.dw != NULL)
	{
		max = little_long(scripts.dw[1]) / 4;
		scripts.dw += 2;
		for (i = max; i > 0; --i, scripts.w += 2)
		{
			ScriptPtr *ptr = const_cast<ScriptPtr *>(FindScript (little_short(scripts.sw[0])));
			if (ptr != NULL)
			{
				ptr->VarCount = little_short(scripts.w[1]);
			}
		}
	}

	// Load script array sizes. (One chunk per script that uses arrays.)
	for (scripts.b = FindChunk(MAKE_ID('S','A','R','Y')); scripts.dw != NULL; scripts.b = NextChunk(scripts.b))
	{
		int size = little_long(scripts.dw[1]);
		if (size >= 6)
		{
			int script_num = little_short(scripts.sw[4]);
			ScriptPtr *ptr = const_cast<ScriptPtr *>(FindScript(script_num));
			if (ptr != NULL)
			{
				ptr->VarCount = ParseLocalArrayChunk(scripts.b, &ptr->LocalArrays, ptr->VarCount);
			}
		}
	}

	// Load script names (if any)
	scripts.b = FindChunk(MAKE_ID('S','N','A','M'));
	if (scripts.dw != NULL)
	{
		UnescapeStringTable(scripts.b + 8, NULL, false);
		for (i = 0; i < NumScripts; ++i)
		{
			// ACC stores script names as an index into the SNAM chunk, with the first index as
			// -1 and counting down from there. We convert this from an index into SNAM into
			// a negative index into the global name table.
			if (Scripts[i].Number < 0)
			{
				const char *str = (const char *)(scripts.b + 8 + scripts.dw[3 + (-Scripts[i].Number - 1)]);
				FName name(str);
				Scripts[i].Number = -name.GetIndex();
			}
		}
		// We need to resort scripts, because the new numbers for named scripts likely
		// do not match the order they were originally in.
		qsort (Scripts, NumScripts, sizeof(ScriptPtr), SortScripts);
	}
}

int FBehavior::SortScripts (const void *a, const void *b)
{
	ScriptPtr *ptr1 = (ScriptPtr *)a;
	ScriptPtr *ptr2 = (ScriptPtr *)b;
	return ptr1->Number - ptr2->Number;
}

//============================================================================
//
// FBehavior :: UnencryptStrings
//
// Descrambles strings in a STRE chunk to transform it into a STRL chunk.
//
//============================================================================

void FBehavior::UnencryptStrings ()
{
	uint32_t *prevchunk = NULL;
	uint32_t *chunk = (uint32_t *)FindChunk(MAKE_ID('S','T','R','E'));
	while (chunk != NULL)
	{
		for (uint32_t strnum = 0; strnum < little_long(chunk[3]); ++strnum)
		{
			int ofs = little_long(chunk[5+strnum]);
			uint8_t *data = (uint8_t *)chunk + ofs + 8, last;
			int p = (uint8_t)(ofs*157135);
			int i = 0;
			do
			{
				last = (data[i] ^= (uint8_t)(p+(i>>1)));
				++i;
			} while (last != 0);
		}
		prevchunk = chunk;
		chunk = (uint32_t *)NextChunk ((uint8_t *)chunk);
		*prevchunk = MAKE_ID('S','T','R','L');
	}
	if (prevchunk != NULL)
	{
		*prevchunk = MAKE_ID('S','T','R','L');
	}
}

//============================================================================
//
// FBehavior :: UnescapeStringTable
//
// Processes escape sequences for every string in a string table.
// Chunkstart points to the string table. Datastart points to the base address
// for offsets in the string table; if NULL, it will use chunkstart. If
// has_padding is true, then this is a STRL chunk with four bytes of padding
// on either side of the string count.
//
//============================================================================

void FBehavior::UnescapeStringTable(uint8_t *chunkstart, uint8_t *datastart, bool has_padding)
{
	assert(chunkstart != NULL);

	uint32_t *chunk = (uint32_t *)chunkstart;

	if (datastart == NULL)
	{
		datastart = chunkstart;
	}
	if (!has_padding)
	{
		chunk[0] = little_long(chunk[0]);
		for (uint32_t strnum = 0; strnum < chunk[0]; ++strnum)
		{
			int ofs = little_long(chunk[1 + strnum]);	// Byte swap offset, if needed.
			chunk[1 + strnum] = ofs;
			strbin((char *)datastart + ofs);
		}
	}
	else
	{
		chunk[1] = little_long(chunk[1]);
		for (uint32_t strnum = 0; strnum < chunk[1]; ++strnum)
		{
			int ofs = little_long(chunk[3 + strnum]);	// Byte swap offset, if needed.
			chunk[3 + strnum] = ofs;
			strbin((char *)datastart + ofs);
		}
	}
}

//============================================================================
//
// FBehavior :: IsGood
//
//============================================================================

bool FBehavior::IsGood ()
{
	bool bad;
	int i;

	// Check that the data format was understood
	if (Format == ACS_Unknown)
	{
		return false;
	}

	// Check that all functions are resolved
	bad = false;
	for (i = 0; i < NumFunctions; ++i)
	{
		ScriptFunction *funcdef = (ScriptFunction *)Functions + i;
		if (funcdef->Address == 0 && funcdef->ImportNum == 0)
		{
			uint32_t *chunk = (uint32_t *)FindChunk (MAKE_ID('F','N','A','M'));
#warning "TODO: handle this case"

#if 0
			Printf (TEXTCOLOR_RED "Could not find ACS function %s for use in %s.\n",
				(char *)(chunk + 2) + chunk[3+i], ModuleName);
#endif
			bad = true;
		}
	}

	// Check that all imported modules were loaded
	for (i = Imports.Size() - 1; i >= 0; --i)
	{
		if (Imports[i] == NULL)
		{
#warning "TODO: handle this case"

#if 0
			Printf (TEXTCOLOR_RED "Not all the libraries used by %s could be found.\n", ModuleName);
#endif
			return false;
		}
	}

	return !bad;
}

const ScriptPtr *FBehavior::FindScript (int script) const
{
	const ScriptPtr *ptr = BinarySearch<ScriptPtr, int>
		((ScriptPtr *)Scripts, NumScripts, &ScriptPtr::Number, script);

	// If the preceding script has the same number, return it instead.
	// See the note by the script sorting above for why.
	if (ptr > Scripts)
	{
		if (ptr[-1].Number == script)
		{
			ptr--;
		}
	}
	return ptr;
}

const ScriptPtr *FBehaviorContainer::FindScript (int script, FBehavior *&module)
{
	for (uint32_t i = 0; i < StaticModules.Size(); ++i)
	{
		const ScriptPtr *code = StaticModules[i]->FindScript (script);
		if (code != NULL)
		{
			module = StaticModules[i];
			return code;
		}
	}
	return NULL;
}

ScriptFunction *FBehavior::GetFunction (int funcnum, FBehavior *&module) const
{
	if ((unsigned)funcnum >= (unsigned)NumFunctions)
	{
		return NULL;
	}
	ScriptFunction *funcdef = (ScriptFunction *)Functions + funcnum;
	if (funcdef->ImportNum)
	{
		return Imports[funcdef->ImportNum - 1]->GetFunction (funcdef->Address, module);
	}
	// Should I just un-const this function instead of using a const_cast?
	module = const_cast<FBehavior *>(this);
	return funcdef;
}

int FBehavior::FindFunctionName (const char *funcname) const
{
	return FindStringInChunk ((uint32_t *)FindChunk (MAKE_ID('F','N','A','M')), funcname);
}

int FBehavior::FindMapVarName (const char *varname) const
{
	return FindStringInChunk ((uint32_t *)FindChunk (MAKE_ID('M','E','X','P')), varname);
}

int FBehavior::FindMapArray (const char *arrayname) const
{
	int var = FindMapVarName (arrayname);
	if (var >= 0)
	{
		return MapVarStore[var];
	}
	return -1;
}

int FBehavior::FindStringInChunk (uint32_t *names, const char *varname) const
{
	if (names != NULL)
	{
		uint32_t i;

		for (i = 0; i < little_long(names[2]); ++i)
		{
			if (stricmp (varname, (char *)(names + 2) + little_long(names[3+i])) == 0)
			{
				return (int)i;
			}
		}
	}
	return -1;
}

int FBehavior::GetArrayVal (int arraynum, int index) const
{
	if ((unsigned)arraynum >= (unsigned)NumTotalArrays)
		return 0;
	const ArrayInfo *array = Arrays[arraynum];
	if ((unsigned)index >= (unsigned)array->ArraySize)
		return 0;
	return array->Elements[index];
}

void FBehavior::SetArrayVal (int arraynum, int index, int value)
{
	if ((unsigned)arraynum >= (unsigned)NumTotalArrays)
		return;
	const ArrayInfo *array = Arrays[arraynum];
	if ((unsigned)index >= (unsigned)array->ArraySize)
		return;
	array->Elements[index] = value;
}

inline bool FBehavior::CopyStringToArray(int arraynum, int index, int maxLength, const char *string)
{
	 // false if the operation was incomplete or unsuccessful

	if ((unsigned)arraynum >= (unsigned)NumTotalArrays || index < 0)
		return false;
	const ArrayInfo *array = Arrays[arraynum];

	if ((signed)array->ArraySize - index < maxLength) maxLength = (signed)array->ArraySize - index;

	while (maxLength-- > 0)
	{
		array->Elements[index++] = *string;
		if (!(*string)) return true; // written terminating 0
		string++;
	}
	return !(*string); // return true if only terminating 0 was not written
}

uint8_t *FBehavior::FindChunk (uint32_t id) const
{
	uint8_t *chunk = Chunks;

	while (chunk != NULL && chunk < Data + DataSize)
	{
		if (((uint32_t *)chunk)[0] == id)
		{
			return chunk;
		}
		chunk += little_long(((uint32_t *)chunk)[1]) + 8;
	}
	return NULL;
}

uint8_t *FBehavior::NextChunk (uint8_t *chunk) const
{
	uint32_t id = *(uint32_t *)chunk;
	chunk += little_long(((uint32_t *)chunk)[1]) + 8;
	while (chunk != NULL && chunk < Data + DataSize)
	{
		if (((uint32_t *)chunk)[0] == id)
		{
			return chunk;
		}
		chunk += little_long(((uint32_t *)chunk)[1]) + 8;
	}
	return NULL;
}

const char *FBehaviorContainer::LookupString (uint32_t index, bool forprint)
{
	uint32_t lib = index >> LIBRARYID_SHIFT;

	if (lib == STRPOOL_LIBRARYID)
	{
		return GlobalACSStrings.GetString(index);
	}
	if (lib >= (uint32_t)StaticModules.Size())
	{
		return NULL;
	}
	return StaticModules[lib]->LookupString (index & 0xffff, forprint);
}

const char *FBehavior::LookupString (uint32_t index, bool forprint) const
{
	if (StringTable == 0)
	{
		return NULL;
	}

	if (Format == ACS_Old)
	{
		uint32_t *list = (uint32_t *)(Data + StringTable);

		if (index >= list[0])
			return NULL;	// Out of range for this list;

		const char *s = (const char *)(Data + list[1 + index]);

		#warning "TODO: what to do about this?"

#if 0
		// Allow translations for Hexen's original strings.
		// This synthesizes a string label and looks it up.
		// It will only do this for original Hexen maps and PCD_PRINTSTRING operations.
		// For localizing user content better solutions exist so this hack won't be available as an editing feature.
		if (ShouldLocalize && forprint)
		{
			FString token = s;
			token.ToUpper();
			token.ReplaceChars(".,-+!?", ' ');
			token.Substitute(" ", "");
			token.Truncate(5);

			FStringf label("TXT_ACS_%s_%d_%.5s", Level->MapName.GetChars(), index, token.GetChars());
			auto p = GStrings[label.GetChars()];
			if (p) return p;
		}
#endif

		return s;
	}
	else
	{
		uint32_t *list = (uint32_t *)(Data + StringTable);

		if (index >= list[1])
			return NULL;	// Out of range for this list
		return (const char *)(Data + StringTable + list[3+index]);
	}
}
