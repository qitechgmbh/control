# NixOS installer ISO configuration
# Builds a minimal installer with an embedded install script.
# Usage: nix build .#nixosConfigurations.installer.config.system.build.isoImage
{ config, pkgs, lib, gitInfo, ... }:

let
  installScript = pkgs.writeShellScriptBin "install" ''
    set -euo pipefail

    echo "============================================"
    echo "  QiTech Control NixOS Installer"
    echo "============================================"
    echo ""

    # --- Detect target disk ---
    TARGET_DISK=""
    for disk in /sys/block/sd* /sys/block/nvme* /sys/block/vd*; do
      [ -e "$disk" ] || continue
      removable=$(cat "$disk/removable" 2>/dev/null || echo "1")
      if [ "$removable" = "0" ]; then
        TARGET_DISK="/dev/$(basename "$disk")"
        break
      fi
    done

    if [ -z "$TARGET_DISK" ]; then
      echo "ERROR: No non-removable disk found."
      exit 1
    fi

    # Show disk info
    echo "Detected target disk: $TARGET_DISK"
    lsblk "$TARGET_DISK"
    echo ""
    echo "WARNING: This will ERASE ALL DATA on $TARGET_DISK"
    read -p "Continue? [y/N] " confirm
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
      echo "Aborted."
      exit 1
    fi

    echo ""
    echo ">>> Partitioning $TARGET_DISK ..."

    # Wipe existing partition table
    wipefs -af "$TARGET_DISK"

    # Create GPT partition table with EFI + root
    parted -s "$TARGET_DISK" -- \
      mklabel gpt \
      mkpart ESP fat32 1MiB 512MiB \
      set 1 esp on \
      mkpart root ext4 512MiB 100%

    # Determine partition device names (handles both /dev/sdX and /dev/nvmeXnYpZ)
    if [[ "$TARGET_DISK" == *"nvme"* ]] || [[ "$TARGET_DISK" == *"mmcblk"* ]]; then
      EFI_PART="''${TARGET_DISK}p1"
      ROOT_PART="''${TARGET_DISK}p2"
    else
      EFI_PART="''${TARGET_DISK}1"
      ROOT_PART="''${TARGET_DISK}2"
    fi

    echo ">>> Formatting partitions ..."
    mkfs.fat -F 32 "$EFI_PART"
    mkfs.ext4 -F "$ROOT_PART"

    echo ">>> Mounting ..."
    mount "$ROOT_PART" /mnt
    mkdir -p /mnt/boot
    mount "$EFI_PART" /mnt/boot

    echo ">>> Generating hardware configuration ..."
    nixos-generate-config --root /mnt

    echo ">>> Cloning repository ..."
    REPO_DIR=$(mktemp -d)
    git clone --depth 1 --branch "${gitInfo.gitAbbreviation}" "${gitInfo.gitUrl}" "$REPO_DIR"
    cd "$REPO_DIR"

    echo ">>> Installing NixOS (this will take a while) ..."
    GIT_TIMESTAMP="${gitInfo.gitTimestamp}" \
    GIT_COMMIT="${gitInfo.gitCommit}" \
    GIT_URL="${gitInfo.gitUrl}" \
    GIT_ABBREVIATION="${gitInfo.gitAbbreviation}" \
    GIT_ABBREVIATION_ESCAPED="${gitInfo.gitAbbreviationEscaped}" \
    nixos-install \
      --flake "$REPO_DIR#nixos" \
      --impure \
      --option sandbox false \
      --option eval-cache false \
      --no-root-passwd

    echo ""
    echo "============================================"
    echo "  Installation complete!"
    echo "  Remove the USB drive and reboot."
    echo "============================================"
  '';
in {
  imports = [
    <nixpkgs/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix>
  ];

  # Faster compression for quicker ISO builds
  isoImage.squashfsCompression = "gzip -Xcompression-level 1";

  # Include tools needed by the install script
  environment.systemPackages = [
    installScript
    pkgs.git
    pkgs.parted
  ];

  # Show instructions on login
  users.motd = ''

    ============================================
      QiTech Control NixOS Installer
    ============================================

    Run 'install' to begin installation.

  '';

  # Enable flakes in the installer
  nix.extraOptions = ''
    experimental-features = nix-command flakes
  '';
}
