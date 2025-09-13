let
  gitTimestamp = builtins.getEnv "GIT_TIMESTAMP";
  gitCommit = builtins.getEnv "GIT_COMMIT";
  gitAbbreviation = builtins.getEnv "GIT_ABBREVIATION";
  gitUrl = builtins.getEnv "GIT_URL";
# Like abbreviation but can be used in system.nixos.label
  gitAbbreviationEscaped = builtins.getEnv "GIT_ABBREVIATION_ESCAPED";
in
{
  gitTimestamp = gitTimestamp;
  gitCommit = gitCommit;
  gitAbbreviation = gitAbbreviation;
  gitUrl = gitUrl;
  gitAbbreviationEscaped = gitAbbreviationEscaped;
}
