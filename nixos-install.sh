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
# A failing step is logged but never aborts the update: the build still works, just on fewer cores.
cpu_log()   { echo "[cpu-unlock] $*"; }
cpu_error() { echo "[cpu-unlock] ERROR: $*" >&2; }
slice_effective() { cat "/sys/fs/cgroup/$1/cpuset.cpus.effective" 2>/dev/null || echo "?"; }
pid_affinity()    { awk '/^Cpus_allowed_list/ {print $2}' "/proc/$1/status" 2>/dev/null || echo "?"; }

SLICES=(system.slice user.slice)
ALL_CPUS="0-$(($(nproc --all) - 1))"
declare -A ORIGINAL_CPUS

cpu_log "Kernel cmdline: $(cat /proc/cmdline)"
cpu_log "Target CPUs: $ALL_CPUS"
cpu_log "Script PID $$, cgroup $(cut -d: -f3 /proc/$$/cgroup), affinity $(pid_affinity $$)"

# Step 1: remember the configured cpusets so they can be restored
for slice in "${SLICES[@]}"; do
  if ORIGINAL_CPUS[$slice]=$(systemctl show -P AllowedCPUs "$slice"); then
    cpu_log "Step 1: $slice AllowedCPUs='${ORIGINAL_CPUS[$slice]}', effective=$(slice_effective "$slice")"
  else
    ORIGINAL_CPUS[$slice]=""
    cpu_error "Step 1: could not read AllowedCPUs of $slice, will restore to '' (unrestricted until reboot)"
  fi
done

restore_cpusets() {
  local rc=$?
  cpu_log "Restore: script exiting with code $rc, re-locking cpusets"
  for slice in "${SLICES[@]}"; do
    if sudo systemctl set-property --runtime "$slice" "AllowedCPUs=${ORIGINAL_CPUS[$slice]}"; then
      cpu_log "Restore: $slice AllowedCPUs='${ORIGINAL_CPUS[$slice]}', effective=$(slice_effective "$slice")"
    else
      cpu_error "Restore: failed to reset $slice, it stays unlocked until the next reboot"
    fi
  done
}
trap restore_cpusets EXIT

# Step 2: widen the cpusets of system.slice (nix-daemon) and user.slice (Electron -> this script)
for slice in "${SLICES[@]}"; do
  if sudo systemctl set-property --runtime "$slice" "AllowedCPUs=$ALL_CPUS"; then
    effective=$(slice_effective "$slice")
    cpu_log "Step 2: $slice AllowedCPUs='$(systemctl show -P AllowedCPUs "$slice")', effective=$effective"
    [ "$effective" = "$ALL_CPUS" ] || cpu_error "Step 2: $slice effective cpuset is '$effective', expected '$ALL_CPUS'"
  else
    cpu_error "Step 2: systemctl set-property on $slice failed"
  fi
done
script_cgroup="$(cut -d: -f3 /proc/$$/cgroup)"
cpu_log "Step 2: own cgroup $script_cgroup effective=$(slice_effective "$script_cgroup")"

# Widening the cpusets only raises the ceiling. Every process also inherits systemd's
# CPUAffinity=0-2 (configuration.nix), so widen the affinity of the processes doing the work too:
# this script (and with it nixos-rebuild / evaluation) and nix-daemon, which forks the builders.
# No explicit restore needed: once the cpusets are restored the kernel clamps the effective CPUs
# back to 0-2 (effective = affinity ∩ cpuset), and the reboot resets everything anyway.

# Step 3: widen this script's affinity (inherited by sudo nixos-rebuild and local builds)
if taskset -a -p -c "$ALL_CPUS" $$ >/dev/null; then
  affinity=$(pid_affinity $$)
  cpu_log "Step 3: script affinity is now $affinity"
  [ "$affinity" = "$ALL_CPUS" ] || cpu_error "Step 3: script affinity is '$affinity', expected '$ALL_CPUS'"
else
  cpu_error "Step 3: taskset on script PID $$ failed"
fi

# Step 4: widen nix-daemon's affinity (inherited by daemon workers and the builders they fork)
if ! sudo systemctl start nix-daemon.service; then
  cpu_error "Step 4: could not start nix-daemon.service"
fi
NIX_DAEMON_PID=$(systemctl show -P MainPID nix-daemon.service || echo 0)
if [ "$NIX_DAEMON_PID" = "0" ]; then
  cpu_error "Step 4: nix-daemon has no main PID, skipping"
elif sudo taskset -a -p -c "$ALL_CPUS" "$NIX_DAEMON_PID" >/dev/null; then
  affinity=$(pid_affinity "$NIX_DAEMON_PID")
  cpu_log "Step 4: nix-daemon PID $NIX_DAEMON_PID affinity is now $affinity"
  [ "$affinity" = "$ALL_CPUS" ] || cpu_error "Step 4: nix-daemon affinity is '$affinity', expected '$ALL_CPUS'"
else
  cpu_error "Step 4: taskset on nix-daemon PID $NIX_DAEMON_PID failed"
fi

cpu_log "Unlock done, starting nixos-rebuild"

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
