import numpy as np
from loadtxt._native import ffi, lib
import warnings


def loadtxt(filename, comments="#", skiprows=0, usecols=None):
    rows_ptr = ffi.new("uintptr_t *")
    cols_ptr = ffi.new("uintptr_t *")
    error_ptr = ffi.new("char **")

    if usecols is not None:
        usecols = np.array(usecols, dtype=np.uint64)
        usecols.sort()
        chunks_ptr = lib.loadtxt_get_chunks(
            filename.encode(),
            comments.encode(),
            skiprows,
            ffi.cast("uint64_t *", ffi.from_buffer(usecols)),
            usecols.shape[0],
            rows_ptr,
            cols_ptr,
            error_ptr,
        )
    else:
        chunks_ptr = lib.loadtxt_get_chunks(
            filename.encode(),
            comments.encode(),
            skiprows,
            ffi.NULL,
            0,
            rows_ptr,
            cols_ptr,
            error_ptr,
        )

    if chunks_ptr == ffi.NULL:
        raise RuntimeError(ffi.string(error_ptr[0]).decode("utf-8"))

    if rows_ptr[0]*cols_ptr[0] == 0:
        warnings.warn('loadtxt: Empty input file: "{}"'.format(filename), stacklevel=2)

    array = np.empty((rows_ptr[0], cols_ptr[0]), dtype=np.float64)
    lib.loadtxt_flatten_chunks(chunks_ptr, ffi.cast("double *", array.ctypes.data))

    return array
