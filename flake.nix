{
  description = "QiTech Control";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";

    # Crane for Rust builds with dependency caching
    crane = {
      url = "github:ipetkov/crane";
    };

    home-manager = {
      url = "github:nix-community/home-manager/release-25.11";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      crane,
      home-manager,
      ...
    }:
    let
      pkgs' = system: import nixpkgs { inherit system; };
      forSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      formatter = forSystems (system: inputs.nixpkgs.legacyPackages.${system}.nixfmt-tree);

      overlays.default =
        # Add our own overlay for QiTech packages with commit hash support
        (
          final: prev: {
            qitechPackages = {
              server = prev.callPackage ./nixos/packages/server.nix {
                craneLib = crane.mkLib final;
              };
              electron = prev.callPackage ./nixos/packages/electron.nix { };
              gitInfo = prev.callPackage ./nixos/gitInfo.nix { pkgs = prev; };
            };
          }
        );

      packages = forSystems (
        system: with (pkgs' system); rec {
          default = server;
          server = callPackage ./nixos/packages/server.nix {
            craneLib = crane.mkLib (pkgs' system);
          };
          electron = callPackage ./nixos/packages/electron.nix { };
          iso-x86_64 =
            (nixpkgs.lib.nixosSystem {
              system = "x86_64-linux";
              specialArgs = {
                inherit inputs;
              };
              modules = [
                # QiTech Control module
                self.nixosModules.qitech
                # Home Manager module
                home-manager.nixosModules.home-manager
                # NixOS configuration
                ./nixos/os/iso.nix
                {
                  nixpkgs.overlays = [
                    # Add our own overlay for QiTech packages with commit hash support
                    self.overlays.default
                  ];
                }
              ];
            }).config.system.build.isoImage;

        }
      );

      devShells = forSystems (system: {
        default =
          with (pkgs' system);
          mkShell {
            packages = with pkgs; [
              cargo
              rustc
              pkg-config
              libudev-zero
              libpcap
              nodejs_22
              nodePackages.npm
              lldb
              electron
              nixfmt
              nixd
            ];

            ELECTRON_SKIP_BINARY_DOWNLOAD = 1;
            # TODO: There's probably a better solution
            ELECTRON_OVERRIDE_DIST_PATH = "${electron}/bin/";

            hardeningDisable = [ "fortify" ];

            shellHook = ''
              echo "QiTech Control Development Environment"
              echo "Rust version: $(rustc --version)"
              echo "Node version: $(${pkgs.nodejs_22}/bin/node --version)"
            '';
          };
      });

      nixosModules.qitech = import ./nixos/modules/qitech.nix;
      nixosModules.default = self.nixosModules.qitech;

      nixosConfigurations.nixos = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        specialArgs = {
          inherit inputs;
        };
        modules = [
          # QiTech Control module
          self.nixosModules.qitech
          # Home Manager module
          home-manager.nixosModules.home-manager
          # NixOS configuration
          ./nixos/os/bare-metal.nix
          {
            nixpkgs.overlays = [
              # Add our own overlay for QiTech packages with commit hash support
              self.overlays.default
            ];
          }
        ];
      };
    };
}
