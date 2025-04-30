# QiTech Industries Control Software - NixOS Quick Start Guide

## Introduction

This guide will help you set up a NixOS system with QiTech Industries Control Software and manage updates.

## Initial Setup

### 1. Add Home Manager Channel

```bash
sudo nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
sudo nix-channel --update
```

### 2. Configure System Files

Create the three main configuration files:

- `flake.nix` - Defines inputs, overlays, and system configuration
- `configuration.nix` - Main NixOS system configuration
- `home.nix` - Home Manager configuration for the qitech user

### 3. Build and Apply Configuration

```bash
# Build and switch to the new configuration
sudo nixos-rebuild switch --flake .#nixos
```

### 4. Manual configuration

Disable any power management features or display sleep settings in the GNOME settings. Check that the on-screen keyboard is enabled under accessibility settings.

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
