import loadtxt
import numpy as np
import timeit
import os

if not os.path.exists('data.txt'):
    print('generating bench data')
    data = np.random.uniform(0.0, 1.0, 1_000_000)
    data *= 1_000_000
    data = data.astype(np.uint64)
    data.shape = -1, 10
    np.savetxt('data.txt', data, fmt="%d")


ldt = timeit.repeat("loadtxt.loadtxt('data.txt', dtype=np.int64)", repeat=100, number=1, globals=globals())
print('loadtxt: {:.3f}'.format(np.min(ldt)))

npy = timeit.repeat("np.loadtxt('data.txt', dtype=np.uint64)", repeat=1, number=1, globals=globals())
print('numpy: {:.3f}'.format(np.min(npy)))

print('ratio: {:.1f}'.format(np.min(npy) / np.min(ldt)))
