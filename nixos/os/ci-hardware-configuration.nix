{ config, pkgs, ... }:

{
  fileSystems."/" = {
    device = "/dev/null";
    fsType = "tmpfs";
  };
  swapDevices = [ ];
  boot.initrd.kernelModules = [ ];
}
