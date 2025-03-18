{
  description = "NixOS configuration with Home Manager";

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
  };

  outputs = { self, nixpkgs, home-manager, qitech-control, rust-overlay, ... }: 
    let
      system = "x86_64-linux";
      
      # Create overlays including qitech packages
      overlays = [
        rust-overlay.overlays.default
        # Add an overlay to make qitech packages available
        (final: prev: {
          qitech-control-server = qitech-control.packages.${system}.server;
          qitech-control-electron = qitech-control.packages.${system}.electron;
        })
      ];
      
      pkgs = import nixpkgs { 
        inherit system overlays; 
      };
    in {
      nixosConfigurations.nixos = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit qitech-control pkgs; }; 
        modules = [
          # Apply the overlay to the system
          { nixpkgs.overlays = overlays; }
          
          ./configuration.nix
          
          # QiTech Control module
          qitech-control.nixosModules.qitech
          
          # Home Manager module
          home-manager.nixosModules.home-manager
          {
            home-manager.useGlobalPkgs = true;
            home-manager.useUserPackages = true;
            home-manager.users.qitech = import ./home.nix;
          }
        ];
      };
    };
}
