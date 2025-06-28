#!/usr/bin/env bash

# Script to update flake.lock using a Docker container with Nix
# This ensures consistent updates regardless of the host system

set -euo pipefail

# Get the repository root directory
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "Repository root: $REPO_ROOT"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in PATH"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    echo "Error: Docker daemon is not running"
    exit 1
fi

echo "Updating flake.lock using Docker..."

# Run nix flake update in a Docker container
# We mount the entire repository to /workspace and work from there
docker run --rm \
    --volume "$REPO_ROOT:/workspace" \
    --workdir /workspace \
    nixos/nix:latest \
    sh -c "
        echo 'Enabling flakes and nix-command...'
        mkdir -p ~/.config/nix
        echo 'experimental-features = nix-command flakes' > ~/.config/nix/nix.conf
        
        echo 'Current flake inputs:'
        nix flake metadata --no-write-lock-file
        
        echo 'Updating flake.lock...'
        nix flake update
        
        echo 'New flake inputs:'
        nix flake metadata --no-write-lock-file
        
        echo 'Checking flake evaluation...'
        nix flake check --no-build
        
        echo 'Done!'
    "

echo "Flake.lock updated successfully!"
echo "You can now commit the changes to flake.lock"
