# NixOS Docs

Nixos is a Linux distro tightly integrated with the Nix package manager. It is known for its declarative configuration, reproducibility, and powerful package management capabilities.

This is why we chose nixos
- enables us to ship OS components like drivers to the customer's computer
- reproducible
- able to roll back

# How we do updates
1. The user selects a version (commit/branch/tag) of the software they want to install
2. The repo is cloned to the local machine
3. Information like the commit hash, branch, tag, timestamp etc. is passed to the nix build process as environment variables `QITECH_OS...`
4. The `nixos-switch` command is run. After rebuilding & compiling the software the system reboots.
   - The nixos-switch command does the following:
   1. The system is built with the new configuration
   2. The system is switched to the new configuration
   3. The system is marked as "installed" with the new version
   4. The system is set to boot into the new version
5. The computer automatically reboots into the new version of the OS + software.

# More Docs
- [Implementation Details](./details.md)
- [Fresh Installation Quick Start guide](./quick-start.md)
