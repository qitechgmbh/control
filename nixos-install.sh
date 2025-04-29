# write the git.nix while
# we capture information about the current git commit
sudo touch ./nixos/os/git.nix
sudo sh -c 'echo "{
  timestamp = \"$(git --no-pager show -s --format=%cI HEAD)\";
  commit = \"$(git rev-parse HEAD)\";
  abbrevation = \"$(git rev-parse --abbrev-ref HEAD)\";
  url = \"$(git config --get remote.origin.url)\";
}" > ./nixos/os/git.nix'

# Now we install the new system
sudo nixos-rebuild switch --flake .#nixos --show-trace --impure --option eval-cache false