import loadtxt
import numpy as np
import timeit
import os

if not os.path.exists('data.txt'):
    print('generating bench data')
    data = np.random.uniform(0.0, 1.0, 1_000_000)
    data.shape = -1, 10
    np.savetxt('data.txt', data)


#assert np.all(np.loadtxt('data.txt') == loadtxt.loadtxt('data.txt'))

assert np.all(loadtxt.loadtxt('data.txt')[:90_000] == loadtxt.loadtxt('data.txt', max_rows=90_000))

ldt = timeit.repeat("loadtxt.loadtxt('data.txt')", repeat=100, number=1, globals=globals())
print('loadtxt: {:.3f}'.format(np.min(ldt)))

ldt = timeit.repeat("loadtxt.loadtxt('data.txt', max_rows=90_000)", repeat=100, number=1, globals=globals())
print('loadtxt: {:.3f}'.format(np.min(ldt)))

'''
npy = timeit.repeat("np.loadtxt('data.txt')", repeat=10, number=1, globals=globals())
print('numpy: {:.3f}'.format(np.min(npy)))

print('ratio: {:.1f}'.format(np.min(npy) / np.min(ldt)))
'''
