import loadtxt
import numpy as np
import time
import os

if not os.path.exists('data.txt'):
    print('generating bench data')
    data = np.random.uniform(np.finfo(np.float32).max, np.finfo(np.float32).min, 10_000_000)
    data.shape = -1, 10
    np.savetxt('data.txt', data)

n = 20
checked = loadtxt.loadtxt('data.txt')
start = time.time()
for _ in range(n):
    checked = loadtxt.loadtxt('data.txt')
print('checked', (time.time()-start) / n)
