import loadtxt
import numpy as np

assert np.all(loadtxt.loadtxt('test.txt') == np.loadtxt("test.txt"))

assert np.all(loadtxt.loadtxt_flat("test.txt") ==
              np.loadtxt("test.txt").flatten())

assert np.all(
    loadtxt.loadtxt_flat(
        "test.txt",
        dtype=int) == np.loadtxt(
            "test.txt",
        dtype=int).flatten())

assert np.all(
    loadtxt.loadtxt(
        'test.txt',
        skiprows=1) == np.loadtxt(
            'test.txt',
        skiprows=1))

assert np.all(loadtxt.loadtxt_unsafe('test.txt') == np.loadtxt('test.txt').flatten())

