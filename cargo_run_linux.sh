#!/bin/bash

set -e

FEATURE=""

# Check if "mock-machine" was passed as an argument
if [[ "$1" == "mock-machine" ]]; then
    echo "Building Debug Code with mock-machine feature"
    FEATURE="--features mock-machine"
fi

if [[ "$1" == "release" ]]; then
    echo "building Release Code"
    cargo build --release
else
    echo "Building Debug Code"
    cargo build $FEATURE
fi

# Only setcap if not building with mock-machine
if [ "$1" != "mock-machine" ]; then
    echo "Setting capabilities for server executable"
    sudo setcap 'cap_net_raw,cap_sys_nice=eip' ./target/debug/server
fi

# run
./target/debug/server
