#!/bin/bash

set -e  # Exit on any error

echo "Updating flake.lock using Docker with Nix..."

# Get the repository root directory (go up one level from nixos)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Repository root: $REPO_ROOT"

# Create a temporary script to run inside the Docker container
TEMP_SCRIPT=$(mktemp)
cat > "$TEMP_SCRIPT" << 'EOF'
#!/bin/bash
set -e

cd /workspace

# Configure git if needed
git config --global --add safe.directory /workspace
git config user.name >/dev/null 2>&1 || git config --global user.name "docker-flake-update"
git config user.email >/dev/null 2>&1 || git config --global user.email "docker-flake-update@localhost"

echo "Current working directory: $(pwd)"
echo "Files in directory:"
ls -la

echo ""
echo "Current flake inputs:"
nix flake show || echo "Could not show flake inputs"

echo ""
echo "Updating flake lock file..."

# Update the flake lock
echo "Running nix flake update..."
nix flake update

echo ""
echo "Checking flake after update..."
nix flake check --no-build || echo "Note: Some checks failed, but flake.lock was updated"

echo ""
echo "Updated flake inputs:"
nix flake show || echo "Could not show updated inputs"

echo ""
echo "Flake lock update completed successfully!"
EOF

# Make the script executable
chmod +x "$TEMP_SCRIPT"

echo "Running Docker container to update flake.lock..."

# Run the Docker container with the update script
docker run -it --rm \
  -v "$REPO_ROOT":/workspace \
  -v "$TEMP_SCRIPT":/tmp/update-flake.sh \
  -w /workspace \
  -e HOME=/tmp \
  -e NIX_CONFIG="experimental-features = nix-command flakes" \
  nixos/nix:latest \
  bash /tmp/update-flake.sh

# Clean up
rm -f "$TEMP_SCRIPT"

echo ""
echo "Docker flake update completed successfully!"
echo "The flake.lock file has been updated in: $REPO_ROOT"
echo "You can now run nixos-install.sh with the updated flake.lock"
