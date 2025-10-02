#!/bin/sh

set -euo pipefail

# Capture all git information
export GIT_TIMESTAMP=$(git --no-pager show -s --format=%cI HEAD)  # e.g., "2025-06-10T14:30:45+02:00"
export GIT_COMMIT=$(git rev-parse HEAD)                           # e.g., "b2c7f6e0b138174770798f84ada8b0aa65afeb"
export GIT_URL=$(git config --get remote.origin.url)             # e.g., "https://github.com/qitechindustries/control.git"

# Determine the git abbreviation (tag, branch, or commit hash)
export GIT_TAG=$(git describe --tags --exact-match HEAD 2>/dev/null || echo "")
if [ -n "$GIT_TAG" ]; then
  export GIT_ABBREVIATION="$GIT_TAG"                              # e.g., "2.0.0" (when on a tag)
else
  GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
  if [ "$GIT_BRANCH" = "HEAD" ]; then
    export GIT_ABBREVIATION=$(git rev-parse --short HEAD)         # e.g., "b2c7f6e" (when on detached HEAD/commit)
  else
    export GIT_ABBREVIATION="$GIT_BRANCH"                         # e.g., "main", "develop" (when on a branch)
  fi
fi

# Create escaped version for system.nixos.label
export GIT_ABBREVIATION_ESCAPED=$(echo "$GIT_ABBREVIATION" | sed -e 's/+/-/g' -e 's/[^a-zA-Z0-9:_\.-]//g')  # e.g., "2-0-0", "main", "b2c7f6e"

sudo \
    --preserve-env=GIT_TIMESTAMP \
    --preserve-env=GIT_COMMIT \
    --preserve-env=GIT_URL \
    --preserve-env=GIT_ABBREVIATION \
    --preserve-env=GIT_ABBREVIATION_ESCAPED \
    nixos-rebuild boot \
    	--flake .#nixos \
    	--show-trace \
    	--impure \
    	--option sandbox false \
    	--option eval-cache false

sudo reboot
