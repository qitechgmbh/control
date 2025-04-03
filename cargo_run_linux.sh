#!/bin/bash

if [ $1 == "release" ]; then
    echo "building Release Code"
    cargo build --release
else
    echo "building Debug Code"
    cargo build
fi
# set cap for server executable
sudo setcap cap_net_raw=eip ./target/debug/server
# run
./target/debug/server
