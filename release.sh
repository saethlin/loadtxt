#!/bin/sh

set -e

git clone --depth=1 https://github.com/saethlin/loadtxt
cd loadtxt
python3 setup.py bdist_wheel
python3 -m auditwheel repair dist/*.whl

cd wheelhouse
COMMIT_HASH=$(git log -n 1 --pretty=format:"%H" | cut -c-7)
curl -H "Authorization: token ${GITHUB_TOKEN}" -H "Accept: application/vnd.github.v3+json" --data "{\"tag_name\": \"${COMMIT_HASH}\"}" https://api.github.com/repos/saethlin/loadtxt/releases >  /tmp/response
ID=$(python3 -c 'import json,sys;obj=json.load(sys.stdin);print(obj["id"])' < /tmp/response)
WHEEL=$(ls loadtxt*.whl)
curl -H "Authorization: token ${GITHUB_TOKEN}" -H "Accept: application/vnd.github.v3+json" -H "Content-type: application/octet-stream" --data-binary @${WHEEL} "https://uploads.github.com/repos/saethlin/loadtxt/releases/${ID}/assets?name=${WHEEL}"
