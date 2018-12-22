import numpy as np
from loadtxt._native import ffi, lib


def loadtxt(filename, comments='#', skiprows=0, transpose=False):
    row_ptr = ffi.new("uint64_t *")
    col_ptr = ffi.new("uint64_t *")
    has_error = ffi.new("uint8_t *")
    error_line = ffi.new("uint64_t *")

    data_ptr = lib.loadtxt(filename.encode(), comments.encode(), skiprows, row_ptr, col_ptr, has_error, error_line)

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
    size_ptr = ffi.new("uint64_t *")

    if dtype == int or dtype == np.int64:
        dtype = np.int64
        data_ptr = lib.loadtxt_i64_unchecked(filename.encode(), size_ptr)

    elif dtype == float or dtype == np.float64:
        dtype = np.float64
        data_ptr = lib.loadtxt_f64_unchecked(filename.encode(), size_ptr)

    if data_ptr == ffi.NULL:
        raise RuntimeError("Unchecked parsing failed. Use loadtxt.loadtxt to get useful errors")

    size = size_ptr[0]
    buf = ffi.buffer(data_ptr, 8 * size)

    return np.frombuffer(buf, dtype=dtype, count=size)
