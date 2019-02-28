import numpy as np
from loadtxt._native import ffi, lib


def loadtxt(filename, comments="#", skiprows=0):
    row_ptr = ffi.new("uintptr_t *")
    col_ptr = ffi.new("uintptr_t *")
    error_ptr = ffi.new("char **")

    data_ptr = lib.loadtxt(
        filename.encode(), comments.encode(), skiprows, row_ptr, col_ptr, error_ptr
    )

    if data_ptr == ffi.NULL:
        raise RuntimeError(ffi.string(error_ptr[0]).decode("utf-8"))

    rows = row_ptr[0]
    cols = col_ptr[0]

    buf = ffi.buffer(data_ptr, 8 * rows * cols)
    array = np.frombuffer(buf, dtype=np.float64, count=rows * cols)
    array.shape = (rows, cols)

    array = array.copy()
    lib.free(data_ptr, rows * cols)

    return array

