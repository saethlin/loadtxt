import numpy as np
from loadtxt._native import ffi, lib


def loadtxt(filename, comments='#', skiprows=0, transpose=False):
    row_ptr = ffi.new("uint64_t *")
    col_ptr = ffi.new("uint64_t *")

    data_ptr = lib.loadtxt(filename.encode(), comments.encode(), skiprows, row_ptr, col_ptr)
    rows = row_ptr[0]
    columns = col_ptr[0]

    buf = ffi.buffer(data_ptr, 8 * rows * columns)
    array = np.frombuffer(buf, dtype=np.float64, count=rows * columns)
    array.shape = (rows, columns)
    if transpose:
        array = np.transpose(array)
    return array


def loadtxt_flat(filename, dtype=float):
    size_ptr = ffi.new("uint64_t *")

    if dtype == float:
        data_ptr = lib.loadtxt_flat_f64(filename.encode(), size_ptr)
        size = size_ptr[0]

        buf = ffi.buffer(data_ptr, 8 * size)
        return np.frombuffer(buf, dtype=np.float64, count=size)

    elif dtype == int:
        data_ptr = lib.loadtxt_flat_i64(filename.encode(), size_ptr)
        size = size_ptr[0]

        buf = ffi.buffer(data_ptr, 8 * size)
        return np.frombuffer(buf, dtype=np.int64, count=size)

    else:
        raise ValueError("Unsupported data type: {}".format(dtype))


def loadtxt_unsafe(filename):
    size_ptr = ffi.new("uint64_t *")

    data_ptr = lib.loadtxt_unsafe_i64(filename.encode(), size_ptr)
    size = size_ptr[0]

    buf = ffi.buffer(data_ptr, 8 * size)
    return np.frombuffer(buf, dtype=np.int64, count=size)
