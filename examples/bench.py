import loadtxt
import numpy as np
import time

data = np.random.uniform(np.finfo(np.float32).max, np.finfo(np.float32).min, 1_000_000)
data.shape = -1, 10
#data = np.random.rand(100_000, 10)
np.savetxt('data.txt', data)

start = time.time()
from_loadtxt = loadtxt.loadtxt_unchecked('data.txt', float)
print('loadtxt', time.time() - start)

start = time.time()
from_numpy = np.loadtxt('data.txt')
print('numpy', time.time()-start)

assert from_loadtxt.shape == from_numpy.shape
assert np.all(from_loadtxt == from_numpy)
