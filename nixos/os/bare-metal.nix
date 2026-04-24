{ ... }:
{
  # Bootloader
  boot.loader.systemd-boot = {
    enable = true;
    consoleMode = "max"; # Use the highest available resolution
  };
  boot.loader.efi.canTouchEfiVariables = true;

  i18n.defaultLocale = "en_GB.UTF-8";
  i18n.supportedLocales = [ "all" ];

  services.xserver.xkb = {
    layout = "de";
    variant = "";
  };
  console.keyMap = "de";

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
