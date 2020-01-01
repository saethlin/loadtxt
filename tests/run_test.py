import numpy as np
import loadtxt
import glob

files = sorted(glob.glob("*.txt"))
print("test files:", files)
for filename in files:
    print(filename)
    try:
        numpy = np.loadtxt(filename)
    except Exception as e:
        print(e)
    try:
        mine = loadtxt.loadtxt(filename)
    except Exception as e:
        print(e)

    if not np.all(numpy == mine):
        print(numpy.shape, numpy)
        print(mine.shape, mine)
        raise AssertionError
