#!/bin/bash

send_add() {
    data=$1
    max_clicks=$2
    lifetime=$3
    msg="{\"data\": \"$data\", \"max_clicks\": $max_clicks, \"lifetime\": $lifetime}"
    js=$(curl -d "$msg" http://127.0.0.1:8080/add 2>/dev/null)
    echo "$js"
}

send_get() {
    url=$1
    resp=$(curl "$url" 2>/dev/null)
    echo "$resp"
}

prepare_env() {
    test_id=$1
    config_fn=$2
    test_dir="$WORK_DIR/$test_id"
    mkdir -p "$test_dir"

    cp "$TESTS_DIR/$config_fn" "$test_dir"

    echo "$test_dir"
}


FILE=$(realpath "$0")
tests_dir=$(dirname "$FILE")
root_dir=$(realpath "$tests_dir/..")

export PAYLOAD="my secret data"

export ROOT_DIR="$root_dir"
export TESTS_DIR="$tests_dir"
export WORK_DIR="$TESTS_DIR/_UNIVERSE_"

