#ifndef __COMMON_H__
#define __COMMON_H__

#ifdef _MSC_VER
#pragma once
#endif

#include <limits.h>
#include <exception>
#include <stdexcept>
#include <math.h>

#ifdef _MSC_VER
typedef unsigned __int32 uint32_t;
typedef __int32 int32_t;
#else
#include <stdint.h>
#endif

#ifndef stricmp
#define stricmp strcasecmp
#endif

#ifndef strnicmp
#define strnicmp strncasecmp
#endif

#include "znbx.h"

#define FIXED_MAX INT_MAX
#define FIXED_MIN INT_MIN

#define FRACBITS 16

static_assert(UINT_MAX == 0xffffffff);

[[nodiscard]] inline znbx_Angle PointToAngle(znbx_I16F16 x, znbx_I16F16 y) {
	double ang = atan2(double(y), double(x));
	const double rad2bam = double(1 << 30) / M_PI;
	double dbam = ang * rad2bam;
	// Convert to signed first since negative double to unsigned is undefined.
	return znbx_Angle(int(dbam)) << 1;
}

static const uint16_t NO_MAP_INDEX = 0xffff;
static const uint32_t NO_INDEX = 0xffffffff;
static const znbx_Angle ANGLE_MAX = 0xffffffff;
static const znbx_Angle ANGLE_180 = (1u << 31);
static const znbx_Angle ANGLE_EPSILON = 5000;

#if defined(_MSC_VER) && !defined(__clang__) && defined(_M_IX86)

#pragma warning(disable : 4035)

inline znbx_I16F16 Scale(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c) {
	__asm mov eax, a __asm mov ecx, c __asm imul b __asm idiv ecx
}

inline znbx_I16F16 DivScale30(znbx_I16F16 a, znbx_I16F16 b) {
	__asm mov edx, a __asm sar edx, 2 __asm mov eax, a __asm shl eax, 30 __asm idiv b
}

inline znbx_I16F16 MulScale30(znbx_I16F16 a, znbx_I16F16 b) {
	__asm mov eax, a __asm imul b __asm shrd eax, edx, 30
}

inline znbx_I16F16 DMulScale32(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c, znbx_I16F16 d) {
	__asm mov eax, a __asm imul b __asm mov ebx, eax __asm mov eax, c __asm mov esi,
		edx __asm imul d __asm add eax, ebx __asm adc edx, esi __asm mov eax, edx
}

#pragma warning(default : 4035)

#elif defined(__GNUC__) && defined(__i386__)

#ifdef __clang__
inline znbx_I16F16 Scale(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c) {
	znbx_I16F16 result, dummy;

	asm volatile("imull %3\n\t"
				 "idivl %4"
				 : "=a"(result), "=&d"(dummy)
				 : "a"(a), "r"(b), "r"(c)
				 : "%cc");

	return result;
}

inline znbx_I16F16 DivScale30(znbx_I16F16 a, znbx_I16F16 b) {
	znbx_I16F16 result, dummy;
	asm volatile("idivl %4"
				 : "=a"(result), "=d"(dummy)
				 : "a"(a << 30), "d"(a >> 2), "r"(b)
				 : "%cc");
	return result;
}
#else
inline znbx_I16F16 Scale(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c) {
	znbx_I16F16 result, dummy;

	asm volatile("imull %3\n\t"
				 "idivl %4"
				 : "=a,a,a,a,a,a"(result), "=&d,&d,&d,&d,d,d"(dummy)
				 : "a,a,a,a,a,a"(a), "m,r,m,r,d,d"(b), "r,r,m,m,r,m"(c)
				 : "%cc");

	return result;
}

inline znbx_I16F16 DivScale30(znbx_I16F16 a, znbx_I16F16 b) {
	znbx_I16F16 result, dummy;
	asm volatile("idivl %4"
				 : "=a,a"(result), "=d,d"(dummy)
				 : "a,a"(a << 30), "d,d"(a >> 2), "r,m"(b)
				 : "%cc");
	return result;
}
#endif

inline znbx_I16F16 MulScale30(znbx_I16F16 a, znbx_I16F16 b) {
	return ((int64_t)a * b) >> 30;
}

inline znbx_I16F16 DMulScale30(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c, znbx_I16F16 d) {
	return (((int64_t)a * b) + ((int64_t)c * d)) >> 30;
}

inline znbx_I16F16 DMulScale32(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c, znbx_I16F16 d) {
	return (((int64_t)a * b) + ((int64_t)c * d)) >> 32;
}

#else

inline znbx_I16F16 Scale(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c) {
	return (znbx_I16F16)(double(a) * double(b) / double(c));
}

inline znbx_I16F16 DivScale30(znbx_I16F16 a, znbx_I16F16 b) {
	return (znbx_I16F16)(double(a) / double(b) * double(1 << 30));
}

inline znbx_I16F16 MulScale30(znbx_I16F16 a, znbx_I16F16 b) {
	return (znbx_I16F16)(double(a) * double(b) / double(1 << 30));
}

inline znbx_I16F16 DMulScale30(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c, znbx_I16F16 d) {
	return (znbx_I16F16)((double(a) * double(b) + double(c) * double(d)) / double(1 << 30));
}

inline znbx_I16F16 DMulScale32(znbx_I16F16 a, znbx_I16F16 b, znbx_I16F16 c, znbx_I16F16 d) {
	return (znbx_I16F16)((double(a) * double(b) + double(c) * double(d)) / 4294967296.0);
}

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

#define DELETE_COPIERS(type)    \
	type(const type&) = delete; \
	type& operator=(const type&) = delete;
#define DELETE_MOVERS(type) \
	type(type&&) = delete;  \
	type& operator=(type&&) = delete;

#define DELETE_COPIERS_AND_MOVERS(type) DELETE_COPIERS(type) DELETE_MOVERS(type)

#endif // __BIG_ENDIAN__
#endif // __APPLE__

#endif //__COMMON_H__
