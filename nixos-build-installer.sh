#!/bin/sh

set -euo pipefail

# Capture all git information (same logic as nixos-install.sh)
export GIT_TIMESTAMP=$(git --no-pager show -s --format=%cI HEAD)
export GIT_COMMIT=$(git rev-parse HEAD)
export GIT_URL=$(git config --get remote.origin.url)

# Determine the git abbreviation (tag, branch, or commit hash)
export GIT_TAG=$(git describe --tags --exact-match HEAD 2>/dev/null || echo "")
if [ -n "$GIT_TAG" ]; then
  export GIT_ABBREVIATION="$GIT_TAG"
else
  GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
  if [ "$GIT_BRANCH" = "HEAD" ]; then
    export GIT_ABBREVIATION=$(git rev-parse --short HEAD)
  else
    export GIT_ABBREVIATION="$GIT_BRANCH"
  fi
fi

export GIT_ABBREVIATION_ESCAPED=$(echo "$GIT_ABBREVIATION" | sed -e 's/+/-/g' -e 's/[^a-zA-Z0-9:_\.-]//g')

echo "Building NixOS installer ISO..."
echo "  Git commit:       $GIT_COMMIT"
echo "  Git abbreviation: $GIT_ABBREVIATION"
echo "  Git URL:          $GIT_URL"
echo ""

nix build \
  .#nixosConfigurations.installer.config.system.build.isoImage \
  --impure \
  --option sandbox false \
  --option eval-cache false \
  --show-trace

ISO_PATH=$(find result/iso -name '*.iso' 2>/dev/null | head -1)
if [ -n "$ISO_PATH" ]; then
  echo ""
  echo "ISO built successfully: $ISO_PATH"
  echo "Flash to USB with: sudo dd if=$ISO_PATH of=/dev/sdX bs=4M status=progress oflag=sync"
else
  echo "ERROR: ISO not found in result/iso/"
  exit 1
fi
