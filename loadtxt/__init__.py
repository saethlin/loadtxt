import numpy as np
from loadtxt._native import ffi, lib


def loadtxt(filename, comments="#", skiprows=0, transpose=False):
    row_ptr = ffi.new("uint64_t *")
    col_ptr = ffi.new("uint64_t *")
    has_error = ffi.new("uint8_t *")
    error_line = ffi.new("uint64_t *")

    data_ptr = lib.loadtxt(
        filename.encode(),
        comments.encode(),
        skiprows,
        row_ptr,
        col_ptr,
        has_error,
        error_line,
    )

    if has_error[0]:
        raise ValueError("Parsing failed at line {}.".format(error_line[0]))

    rows = row_ptr[0]
    columns = col_ptr[0]

    buf = ffi.buffer(data_ptr, 8 * rows * columns)
    array = np.frombuffer(buf, dtype=np.float64, count=rows * columns)
    array.shape = (rows, columns)
    if transpose:
        array = np.transpose(array)
    return array


def loadtxt_unchecked(filename, dtype):
    rows_ptr = ffi.new("uint64_t *")
    cols_ptr = ffi.new("uint64_t *")
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
        error = ffi.string(error_ptr[0]).decode("utf-8")
        raise RuntimeError(f"Parsing failed: {error}")

    rows = rows_ptr[0]
    cols = cols_ptr[0]
    buf = ffi.buffer(data_ptr, 8 * rows * cols)

    arr = np.frombuffer(buf, dtype=dtype, count=rows * cols)
    arr.shape = (rows, cols)
    return arr
