#!/bin/bash

export CARGO_BUILD_JOBS=8

# Nix store uses sqlite ... SO HAHAHAHAHH NO CONCURRENT ACCESS AHAHAHHAHAHAHHAHAHAHAH
time nix build .#packages.x86_64-linux.server --extra-experimental-features nix-command --extra-experimental-features flakes --out-link /tmp/server-result
serverstore=$(readlink -f /tmp/server-result)
time nix build .#packages.x86_64-linux.electron --extra-experimental-features nix-command --extra-experimental-features flakes --out-link /tmp/electron-result
electronstore=$(readlink -f /tmp/electron-result)

# Copy the results to the store
# nix copy --to file:///tmp/server "$serverstore" --extra-experimental-features nix-command
# nix copy --to file:///tmp/server "$electronstore" --extra-experimental-features nix-command

nix-store --generate-binary-cache-key cache.key cache.pem --extra-experimental-features nix-command
nix-store sign --key-file "$NIXCACHE_SIGNING_KEY" "$serverstore" "$electronstore" --extra-experimental-features nix-command




#export
#nix-store --export /nix/store/<hash>-server > nix-cache/server.nar
#nix-store --export /nix/store/<hash>-electron > nix-cache/electron.nar

# Any http server
# cd ~/nix-cache
# python3 -m http.server 8080
# 