import numpy as np
import loadtxt
import glob

for filename in sorted(glob.glob("*.txt")):
    print(filename)
    try:
        numpy = np.loadtxt(filename)
    except Exception as e:
        print(e)
    try:
        mine = loadtxt.loadtxt(filename)
    except Exception as e:
        print(e)
    assert np.all(numpy == mine)
