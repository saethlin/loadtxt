#!/bin/sh

set -e

git clone --depth=1 https://github.com/saethlin/loadtxt
cd loadtxt
python3 setup.py bdist_wheel
python3 -m auditwheel repair dist/*.whl -w /wheelhouse
