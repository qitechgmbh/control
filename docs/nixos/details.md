# QiTech Control

## Nix Configuration

This document explains the Nix configuration for the QiTech Control, covering how packages are built, how to use the NixOS module, and local development workflows.

### Project Structure

The project uses Nix flakes for reproducible builds and includes configurations for both local development and NixOS system integration:

- `flake.nix` - Main flake definition with package outputs and NixOS module
- `nixos/default.nix` - Legacy non-flake interface and development shell
- `nixos/packages/server.nix` - Server component package definition
- `nixos/packages/electron.nix` - Electron frontend package definition
- `nixos/modules/qitech.nix` - NixOS module for system integration

### Package Descriptions

#### flake.nix

The `flake.nix` file defines the main entry point for the Nix flake system, providing:

- Package outputs for both the server and electron frontend
- Use of the Rust beta channel with specific extensions and targets
- NixOS module export for system-wide integration

```bash
## Build the server component
nix build .#server

## Build the electron frontend
nix build .#electron

## Build the default package (server)
nix build
```

#### nixos/default.nix

This file provides a non-flake way to use the packages and defines a development shell:

```bash
## Start development shell without using flakes
nix-shell ./nixos

## Build server without flakes
nix-build ./nixos -A server

## Build electron without flakes
nix-build ./nixos -A electron
```

#### nixos/packages/server.nix

This defines how to build the server component:

- Uses Rust beta toolchain
- Requires libpcap and libudev for hardware access
- Includes memory optimization for constrained build environments
- Creates dynamically linked binary with runtime dependencies

#### nixos/packages/electron.nix

This defines how to build the Electron frontend:

- Builds and packages the Electron application with npm
- Configures desktop integration (icons, application menu entries)
- Handles proper security and sandbox settings
- Creates a wrapper script for launching the application

#### nixos/modules/qitech.nix

NixOS module for system-wide integration:

- Configures the systemd service for the server
- Sets up a dedicated user/group with appropriate permissions
- Configures udev rules for hardware access
- Sets up real-time privileges for low-latency operation
- Provides firewall configuration options
- Installs desktop integration for the Electron app

### Development Workflow

#### Local Development

For local development without installing system-wide:

```bash
## Enter development shell with all dependencies
nix develop
## OR for non-flake users
nix-shell ./nixos

## Run the server in dev mode
cd server
cargo run

## Run the electron frontend in dev mode
cd electron
npm run dev
```

#### Building Packages

```bash
## Build all packages
nix flake check

## Test build just the server
nix build .#server

## Test build just the electron frontend
nix build .#electron

## Run the built server
./result/bin/qitech-control-server

## Run the built electron app
./result/bin/qitech-control-electron
```

## NixOS System Configuration

This documentation covers the NixOS system configuration for QiTech Control, including system integration, user environment, and system management procedures.

### Configuration Files Overview

The NixOS system is configured through three main files:

- `flake.nix` - The entry point that defines inputs, overlays, and system configuration
- `configuration.nix` - The main NixOS system configuration
- `home.nix` - The Home Manager configuration for the qitech user

After initial setup, it is necessary to add Home Manager to the system using:

```bash
sudo nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager

sudo nix-channel --update
```

### File Details

#### flake.nix

This file defines the Nix flake structure and imports for the QiTech Control system:

- **Inputs**:
  - `nixpkgs` - The NixOS package repository
  - `home-manager` - For managing user environment configuration
  - `qitech-control` - The QiTech Control software repository
  - `rust-overlay` - For Rust toolchain management

- **Overlays**:
  - Makes Rust toolchain available system-wide
  - Brings QiTech packages into the system package set

- **System Configuration**:
  - Creates a complete NixOS system named "nixos"
  - Incorporates QiTech modules and Home Manager
  - Applies overlays to make packages available

```bash
## Update all flake inputs to latest versions
nix flake update

## Update only the qitech-control input
nix flake lock --update-input qitech-control
```

#### configuration.nix

The main NixOS system configuration file:

- **System Basics**:
  - Uses systemd-boot and EFI
  - Configures the latest Linux kernel
  - Enables flakes support

