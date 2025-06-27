{ pkgs ? import <nixpkgs> {}
}:

let
  # Use standard Rust 1.86 from nixpkgs
  rust = pkgs.rustc;
in {
  server = pkgs.callPackage ./packages/server.nix { commitHash = "dev"; };
  electron = pkgs.callPackage ./packages/electron.nix { 
    nodejs = pkgs.nodejs_22; 
    commitHash = "dev";
  };

  # Package for quick development testing without a flake
  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      rustc
      cargo
      rust-analyzer
      pkg-config
      libudev-zero
      libpcap
      nodejs_22
      nodePackages.npm
    ];
    
    shellHook = ''
      echo "QiTech Control Development Environment"
      echo "Rust version: $(rustc --version)"
      echo "Node version: $(${pkgs.nodejs_22}/bin/node --version)"
    '';
  };
}
