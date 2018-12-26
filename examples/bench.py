import loadtxt
import numpy as np
import time

data = np.random.rand(100_000, 10)
np.savetxt('data.txt', data)

start = time.time()
from_loadtxt = loadtxt.loadtxt_unchecked('data.txt', float)
print('loadtxt', time.time() - start)

start = time.time()
from_numpy = np.loadtxt('data.txt')
print('numpy', time.time()-start)

assert np.all(from_loadtxt == from_numpy)
