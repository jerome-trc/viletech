#pragma once

#if defined(__unix)

#include <stddef.h>

/* Compare S1 and S2, ignoring case.  */
int strcasecmp (const char *__s1, const char *__s2);

/* Compare no more than N chars of S1 and S2, ignoring case.  */
int strncasecmp (const char *__s1, const char *__s2, size_t __n);

char *strdup (const char *__s);

char *strsignal (int __sig);

size_t strnlen (const char *__string, size_t __maxlen);

/* File types for `d_type'.  */
enum
  {
    DT_UNKNOWN = 0,
# define DT_UNKNOWN	DT_UNKNOWN
    DT_FIFO = 1,
# define DT_FIFO	DT_FIFO
    DT_CHR = 2,
# define DT_CHR		DT_CHR
    DT_DIR = 4,
# define DT_DIR		DT_DIR
    DT_BLK = 6,
# define DT_BLK		DT_BLK
    DT_REG = 8,
# define DT_REG		DT_REG
    DT_LNK = 10,
# define DT_LNK		DT_LNK
    DT_SOCK = 12,
# define DT_SOCK	DT_SOCK
    DT_WHT = 14
# define DT_WHT		DT_WHT
  };

#endif // if defined(__unix)
