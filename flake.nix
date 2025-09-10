{
  description = "QiTech Control";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";

    home-manager = {
      url = "github:nix-community/home-manager";
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

  outputs = { self, nixpkgs, flake-utils, qitech-control, home-manager, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          # Add our own overlay for QiTech packages
          (final: prev: {
            qitechPackages = {
              server = final.callPackage ./nixos/packages/server.nix {};
              electron = final.callPackage ./nixos/packages/electron.nix {
                nodejs = final.nodejs_22;
              };
            };
          })
        ];

        pkgs = import nixpkgs { inherit system overlays; };
        gitInfo = import ./nixos/gitInfo.nix { inherit pkgs; };
        # Use Rust 1.86 stable from nixpkgs
        rust = pkgs.rustc;
      in {
        packages = {
          server = pkgs.qitechPackages.server;
          electron = pkgs.qitechPackages.electron;
          default = self.packages.${system}.server;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            pkg-config
            rustc
            cargo
            libudev-zero
            libpcap
            nodejs_22
            nodePackages.npm
          ];

          hardeningDisable = [ "fortify" ];

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
      nixosConfigurations =
      let
        system = builtins.currentSystem;
        pkgs = import nixpkgs { inherit system; };
        gitInfo = import ./nixos/gitInfo.nix { inherit pkgs; };
      in {
        # Replace "nixos" with your actual hostname
        nixos = nixpkgs.lib.nixosSystem {
          system = system;
          specialArgs = {
            gitInfo = gitInfo; # Pass gitInfo to modules
          };
          modules = [
            # Apply the overlays to the system
            { nixpkgs.overlays = [
                # Add our own overlay for QiTech packages with commit hash support
                (final: prev: {
                  qitechPackages = {
                    server = final.callPackage ./nixos/packages/server.nix {};
                    electron = final.callPackage ./nixos/packages/electron.nix {
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
