# QiTech Control - NixOS Quick Start Guide

## Introduction

This guide will help you set up a NixOS system with QiTech Control and manage updates.

## Initial Setup

### 1. Install Git

```bash
nixos-env -i git
```

### 2. Clone the Control Repo

```bash
git clone https://github.com/qitechgmbh/control
cd control
```

### 3. Run Install Script

```bash
# run as admin either with sudo or as root!
sudo ./nixos-install.sh
```

After the Installation script is finished the Computer should have rebooted and automatically started the control software.

### 4. Manual configuration

Disable any power management features or display sleep settings in the GNOME settings.
Check that the on-screen keyboard is enabled under accessibility settings.

## System Configuration Overview

### flake.nix

- Defines inputs (nixpkgs, home-manager, qitech-control, rust-overlay)
- Sets up overlays for Rust and QiTech packages
- Creates NixOS system configuration

### configuration.nix

- Configures system basics (boot, kernel, flakes support)
- Sets up users and groups (qitech user, realtime group)
- Configures real-time operations
- Sets up GNOME desktop environment
- Enables QiTech service with firewall rules
- Installs required packages

### home.nix

- Configures autostart for QiTech electron app
- Customizes GNOME desktop (wallpaper, keyboard, power settings)
- Configures dock with pinned QiTech app
- Sets workspace preferences

## Managing the System

### Updating the System

```bash
# Update all flake inputs
nix flake update

# Update only QiTech Control
nix flake lock --update-input qitech-control

# Rebuild system with updates
sudo nixos-rebuild switch --flake .#nixos
```

### Service Management

```bash
# View service logs
journalctl -u qitech-control-server

# Real-time log viewing
journalctl -u qitech-control-server -f

# Check service status
systemctl status qitech-control-server

# Restart service
sudo systemctl restart qitech-control-server
```

### Application Management

```bash
# Manually start QiTech electron app
qitech-control-electron

# Kill running instances
pkill -f qitech-control-electron
```

### System Rollback

```bash
# Roll back to previous configuration
sudo nixos-rebuild switch --flake .#nixos --rollback

# Or select previous generation from boot menu
```

## Environment Variables

The system uses special environment variables:

- `QITECH_OS`: Set to "true" for NixOS deployments

These are configured in `configuration.nix`, `electron.nix`, and `home.nix`.
