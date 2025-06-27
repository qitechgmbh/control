{
  description = "QiTech Control";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    
    # Crane for Rust builds with dependency caching
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    home-manager = {
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # Add the QiTech Control repository
    qitech-control = {
      url = "github:qitechgmbh/control";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # Add flake-utils which was missing
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, qitech-control, home-manager, ... }:
    let
      # Import git info at the top level so it's available everywhere
      installInfo = import ./nixos/os/installInfo.nix;
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ 
          # Add our own overlay for QiTech packages
          (final: prev: {
            qitechPackages = {
              server = final.callPackage ./nixos/packages/server.nix { 
                commitHash = builtins.getEnv "QITECH_COMMIT_HASH";
                inherit crane;
              };
              electron = final.callPackage ./nixos/packages/electron.nix { 
                commitHash = builtins.getEnv "QITECH_COMMIT_HASH";
                nodejs = final.nodejs_22;
              };
            };
          })
        ];
        pkgs = import nixpkgs { inherit system overlays; };

        craneLib = crane.mkLib pkgs;
        
        # Use Rust 1.86 stable from nixpkgs
        rust = pkgs.rustc;
      in {
        packages = {
          server = pkgs.qitechPackages.server;
          electron = pkgs.qitechPackages.electron;
          default = self.packages.${system}.server;
        };
        
        devShells.default = craneLib.devShell {
          packages = with pkgs; [
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
    ) // {
      nixosModules.qitech = import ./nixos/modules/qitech.nix;
      nixosModules.default = self.nixosModules.qitech;
      
      # Define nixosConfigurations outside of eachDefaultSystem
      nixosConfigurations = {
        # Replace "nixos" with your actual hostname
        nixos = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux"; # Specify the correct system
          specialArgs = {
            installInfo = installInfo; # Pass installInfo to modules
          };
          modules = [
            # Apply the overlays to the system
            { nixpkgs.overlays = [
                # Add our own overlay for QiTech packages with commit hash support
                (final: prev: {
                  qitechPackages = {
                    server = final.callPackage ./nixos/packages/server.nix { 
                      commitHash = installInfo.gitCommit;
                      inherit crane;
                    };
                    electron = final.callPackage ./nixos/packages/electron.nix {
                      commitHash = installInfo.gitCommit;
                      nodejs = final.nodejs_22;
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