- **User and Group Management**:
  - Creates a `realtime` group for low-latency operations
  - Configures the `qitech` user with appropriate permissions
  - Sets up the `qitech-service` user/group for the service

- **Real-time Configuration**:
  - Sets process priority, memory locking, and nice levels for real-time users
  - Disables sleep/suspend/hibernate

- **Desktop Environment**:
  - Configures GNOME desktop with automatic login
  - Removes unnecessary GNOME applications
  - Adds required extensions and tools

- **QiTech Control Integration**:
  - Enables the QiTech service
  - Configures firewall and networking
  - Sets the service to run on port 3001

- **Package Management**:
  - Installs the QiTech electron app system-wide
  - Adds required tools like git and GNOME extensions

#### home.nix

The Home Manager configuration for the `qitech` user:

- **Autostart Configuration**:
  - Adds an autostart entry for the QiTech electron app

- **GNOME Desktop Customization**:
  - Sets custom QiTech wallpaper
  - Enables on-screen keyboard
  - Disables screen blanking, timeout, and locking
  - Configures power settings to prevent sleep

- **Dock Configuration**:
  - Configures the GNOME dash-to-dock extension
  - Sets the dock to be visible on all monitors
  - Pins the QiTech application to the dock

- **Workspace Settings**:
  - Configures display and workspace behavior
  - Sets favorite applications

### System Management

#### Updating and Rebuilding the System

To apply changes to your system configuration:

```bash
## Rebuild the system and switch to the new configuration
sudo nixos-rebuild switch --flake .#nixos

## Test a configuration without applying it
sudo nixos-rebuild test --flake .#nixos

## Build but don't activate (creates result symlink)
sudo nixos-rebuild build --flake .#nixos
```

#### Updating QiTech Software

When a new version of QiTech Control is available:

```bash
## Update the QiTech Control flake input
nix flake lock --update-input qitech-control

## Rebuild the system with the updated package
sudo nixos-rebuild switch --flake .#nixos
```

#### Viewing Logs and Troubleshooting

To check the QiTech service logs:

```bash
## View service logs
journalctl -u qitech-control-server

## View logs in real-time
journalctl -u qitech-control-server -f

## Check service status
systemctl status qitech-control-server
```

### Network and Hardware Integration

The QiTech Control software requires specific hardware access:

- **Network Configuration**:
  - The service runs on port 3001 (configured in `configuration.nix`)
  - The firewall is configured to allow access to this port

- **Hardware Access**:
  - Real-time privileges are granted for low-latency operations
  - The `qitech-service` user has permissions for hardware access
  - The `qitech` user belongs to the `realtime` group

### Desktop Integration

The QiTech Electron application is:

- Installed system-wide via the overlay in `flake.nix`
- Set to auto-start via the `home.nix` configuration
- Pinned to the dock for easy access
- Configured with a custom desktop entry and icon

### Environment Variables

The QiTech Control software on NixOS systems includes special environment variables that allow the application to detect it's running in a NixOS environment. These variables can be used in the Electron/React code to enable NixOS-specific features or behaviors.

- **QITECH_OS**: Set to "true" to identify NixOS deployments
- **QITECH_OS_GIT_TIMESTAMP**: Contains an ISO timestamp of the commit the system was built from
- **QITECH_OS_GIT_COMMIT**: Contains the commit hash of the system build
- **QITECH_OS_GIT_ABBREVIATION**: Contains the branch/tag/commit of the system build
- **QITECH_OS_GIT_URL**: Contains the URL of the repository the system was built from

### Common Operations

#### Restarting the QiTech Service

```bash
## Restart the QiTech server
sudo systemctl restart qitech-control-server

## Check if it's running properly
sudo systemctl status qitech-control-server
```

#### Managing the QiTech Application

```bash
## Manually start the QiTech electron app
qitech-control-electron

## Kill any running instances (if needed)
pkill -f qitech-control-electron
```

#### System Rollback

If a configuration causes issues, you can roll back:

```bash
## Boot into previous generation
## Select previous generation from boot menu
## Or roll back without rebooting
sudo nixos-rebuild switch --flake .#nixos --rollback
```
