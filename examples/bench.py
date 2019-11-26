import loadtxt
import numpy as np
import time
import os

if not os.path.exists('data.txt'):
    print('generating bench data')
    data = np.random.uniform(0.0, 1.0, 1_000_000)
    data.shape = -1, 10
    np.savetxt('data.txt', data)

n = 100
start = time.time()
for _ in range(n):
     loadtxt.loadtxt('data.txt')
print('checked', (time.time()-start) / n)

start = time.time()
np_ver = np.loadtxt('data.txt')
print('numpy', (time.time()-start))

print(np.finfo(np_ver.dtype).eps)
print(np.max(np.abs(np_ver - loadtxt_ver)/np_ver))
