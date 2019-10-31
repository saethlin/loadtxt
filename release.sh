#!/bin/sh

set -e

git clone https://github.com/saethlin/loadtxt
cd loadtxt
python3 setup.py bdist_wheel
python3 -m auditwheel repair dist/loadtxt*.whl -w /wheelhouse
