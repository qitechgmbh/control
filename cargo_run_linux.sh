#!/bin/bash

FEATURE=""

# Check if "mock-machine" was passed as an argument
if [ "$1" == "mock-machine" ]; then
    echo "Building Debug Code with mock-machine feature"
    FEATURE="--features mock-machine"
else
    echo "Building Debug Code"
fi

# Build the project with or without features
cargo build $FEATURE

# Only setcap if not building with mock-machine
if [ "$1" != "mock-machine" ]; then
    echo "Setting capabilities for server executable"
    sudo setcap cap_net_raw=eip ./target/debug/server
fi

# Run the server
./target/debug/server
