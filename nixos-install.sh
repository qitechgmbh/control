# write the installInfo.nix while
# we capture information about the current git commit
sudo touch ./nixos/os/installInfo.nix

# Capture all git information
GIT_TIMESTAMP=$(git --no-pager show -s --format=%cI HEAD)  # e.g., "2025-06-10T14:30:45+02:00"
GIT_COMMIT=$(git rev-parse HEAD)                           # e.g., "b2c7f6e0b138174770798f84ada8b0aa65afeb"
GIT_URL=$(git config --get remote.origin.url)             # e.g., "https://github.com/qitechindustries/control.git"

# Determine the git abbreviation (tag, branch, or commit hash)
GIT_TAG=$(git describe --tags --exact-match HEAD 2>/dev/null || echo "")
if [ -n "$GIT_TAG" ]; then
  GIT_ABBREVIATION="$GIT_TAG"                              # e.g., "2.0.0" (when on a tag)
else
  GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
  if [ "$GIT_BRANCH" = "HEAD" ]; then
    GIT_ABBREVIATION=$(git rev-parse --short HEAD)         # e.g., "b2c7f6e" (when on detached HEAD/commit)
  else
    GIT_ABBREVIATION="$GIT_BRANCH"                         # e.g., "main", "develop" (when on a branch)
  fi
fi

# Create escaped version for system.nixos.label
GIT_ABBREVIATION_ESCAPED=$(echo "$GIT_ABBREVIATION" | sed -e 's/+/-/g' -e 's/[^a-zA-Z0-9:_\.-]//g')  # e.g., "2-0-0", "main", "b2c7f6e"

sudo cat > ./nixos/os/installInfo.nix << EOF
{
  gitTimestamp = "$GIT_TIMESTAMP";
  gitCommit = "$GIT_COMMIT";
  gitAbbreviation = "$GIT_ABBREVIATION";
  gitUrl = "$GIT_URL";
  # Like abbreviation but can be used in system.nixos.label
  gitAbbreviationEscaped = "$GIT_ABBREVIATION_ESCAPED";
}
EOF

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
if sudo nixos-rebuild boot --flake .#nixos --show-trace --impure --option sandbox false --option eval-cache false; then
  reboot
else
  exit 1
fi
