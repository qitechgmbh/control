{ pkgs ? import <nixpkgs> { } }:

let
  # Use the flake's packages when available
  flake = builtins.getFlake (toString ../.);
  system = builtins.currentSystem;
in {
  server = flake.packages.${system}.server;
  electron = flake.packages.${system}.electron;

  # Use the flake's development shell for consistency
  shell = flake.devShells.${system}.default;
}
