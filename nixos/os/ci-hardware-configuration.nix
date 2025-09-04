{ config, pkgs, ... }:

{
  boot.kernelPackages = pkgs.linuxPackages;
  fileSystems."/" = { device = "/dev/null"; fsType = "tmpfs"; };
  swapDevices = [ ];
  boot.initrd.kernelModules = [ ];
}