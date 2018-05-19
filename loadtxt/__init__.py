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


def loadtxt_unchecked(filename):
    size_ptr = ffi.new("uint64_t *")

    data_ptr = lib.loadtxt_unchecked(filename.encode(), size_ptr)
    size = size_ptr[0]

    buf = ffi.buffer(data_ptr, 8 * size)
    return np.frombuffer(buf, dtype=np.int64, count=size)
