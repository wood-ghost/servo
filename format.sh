#!/bin/bash

REPOSITORY_ROOT="/home/yizhiy/Desktop/servo"
NR_ROOT="${REPOSITORY_ROOT}/components/shared/net/src/"

# check if verusfmt is installed
if ! command -v verusfmt &> /dev/null
then
    cargo install verusfmt
fi

pushd ${NR_ROOT} > /dev/null
find . -type f -name '*.rs' -exec verusfmt {} \;
popd > /dev/null