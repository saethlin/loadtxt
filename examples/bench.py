import loadtxt
import numpy as np
import time
import os

if not os.path.exists('data.txt'):
    print('generating bench data')
    data = np.random.uniform(np.finfo(np.float32).max, np.finfo(np.float32).min, 1_000_000)
    data.shape = -1, 10
    np.savetxt('data.txt', data)

n = 20
start = time.time()
for _ in range(n):
     loadtxt.loadtxt('data.txt')
print('checked', (time.time()-start) / n)

start = time.time()
np.loadtxt('data.txt')
print('numpy', (time.time()-start))
