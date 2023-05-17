#!/bin/bash

# set -x
set -e

FILE=$(realpath "$0")
tests_dir=$(dirname "$FILE")

# shellcheck disable=SC1091
source "$tests_dir/utils.sh"

# Prepare work dir for current test
test_id=$(basename "$0")
db_type=$1

config_fn="config_${db_type}.toml"
work_dir=$(prepare_env "$test_id" "$config_fn")
cd "$work_dir"

# Run service
"$ROOT_DIR/target/release/onetimer" "$config_fn" &
pid=$!
sleep 2
trap 'kill $pid' EXIT

echo "[$test_id] Check lifetime [$db_type]:"

# Check lifetime
add_resp=$(send_add "$PAYLOAD" "3" "3")
status=$( echo "$add_resp" | jq -r .status )
if [ "$status" != OK ]; then
    echo "ADD FAILED"
    exit 1
fi
sleep 5

url=$( echo "$add_resp" | jq -r .msg )
get_resp=$(send_get "$url")
msg=$( echo "$get_resp" | jq -r .msg )
resp_status=$( echo "$get_resp" | jq -r .status )
if [ "$resp_status" == OK ]; then
    echo "GET STATUS FAILED"
    exit 1
fi
if [ "$msg" == "$PAYLOAD" ]; then
    echo "GET FAILED"
    exit 1
fi

echo OK
