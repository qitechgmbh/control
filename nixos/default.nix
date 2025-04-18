{ pkgs ? import <nixpkgs> {
    overlays = [
      (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
    ];
  }
}:

let
  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
    targets = [ "x86_64-unknown-linux-gnu" ];
  };
in {
  server = pkgs.callPackage ./nixos/packages/server.nix { inherit rust; };
  electron = pkgs.callPackage ./nixos/packages/electron.nix { };

  # Package for quick development testing without a flake
  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      rust
      pkg-config
      libudev-zero
      libpcap
      nodejs
      nodePackages.npm
    ];
    
    shellHook = ''
      echo "QiTech Industries Control Software Development Environment"
      echo "Rust version: $(${rust}/bin/rustc --version)"
      echo "Node version: $(${pkgs.nodejs}/bin/node --version)"
    '';
  };
}
