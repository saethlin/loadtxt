import numpy as np
import loadtxt
import glob

for filename in sorted(glob.glob("*.txt")):
    print(filename)
    numpy = np.loadtxt(filename)
    mine = np.loadtxt(filename)
    assert np.all(numpy == mine)
