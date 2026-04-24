{ ... }:
{
  # Bootloader
  boot.loader.systemd-boot = {
    enable = true;
    consoleMode = "max"; # Use the highest available resolution
  };
  boot.loader.efi.canTouchEfiVariables = true;

  i18n.supportedLocales = [ "all" ];

  imports = [
    ./configuration.nix
    (
      if builtins.pathExists "/etc/nixos/hardware-configuration.nix" then
        /etc/nixos/hardware-configuration.nix
      else
        ./ci-hardware-configuration.nix
    )
  ];
}
