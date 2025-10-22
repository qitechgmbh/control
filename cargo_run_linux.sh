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
    cargo build $FEATURE --features development-build
fi

# Only setcap if not building with mock-machine
if [ "$1" != "mock-machine" ]; then
    echo "Setting capabilities for server executable"
    sudo setcap 'cap_dac_override,cap_net_raw,cap_sys_nice,cap_ipc_lock=eip' ./target/debug/server

    if compgen -G "/dev/ttyUSB*" > /dev/null; then
      echo "Setting permissions for serial ports"
      sudo chown "root:$(groups | cut -f1 -d' ')" /dev/ttyUSB*
    fi
fi

# run
./target/debug/server
