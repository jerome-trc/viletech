#pragma once

#ifndef stricmp
#define stricmp strcasecmp
#endif

#ifndef strnicmp
#define strnicmp strncasecmp
#endif

#ifndef __BIG_ENDIAN__
#define MAKE_ID(a, b, c, d) ((uint32_t)((a) | ((b) << 8) | ((c) << 16) | ((d) << 24)))
#else
#define MAKE_ID(a, b, c, d) ((uint32_t)((d) | ((c) << 8) | ((b) << 16) | ((a) << 24)))
#endif

#ifdef __APPLE__
#include <CoreFoundation/CoreFoundation.h>

#define little_short(x) CFSwapInt16LittleToHost(x)
#define little_long(x) CFSwapInt32LittleToHost(x)
#else
#ifdef __BIG_ENDIAN__

// Swap 16bit, that is, MSB and LSB byte.
// No masking with 0xFF should be necessary.
inline short little_short(short x) {
	return (short)((((unsigned short)x) >> 8) | (((unsigned short)x) << 8));
}

inline unsigned short little_short(unsigned short x) {
	return (unsigned short)((x >> 8) | (x << 8));
}

// Swapping 32bit.
inline unsigned int little_long(unsigned int x) {
	return (unsigned int)((x >> 24) | ((x >> 8) & 0xff00) | ((x << 8) & 0xff0000) | (x << 24));
}

inline int little_long(int x) {
	return (int)((((unsigned int)x) >> 24) | ((((unsigned int)x) >> 8) & 0xff00) |
				 ((((unsigned int)x) << 8) & 0xff0000) | (((unsigned int)x) << 24));
}

#else

#define little_short(x) (x)
#define little_long(x) (x)

#endif // __BIG_ENDIAN__
#endif // __APPLE__

#define DELETE_COPIERS(type)    \
	type(const type&) = delete; \
	type& operator=(const type&) = delete;
#define DELETE_MOVERS(type) \
	type(type&&) = delete;  \
	type& operator=(type&&) = delete;

#define DELETE_COPIERS_AND_MOVERS(type) DELETE_COPIERS(type) DELETE_MOVERS(type)

/// [RH] Replaces the escape sequences in a string with actual escaped characters.
/// This operation is done in-place. The result is the new length of the string.
inline int strbin(char* str) {
	char* start = str;
	char *p = str, c;
	int i;

	while ((c = *p++)) {
		if (c != '\\') {
			*str++ = c;
		} else if (*p != 0) {
			switch (*p) {
			case 'a':
				*str++ = '\a';
				break;
			case 'b':
				*str++ = '\b';
				break;
			case 'c':
				*str++ = '\034'; // TEXTCOLOR_ESCAPE
				break;
			case 'f':
				*str++ = '\f';
				break;
			case 'n':
				*str++ = '\n';
				break;
			case 't':
				*str++ = '\t';
				break;
			case 'r':
				*str++ = '\r';
				break;
			case 'v':
				*str++ = '\v';
				break;
			case '?':
				*str++ = '\?';
				break;
			case '\n':
				break;
			case 'x':
			case 'X':
				c = 0;
				for (i = 0; i < 2; i++) {
					p++;
					if (*p >= '0' && *p <= '9')
						c = (c << 4) + *p - '0';
					else if (*p >= 'a' && *p <= 'f')
						c = (c << 4) + 10 + *p - 'a';
					else if (*p >= 'A' && *p <= 'F')
						c = (c << 4) + 10 + *p - 'A';
					else {
						p--;
						break;
					}
				}
				*str++ = c;
				break;
			case '0':
			case '1':
			case '2':
			case '3':
			case '4':
			case '5':
			case '6':
			case '7':
				c = *p - '0';
				for (i = 0; i < 2; i++) {
					p++;
					if (*p >= '0' && *p <= '7')
						c = (c << 3) + *p - '0';
					else {
						p--;
						break;
					}
				}
				*str++ = c;
				break;
			default:
				*str++ = *p;
				break;
			}
			p++;
		}
	}

	*str = 0;

	return int(str - start);
}

template<class ClassType, class KeyType>
inline
const ClassType *BinarySearch (const ClassType *first, int max,
	const KeyType ClassType::*keyptr, const KeyType key)
{
	int min = 0;
	--max;

	while (min <= max)
	{
		int mid = (min + max) / 2;
		const ClassType *probe = &first[mid];
		const KeyType &seekey = probe->*keyptr;
		if (seekey == key)
		{
			return probe;
		}
		else if (seekey < key)
		{
			min = mid + 1;
		}
		else
		{
			max = mid - 1;
		}
	}

	return nullptr;
}
