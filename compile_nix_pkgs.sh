#!/bin/bash

export CARGO_BUILD_JOBS=8
export COMMIT_HASH=$(git rev-parse HEAD)
# Nix store uses sqlite ... SO HAHAHAHAHH NO CONCURRENT ACCESS AHAHAHHAHAHAHHAHAHAHAH
time nix build .#packages.x86_64-linux.server --impure --extra-experimental-features nix-command --extra-experimental-features flakes --out-link /tmp/server-result
serverstore=$(readlink -f /tmp/server-result)
nix-store --export $serverstore > /tmp/server.nar
ls -lh /tmp/server.nar
echo "$serverstore"
# FROM Here just send it to the dev-cache
printf "Now you need to scp the /tmp/server.nar and /tmp/electron.nar to the dev-cache\n"

cp /tmp/server.nar "/opt/dev-cache/$COMMIT_HASH-server.nar"
ls -lh "/opt/dev-cache/$COMMIT_HASH-server.nar"

nix path-info --hash "$serverstore"



#time nix build .#packages.x86_64-linux.electron --extra-experimental-features nix-command --extra-experimental-features flakes --out-link /tmp/electron-result
#electronstore=$(readlink -f /tmp/electron-result)

#nix-store --generate-binary-cache-key cache.key cache.pem --extra-experimental-features nix-command
#nix store sign --key-file cache.key "$serverstore" --extra-experimental-features nix-command
#nix path-info --json "$serverstore" --extra-experimental-features nix-command --extra-experimental-features flakes

#export
#nix-store --export /nix/store/<hash>-server > nix-cache/server.nar
#nix-store --export /nix/store/<hash>-electron > nix-cache/electron.nar

# Any http server
# cd ~/nix-cache
# python3 -m http.server 8080
# 