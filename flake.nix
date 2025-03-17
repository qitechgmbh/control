{
  description = "QiTech Industries Control Software";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        # Use Rust beta as required
        rust = pkgs.rust-bin.beta.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
      in {
        packages = {
          server = pkgs.callPackage ./nixos/packages/server.nix { 
            inherit rust;
          };
          
          electron = pkgs.callPackage ./nixos/packages/electron.nix {};
          
          default = self.packages.${system}.server;
        };
      }
    ) // {
      nixosModules.qitech = import ./nixos/modules/qitech.nix;
      nixosModules.default = self.nixosModules.qitech;
    };
}
