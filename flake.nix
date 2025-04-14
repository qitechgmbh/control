{
  description = "QiTech Industries Control Software";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    
    home-manager = {
      url = "github:nix-community/home-manager/release-24.11";
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
      
      # Define nixosConfigurations outside of eachDefaultSystem
      nixosConfigurations = {
        # Replace "nixos" with your actual hostname
        nixos = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux"; # Specify the correct system
          specialArgs = { inherit qitech-control; }; 
          modules = [
            # Apply the overlay to the system
            { nixpkgs.overlays = [ (import rust-overlay) ]; }
            
            ./nixos/os/configuration.nix
            
            # QiTech Control module
            qitech-control.nixosModules.qitech
            
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
