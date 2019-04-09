import numpy as np
from loadtxt._native import ffi, lib
import warnings


def loadtxt(filename, comments="#", skiprows=0, usecols=None):
    row_ptr = ffi.new("uintptr_t *")
    col_ptr = ffi.new("uintptr_t *")
    error_ptr = ffi.new("char **")

    if usecols is not None:
        usecols = np.array(usecols, dtype=np.uint64)
        usecols.sort()
        data_ptr = lib.loadtxt(
            filename.encode(),
            comments.encode(),
            skiprows,
            ffi.cast("uint64_t *", ffi.from_buffer(usecols)),
            usecols.shape[0],
            row_ptr,
            col_ptr,
            error_ptr,
        )
    else:
        data_ptr = lib.loadtxt(
            filename.encode(),
            comments.encode(),
            skiprows,
            ffi.NULL,
            0,
            row_ptr,
            col_ptr,
            error_ptr,
        )

    if data_ptr == ffi.NULL:
        raise RuntimeError(ffi.string(error_ptr[0]).decode("utf-8"))

    rows = row_ptr[0]
    cols = col_ptr[0]

    buf = ffi.buffer(data_ptr, 8 * rows * cols)
    array = np.frombuffer(buf, dtype=np.float64, count=rows * cols)
    array.shape = (rows, cols)

    array = array.copy()
    lib.loadtxt_free(data_ptr, rows * cols)

    if rows * cols == 0:
        warnings.warn('loadtxt: Empty input file: "{}"'.format(filename), stacklevel=2)

    return array
