#!/bin/bash

set -e
# set -x

FILE=$(realpath "$0")
tests_dir=$(dirname "$FILE")

for f in "$tests_dir"/T*.sh; do
    for db in "memory" "sqlite" "file" "mysql" "postgresql"; do
        "$f" $db
    done
done
