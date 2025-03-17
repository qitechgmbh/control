docker run -it --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  -e HOME=/tmp \
  -e NIX_CONFIG="experimental-features = nix-command flakes" \
  nixos/nix:latest \
  bash -c "git config --global --add safe.directory /workspace && bash"