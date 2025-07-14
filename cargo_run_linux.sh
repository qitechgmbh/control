#!/bin/bash

if [ "$1" == "release" ]; then
    echo "building Release Code"
    cargo build --release
    TARGET=./target/release/server
elif [ "$1" == "mock-machine" ]; then
    echo "building with feature mock-machine"
    cargo build --features mock-machine
    TARGET=./target/debug/server
else
    echo "building Debug Code"
    cargo build
    TARGET=./target/debug/server
fi

sudo setcap cap_net_raw=eip "$TARGET"
"$TARGET"
