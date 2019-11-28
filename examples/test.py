import loadtxt
import numpy as np

assert np.all(loadtxt.loadtxt('test.txt') == np.loadtxt("test.txt"))

assert np.all(
    loadtxt.loadtxt(
        'test.txt',
        skiprows=1) == np.loadtxt(
            'test.txt',
        skiprows=1))

