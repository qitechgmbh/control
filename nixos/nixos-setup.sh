#!/usr/bin/env bash

set -euo pipefail
shopt -s nullglob

function yes_or_no {
	while true; do
		read -p "$* [y/n]: " yn
		case $yn in
			[Yy]*) return 0 ;;
			[Nn]*) return 1 ;;
		esac
	done
}

# Gain root permissions
[[ "$EUID" == 0 ]] || exec sudo -s "$0" "$@"

echo "============================================"
echo "  QiTech Control NixOS Installer"
echo "============================================"
echo

# Fail if dependencies are missing
ERR=0
for app in blkid git lsblk mkdir mkfs.vfat mkfs.ext4 nixos-install parted partprobe udevadm wipefs
do
	if ! which $app &> /dev/null; then
		ERR=$((ERR + 1))
		echo "Error: $app is missing"
	fi
done
if [[ $ERR -gt 0 ]]; then exit 1; fi

if ! ping -c1 github.com &> /dev/null; then
	echo "Error: No connection to the internet"
	exit 1
fi

echo
echo "List of block devices"
lsblk -o name,rm,size,type,mountpoints
echo

PS3="Select installation target drive: "
TARGET_DISK=""

if ! ls /sys/block/{sd?,nvme?n?,mmcblk?,vd?} &> /dev/null; then
	echo "Error: no usable target drives"
	exit 1
fi

select disk in $(basename -a /sys/block/{sd?,nvme?n?,mmcblk?,vd?})
do
	echo "WARNING: ALL DATA ON /dev/$disk WILL BE ERASED!!!"
	if yes_or_no "Continue?"; then
		TARGET_DISK="/dev/$disk"
		break
	fi
done

# Make the kernel pick up the partition table we just wrote and wait until udev
# has (re)created the partition nodes. Without this the nodes may still carry the
# offsets of the *previous* table, so mkfs writes the superblock to a location
# the later mount no longer reads from ("bad superblock" / mount exit 32).
function settle_partitions {
	sync
	partprobe "$TARGET_DISK" &> /dev/null || true
	udevadm settle --timeout=30 &> /dev/null || true
}

# Wait for a device node to show up after a partition table change
function wait_for_device {
	local dev="$1"
	for _ in $(seq 1 30); do
		[[ -b "$dev" ]] && return 0
		sleep 1
		udevadm settle --timeout=1 &> /dev/null || true
	done
	return 1
}

echo ">>> Partitioning $TARGET_DISK ..."

# Unmount conflicting remnant mounts
swapoff -a &> /dev/null || true
umount -R /mnt &> /dev/null || true
umount "${TARGET_DISK}"* &> /dev/null || true

# Remove stale filesystem/partition signatures so nothing from a previous
# install can be probed later on. Only touch real partitions of the target,
# never sibling devices like /dev/mmcblk0boot0.
while read -r part; do
	[[ -n "$part" && "$part" != "$TARGET_DISK" ]] || continue
	wipefs -a "$part" &> /dev/null || true
done < <(lsblk -nrpo name "$TARGET_DISK")
wipefs -a "$TARGET_DISK" &> /dev/null || true
settle_partitions

parted -s "$TARGET_DISK" -- \
	mklabel gpt \
	mkpart ESP fat32 1MiB 512MiB \
	set 1 esp on \
	mkpart root ext4 512MiB 100%

settle_partitions

# Devices whose name ends in a digit (/dev/mmcblk0, /dev/nvme0n1) name their
# partitions "p1"/"p2", plain disks (/dev/sda, /dev/vda) name them "1"/"2".
if [[ "$TARGET_DISK" =~ [0-9]$ ]]; then
	PART_PREFIX="${TARGET_DISK}p"
else
	PART_PREFIX="$TARGET_DISK"
fi
EFI_PART="${PART_PREFIX}1"
ROOT_PART="${PART_PREFIX}2"

for part in "$EFI_PART" "$ROOT_PART"; do
	if ! wait_for_device "$part"; then
		echo "Error: partition $part did not appear after partitioning"
		lsblk -o name,size,type "$TARGET_DISK"
		exit 1
	fi
done

echo ">>> Formatting partitions ..."
mkfs.fat -F 32 "$EFI_PART"
mkfs.ext4 -F "$ROOT_PART"
settle_partitions

# Fail here with a clear message instead of a cryptic mount error
for part in "$EFI_PART" "$ROOT_PART"; do
	if ! blkid "$part" &> /dev/null; then
		echo "Error: no filesystem detected on $part after formatting"
		exit 1
	fi
done

echo ">>> Mounting ..."
mount "$ROOT_PART" /mnt
mkdir -p /mnt/boot
mount "$EFI_PART" /mnt/boot

echo ">>> Generating hardware configuration ..."
nixos-generate-config --root /mnt

# Symlink this to avoid building the (invalid) iso config
if [[ -e /etc/nixos/hardware-configuration.nix ]]; then
	echo "Warning: found a unrelated /etc/nixos/hardware-configuration.nix"
	echo "Backing up to /etc/nixos/hardware-configuration.nix.bak"
	cp --backup=t -f /etc/nixos/hardware-configuration.nix /etc/nixos/hardware-configuration.nix.bak
fi
ln -sf /mnt/etc/nixos/hardware-configuration.nix /etc/nixos/hardware-configuration.nix

echo ">>> Cloning repository ..."
# Use remote URL and current branch if we are in the repo, hardcoded defaults otherwise
export GIT_URL=$(git config --get remote.origin.url 2> /dev/null || echo "https://github.com/qitechgmbh/control.git")
export GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2> /dev/null || echo "master")

REPO_DIR=$(mktemp -d)
git clone --depth 1 --branch "$GIT_BRANCH" "$GIT_URL" "$REPO_DIR"
cd "$REPO_DIR"

# Capture all git information
export GIT_TIMESTAMP=$(git --no-pager show -s --format=%cI HEAD) # e.g., "2025-06-10T14:30:45+02:00"
export GIT_COMMIT=$(git rev-parse HEAD) # e.g., "b2c7f6e0b138174770798f84ada8b0aa65afeb"
export GIT_TAG=$(git describe --tags --exact-match HEAD 2>/dev/null || echo "")
if [ -n "$GIT_TAG" ]; then
	export GIT_ABBREVIATION="$GIT_TAG" # e.g., "2.0.0" (when on a tag)
else
	if [ "$GIT_BRANCH" = "HEAD" ]; then
		export GIT_ABBREVIATION=$(git rev-parse --short HEAD) # e.g., "b2c7f6e" (when on detached HEAD/commit)
	else
		export GIT_ABBREVIATION="$GIT_BRANCH" # e.g., "main", "develop" (when on a branch)
	fi
fi
export GIT_ABBREVIATION_ESCAPED=$(echo "$GIT_ABBREVIATION" | sed -e 's/+/-/g' -e 's/[^a-zA-Z0-9:_\.-]//g') # e.g., "2-0-0", "main", "b2c7f6e"

echo ">>> Installing NixOS (this will take a while) ..."
nixos-install \
	--flake .#nixos \
	--show-trace \
	--impure \
	--no-root-passwd

echo
echo "============================================"
echo "  Installation complete!"
echo "  Remove the USB drive and reboot."
echo "============================================"

# Revoke cached sudo permission
sudo -k
