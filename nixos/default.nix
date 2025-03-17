{ pkgs ? import <nixpkgs> {
    overlays = [
      (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
    ];
  }
}:

{
  packages = {
    server = pkgs.callPackage ./packages/server.nix {};
    electron = pkgs.callPackage ./packages/electron.nix {};
  };

  nixosModules.qitech = import ./modules/qitech.nix;
}
