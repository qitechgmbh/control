# copy /etc/nixos/hardware-configuration.nix to ./nixos/os/hardware-configuration.nix
# -f flag forces overwriting if file already exists
cp -f /etc/nixos/hardware-configuration.nix ./nixos/os/hardware-configuration.nix 

sudo nixos-rebuild switch --flake .#nixos --show-trace