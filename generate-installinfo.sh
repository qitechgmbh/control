touch /tmp/installInfo.nix

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

tee /tmp/installInfo.nix > /dev/null << EOF
{
  gitTimestamp = "$GIT_TIMESTAMP";
  gitCommit = "$GIT_COMMIT";
  gitAbbreviation = "$GIT_ABBREVIATION";
  gitUrl = "$GIT_URL";
  # Like abbreviation but can be used in system.nixos.label
  gitAbbreviationEscaped = "$GIT_ABBREVIATION_ESCAPED";
}
EOF