# write the git.nix while
# we capture information about the current git commit
sudo touch ./nixos/os/git.nix
sudo sh -c 'echo "{
  timestamp = \"$(date -Iseconds)\";
  commit = \"$(git rev-parse HEAD)\";
  abbreviation = \"$(git rev-parse --abbrev-ref HEAD)\";
  url = \"$(git config --get remote.origin.url)\";
  # Like abbreviation but can be used in system.nixos.label
  abbreviationEscaped = \"$(git rev-parse --abbrev-ref HEAD | sed -e "s/+/-/g" -e "s/[^a-zA-Z0-9:_\.-]//g")\";
  # Like timestamp but can be used in system.nixos.label
  currentTimestampEscaped = \"$(date -Iseconds | sed -e "s/+/-/g" -e "s/[^a-zA-Z0-9:_\.-]//g")\";
}" > ./nixos/os/git.nix'

# make sure the git.nix file is tracked by git
# configure a git user
git config --global user.name "nixos-install.sh"
git config --global user.email "nixos-install.sh@localhost"
# add the git.nix file to the git index
git add ./nixos/os/git.nix
# commit the changes
git commit -m "Add git.nix file with current commit information"
# Now we can install the system

# Now we install the new system
sudo nixos-rebuild switch --flake .#nixos --show-trace --impure --option eval-cache false