{
  description = "QiTech Industries Control Software";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # Add the QiTech Control repository
    qitech-control = {
      url = "github:qitechgmbh/control";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # Add the Rust overlay as an input
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # Add flake-utils which was missing
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, qitech-control, home-manager, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ 
          (import rust-overlay)
          # Add our own overlay for QiTech packages
          (final: prev: {
            qitechPackages = {
              server = final.callPackage ./nixos/packages/server.nix { 
                rust-bin = final.rust-bin;
                commitHash = builtins.getEnv "QITECH_COMMIT_HASH";
              };
              electron = final.callPackage ./nixos/packages/electron.nix { 
                commitHash = builtins.getEnv "QITECH_COMMIT_HASH";
              };
            };
          })
        ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Use Rust nightly for edition 2024 support
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
      in {
        packages = {
          server = pkgs.qitechPackages.server;
          electron = pkgs.qitechPackages.electron;
          default = self.packages.${system}.server;
        };
        
        devShells.default = pkgs.mkShell {
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
    ) // {
      nixosModules.qitech = import ./nixos/modules/qitech.nix;
      nixosModules.default = self.nixosModules.qitech;
      
      # Define nixosConfigurations outside of eachDefaultSystem
      nixosConfigurations = {
        # Replace "nixos" with your actual hostname
        nixos = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux"; # Specify the correct system
          specialArgs = {
            gitInfo = import ./nixos/os/git.nix;
          };
          modules = [
            # Apply the overlays to the system
            { nixpkgs.overlays = [
                (import rust-overlay)
                # Add our own overlay for QiTech packages with commit hash support
                (final: prev: {
                  qitechPackages = {
                    server = final.callPackage ./nixos/packages/server.nix { 
                      rust-bin = final.rust-bin;
                      # Get commit hash from environment or use default
                      commitHash = builtins.getEnv "QITECH_COMMIT_HASH";
                    };
                    electron = final.callPackage ./nixos/packages/electron.nix {
                      # Get commit hash from environment or use default
                      commitHash = builtins.getEnv "QITECH_COMMIT_HASH";
                    };
                  };
                })
              ];
            }
            
            ./nixos/os/configuration.nix
            
            # QiTech Control module
            self.nixosModules.qitech
            
            # Home Manager module
            home-manager.nixosModules.home-manager
            {
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.users.qitech = import ./nixos/os/home.nix;
            }
          ];
        };
      };
    };
}
