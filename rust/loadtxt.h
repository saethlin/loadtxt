#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

void free(double *ptr, uintptr_t len);

const double *loadtxt(const char *filename,
                      const char *comments,
                      int skiprows,
                      uint64_t *rows,
                      uint64_t *cols,
                      const char **error);

const double *loadtxt_f64_unchecked(const char *filename,
                                    uint64_t *rows,
                                    uint64_t *columns,
                                    const char **error);

const int64_t *loadtxt_i64_unchecked(const char *filename,
                                     uint64_t *rows,
                                     uint64_t *columns,
                                     const char **error);
