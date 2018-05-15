import numpy as np
from loadtxt._native import ffi,lib

def loadtxt(filename, skiprows=0):
    row_ptr = ffi.new("int *")
    col_ptr = ffi.new("int *")
    
    data_ptr = lib.loadtxt(filename.encode(), skiprows, row_ptr, col_ptr)
    rows = row_ptr[0]
    columns = col_ptr[0]
    
    buf = ffi.buffer(data_ptr, 8*rows*columns)
    array = np.frombuffer(buf, dtype=np.float64, count=rows*columns)
    array.shape = (rows, columns)
    return array

def loadtxt_flat(filename, dtype=float):
    size_ptr = ffi.new("uint64_t *")
   
    if dtype == float:
        data_ptr = lib.loadtxt_flat_f64(filename.encode(), size_ptr)
        size = size_ptr[0]

        buf = ffi.buffer(data_ptr, 8*size)
        return np.frombuffer(buf, dtype=np.float64, count=size)

    elif dtype == int:
        data_ptr = lib.loadtxt_flat_i64(filename.encode(), size_ptr)
        size = size_ptr[0]

        buf = ffi.buffer(data_ptr, 8*size)
        return np.frombuffer(buf, dtype=np.int64, count=size)

    else:
        raise ValueError("Unsupported data type: {}".format(dtype))

