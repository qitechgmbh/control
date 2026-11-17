{
  description = "QiTech Control";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";

    home-manager = {
      url = "github:nix-community/home-manager/release-26.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      home-manager,
      ...
    }:
    let
      pkgs' = system: import nixpkgs { inherit system; };
      forSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
    in
    {
      formatter = forSystems (system: inputs.nixpkgs.legacyPackages.${system}.nixfmt-tree);

      overlays.default =
        # Add our own overlay for QiTech packages with commit hash support
        (
          final: prev: {
            qitechPackages = {
              server = prev.callPackage ./nixos/packages/server.nix { };
              electron = prev.callPackage ./nixos/packages/electron.nix { };
              gitInfo = prev.callPackage ./nixos/gitInfo.nix { pkgs = prev; };
            };

            # IgH EtherCAT master pinned to stable-1.6 with the kernel_bind fix for
            # Linux >= 6.18.56 (sockaddr_unsized). Also used by ethercat.kernelModule.
            # Drop once a release containing this commit lands in nixpkgs.
            ethercat = prev.ethercat.overrideAttrs (old: {
              version = "1.6.13-unstable-2026-11-12";
              src = prev.fetchFromGitLab {
                owner = "etherlab.org";
                repo = "ethercat";
                rev = "6cbfb0f1bfb70b28e9c10c8cdf7e96eb82745a67";
                hash = "sha256-UCw2LsaCXQ6F/aseXeTPvBSmuFqyZbctYCtk/7QFOhg=";
              };
            });
          }
        );

      packages = forSystems (
        system: with (pkgs' system); rec {
          default = server;
          server = callPackage ./nixos/packages/server.nix { };
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
              nodejs_24
              lldb
              electron
              # dev tools
              nixfmt
              nixd
              rustfmt
              rust-analyzer
              clippy
            ];

            ELECTRON_SKIP_BINARY_DOWNLOAD = 1;
            # TODO: There's probably a better solution
            ELECTRON_OVERRIDE_DIST_PATH = "${electron}/bin/";

            hardeningDisable = [ "fortify" ];

            shellHook = ''
              echo "QiTech Control Development Environment"
              echo "Rust version: $(rustc --version)"
              echo "Node version: $(node --version)"
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
