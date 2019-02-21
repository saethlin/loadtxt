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
    columns = col_ptr[0]

    buf = ffi.buffer(data_ptr, 8 * rows * columns)
    array = np.frombuffer(buf, dtype=np.float64, count=rows * columns)
    array.shape = (rows, columns)

    array = array.copy()
    lib.free(data_ptr, rows * columns)

    return array


def loadtxt_unchecked(filename, dtype):
    rows_ptr = ffi.new("uintptr_t *")
    cols_ptr = ffi.new("uintptr_t *")
    error_ptr = ffi.new("char **")

    if dtype == int or dtype == np.int64:
        dtype = np.int64
        data_ptr = lib.loadtxt_i64_unchecked(
            filename.encode(), rows_ptr, cols_ptr, error_ptr
        )

    elif dtype == float or dtype == np.float64:
        dtype = np.float64
        data_ptr = lib.loadtxt_f64_unchecked(
            filename.encode(), rows_ptr, cols_ptr, error_ptr
        )

    else:
        raise ValueError(f"Unsupported dtype {dtype}")

    if data_ptr == ffi.NULL:
        raise RuntimeError(error = ffi.string(error_ptr[0]).decode("utf-8"))

    rows = rows_ptr[0]
    cols = cols_ptr[0]
    buf = ffi.buffer(data_ptr, 8 * rows * cols)

    arr = np.frombuffer(buf, dtype=dtype, count=rows * cols)
    arr.shape = (rows, cols)
    return arr
