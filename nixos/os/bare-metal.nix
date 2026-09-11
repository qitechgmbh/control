{ ... }:
{
  # Bootloader
  boot.loader.systemd-boot = {
    enable = true;
    consoleMode = "max"; # Use the highest available resolution
  };
  boot.loader.efi.canTouchEfiVariables = true;
  boot.tmp.cleanOnBoot = true;

  imports = [
    ./configuration.nix
    (
      if builtins.pathExists "/etc/nixos/hardware-configuration.nix" then
        /etc/nixos/hardware-configuration.nix
      else
        ./ci-hardware-configuration.nix
    )
    # Optional per-unit overrides, outside the repo so they survive a git pull or rebuild.
    (if builtins.pathExists "/etc/nixos/qitech-local.nix" then /etc/nixos/qitech-local.nix else { })
  ];
}
