#!/usr/bin/env bash

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

# Lift the realtime core isolation so the update can build on all CPUs.
# --runtime properties live in /run and are dropped on reboot, which restores the configured cpusets.
# If the build fails we don't reboot, so restore them explicitly.
SLICES=(system.slice user.slice)
declare -A ORIGINAL_CPUS
for slice in "${SLICES[@]}"; do
  ORIGINAL_CPUS[$slice]=$(systemctl show -P AllowedCPUs "$slice" || echo "")
done

restore_cpusets() {
  for slice in "${SLICES[@]}"; do
    sudo systemctl set-property --runtime "$slice" "AllowedCPUs=${ORIGINAL_CPUS[$slice]}" || true
  done
}
trap restore_cpusets EXIT

ALL_CPUS="0-$(($(nproc --all) - 1))"
for slice in "${SLICES[@]}"; do
  sudo systemctl set-property --runtime "$slice" "AllowedCPUs=$ALL_CPUS" || true
done

# Widening the cpusets only raises the ceiling. Every process also inherits systemd's
# CPUAffinity=0-2 (configuration.nix), so widen the affinity of the processes doing the work too:
# this script (and with it nixos-rebuild / evaluation) and nix-daemon, which forks the builders.
# No explicit restore needed: once the cpusets are restored the kernel clamps the effective CPUs
# back to 0-2 (effective = affinity ∩ cpuset), and the reboot resets everything anyway.
taskset -a -p -c "$ALL_CPUS" $$ >/dev/null || true
sudo systemctl start nix-daemon.service || true
NIX_DAEMON_PID=$(systemctl show -P MainPID nix-daemon.service)
if [ "$NIX_DAEMON_PID" != "0" ]; then
  sudo taskset -a -p -c "$ALL_CPUS" "$NIX_DAEMON_PID" >/dev/null || true
fi

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
    	--option eval-cache false \
    	-L

sudo reboot
