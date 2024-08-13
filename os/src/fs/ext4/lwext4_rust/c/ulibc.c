#include <string.h>
#include <stdint.h>
#include <limits.h>

#include "ext4_debug.h"

// +++++++++ musl +++++++++

#define ALIGN (sizeof(size_t))
#define ONES ((size_t) - 1 / UCHAR_MAX)
#define HIGHS (ONES * (UCHAR_MAX / 2 + 1))
#define HASZERO(x) (((x) - ONES) & ~(x) & HIGHS)

__attribute__((weak)) 
char *__stpcpy(char *restrict d, const char *restrict s)
{
#ifdef __GNUC__
        typedef size_t __attribute__((__may_alias__)) word;
        word *wd;
        const word *ws;
        if ((uintptr_t)s % ALIGN == (uintptr_t)d % ALIGN)
        {
                for (; (uintptr_t)s % ALIGN; s++, d++)
                        if (!(*d = *s))
                                return d;
                wd = (void *)d;
                ws = (const void *)s;
                for (; !HASZERO(*ws); *wd++ = *ws++)
                        ;
                d = (void *)wd;
                s = (const void *)ws;
        }
#endif
        for (; (*d = *s); s++, d++)
                ;

        return d;
}

__attribute__((weak)) 
char *strcpy(char *restrict dest, const char *restrict src)
{
        __stpcpy(dest, src);
        return dest;
}

__attribute__((weak)) 
int strcmp(const char *l, const char *r)
{
        for (; *l == *r && *l; l++, r++)
                ;
        return *(unsigned char *)l - *(unsigned char *)r;
}

__attribute__((weak)) 
int strncmp(const char *_l, const char *_r, size_t n)
{
        const unsigned char *l = (void *)_l, *r = (void *)_r;
        if (!n--)
                return 0;
        for (; *l && *r && n && *l == *r; l++, r++, n--)
                ;
        return *l - *r;
}

// fix me
__attribute__((weak)) 
FILE *const stdout = NULL;

__attribute__((weak)) 
int fflush(FILE *f)
{
        // printf("fflush() is not implemented !\n");
        return 0;
}

// +++++++++ uClibc +++++++++

__attribute__((weak)) 
void *memset(void *s, int c, size_t n)
{
        register unsigned char *p = (unsigned char *)s;
        while (n)
        {
                *p++ = (unsigned char)c;
                --n;
        }
        return s;
}

// musl, typedef int (*cmpfun)(const void *, const void *);
typedef int (*__compar_fn_t)(const void *, const void *);
typedef int (*__compar_d_fn_t)(const void *, const void *, void *);
__attribute__((weak)) 
void qsort_r(void *base,
             size_t nel,
             size_t width,
             __compar_d_fn_t comp,
             void *arg)
{
        size_t wgap, i, j, k;
        char tmp;

        if ((nel > 1) && (width > 0))
        {
                // check for overflow
                // assert(nel <= ((size_t)(-1)) / width);
                ext4_assert(nel <= ((size_t)(-1)) / width);
                wgap = 0;
                do
                {
                        wgap = 3 * wgap + 1;
                } while (wgap < (nel - 1) / 3);
                /* From the above, we know that either wgap == 1 < nel or */
                /* ((wgap-1)/3 < (int) ((nel-1)/3) <= (nel-1)/3 ==> wgap <  nel. */
                wgap *= width; /* So this can not overflow if wnel doesn't. */
                nel *= width;  /* Convert nel to 'wnel' */
                do
                {
                        i = wgap;
                        do
                        {
                                j = i;
                                do
                                {
                                        register char *a;
                                        register char *b;

                                        j -= wgap;
                                        a = j + ((char *)base);
                                        b = a + wgap;
                                        if ((*comp)(a, b, arg) <= 0)
                                        {
                                                break;
                                        }
                                        k = width;
                                        do
                                        {
                                                tmp = *a;
                                                *a++ = *b;
                                                *b++ = tmp;
                                        } while (--k);
                                } while (j >= wgap);
                                i += width;
                        } while (i < nel);
                        wgap = (wgap - width) / 3;
                } while (wgap);
        }
}

__attribute__((weak)) 
void qsort(void *base,
           size_t nel,
           size_t width,
           __compar_fn_t comp)
{
        return qsort_r(base, nel, width, (__compar_d_fn_t)comp, NULL);
}
