# write the installInfo.nix while
# we capture information about the current git commit
sudo touch ./nixos/os/installInfo.nix
sudo sh -c 'echo "{
  gitTimestamp = \"$(git --no-pager show -s --format=%cI HEAD)\";
  gitCommit = \"$(git rev-parse HEAD)\";
  gitAbbreviation = \"$(git rev-parse --abbrev-ref HEAD)\";
  gitUrl = \"$(git config --get remote.origin.url)\";
  # Like abbreviation but can be used in system.nixos.label
  gitAbbreviationEscaped = \"$(git rev-parse --abbrev-ref HEAD | sed -e "s/+/-/g" -e "s/[^a-zA-Z0-9:_\.-]//g")\";
  # Like timestamp but can be used in system.nixos.label
  currentTimestampEscaped = \"$(date -Iseconds | sed -e "s/+/-/g" -e "s/[^a-zA-Z0-9:_\.-]//g")\";
}" > ./nixos/os/installInfo.nix'

# make sure the installInfo.nix file is tracked by git
# otherwise it will be ignored by nix
# configure a git user
git config --global user.name "nixos-install.sh"
git config --global user.email "nixos-install.sh@localhost"
# add the installInfo.nix file to the git index
git add ./nixos/os/installInfo.nix
# commit the changes
git commit -m "Add installInfo.nix file with current commit information"
# Now we can install the system

# Now we install the new system
sudo nixos-rebuild boot --flake .#nixos --show-trace --impure --option eval-cache false

# Reboot afterwards
reboot
