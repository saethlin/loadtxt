import numpy as np
from loadtxt._native import ffi,lib

def loadtxt(filename, skiprows=0):
    row_ptr = ffi.new("int *")
    col_ptr = ffi.new("int *")
    
    data_ptr = lib.loadtxt(filename, skiprows, row_ptr, col_ptr)
    rows = row_ptr[0]
    columns = col_ptr[0]
    
    buf = ffi.buffer(data_ptr, 8*rows*columns)
    array = np.frombuffer(buf, dtype=np.float64, count=rows*columns)
    array.shape = (rows, columns)
    return array

