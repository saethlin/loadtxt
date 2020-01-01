import numpy as np
from loadtxt._native import ffi, lib
import warnings


def loadtxt(filename,
            comments="#",
            skiprows=0,
            usecols=None,
            max_rows=None,
            dtype=np.float64):
    rows_ptr = ffi.new("uintptr_t *")
    cols_ptr = ffi.new("uintptr_t *")
    error_ptr = ffi.new("char **")
    if max_rows is not None:
        max_rows_ptr = ffi.new("uint64_t *")
        max_rows_ptr[0] = max_rows
    else:
        max_rows_ptr = ffi.NULL

    if dtype == float:
        dtype = np.float64
    if dtype == int:
        dtype = np.int64

    if dtype not in (np.float64, np.int64):
        raise ValueError(
            f"{dtype} is not a valid dtype for loadtxt. You may use int/numpy.int64 or float/numpy.float64"
        )

    filename_bytes = filename.encode()
    comments_bytes = comments.encode()
    if usecols is not None:
        usecols = np.array(usecols, dtype=np.uint64)
        usecols.sort()
        usecols_ptr = ffi.cast("uint64_t *", ffi.from_buffer(usecols))
        usecols_len = usecols.shape[0]
    else:
        usecols_ptr = ffi.NULL
        usecols_len = 0

    args = [
        filename_bytes,
        len(filename_bytes),
        comments_bytes,
        len(comments_bytes),
        skiprows,
        usecols_ptr,
        usecols_len,
        max_rows_ptr,
        rows_ptr,
        cols_ptr,
        error_ptr,
    ]

    if dtype == np.float64:
        chunks_ptr = lib.loadtxt_get_chunks_f64(*args)
    elif dtype == np.int64:
        chunks_ptr = lib.loadtxt_get_chunks_i64(*args)

    if chunks_ptr == ffi.NULL:
        raise RuntimeError(ffi.string(error_ptr[0]).decode("utf-8"))

    if rows_ptr[0] * cols_ptr[0] == 0:
        warnings.warn('loadtxt: Empty input file: "{}"'.format(filename),
                      stacklevel=2)
        return np.empty((0, 0))

    array = np.empty((rows_ptr[0], cols_ptr[0]), dtype=dtype)
    if dtype == np.float64:
        lib.loadtxt_flatten_chunks_f64(chunks_ptr,
                                   ffi.cast("double *", array.ctypes.data))

    elif dtype == np.int64:
        lib.loadtxt_flatten_chunks_i64(chunks_ptr,
                                   ffi.cast("int64_t *", array.ctypes.data))

    return array
