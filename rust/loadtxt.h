#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

const double *loadtxt(const char *filename,
                      const char *comments,
                      int skiprows,
                      uint64_t *rows,
                      uint64_t *cols,
                      uint8_t *has_error,
                      uint64_t *error_line);

const double *loadtxt_f64_unchecked(const char *filename, uint64_t *size);

const int64_t *loadtxt_i64_unchecked(const char *filename, uint64_t *size);
