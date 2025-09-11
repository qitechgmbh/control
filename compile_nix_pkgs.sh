#!/bin/bash

# Nix store uses sqlite ... SO HAHAHAHAHH NO CONCURRENT ACCESS AHAHAHHAHAHAHHAHAHAHAH
time nix build .#packages.x86_64-linux.server --impure --extra-experimental-features nix-command --extra-experimental-features flakes --out-link /tmp/server-result
serverstore=$(readlink -f /tmp/server-result) 

time nix build .#packages.x86_64-linux.electron --impure --option sandbox false --extra-experimental-features nix-command --extra-experimental-features flakes --out-link /tmp/electron-result
electronstore=$(readlink -f /tmp/electron-result)


nix copy --to "file:///opt/dev-cache" \
  $(readlink -f /tmp/server-result) \
  $(readlink -f /tmp/electron-result) --extra-experimental-features nix-command

nix store sign \
  --key-file /opt/dev-cache/cache-private-key.pem \
  --store "file:///opt/dev-cache" \
  --extra-experimental-features nix-command \
  "$serverstore" "$electronstore"


nix path-info "$serverstore" --extra-experimental-features nix-command --extra-experimental-features flakes


# nix verify --store "file:///opt/dev-cache" \
#   --trusted-public-keys "$(cat /opt/dev-cache/cache-public-key.pem)" --extra-experimental-features nix-command --extra-experimental-features flakes \
#   "$FIRST_PATH" || echo "Verification failed!" 


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