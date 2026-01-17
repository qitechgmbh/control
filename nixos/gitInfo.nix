{ pkgs, ... }:

let
  # Generate Git info in the store
  gitInfoDerivation = pkgs.runCommand "git-info" { } ''
    gitTimestamp="${builtins.getEnv "GIT_TIMESTAMP"}"
    gitCommit="${builtins.getEnv "GIT_COMMIT"}"
    gitUrl="${builtins.getEnv "GIT_URL"}"
    gitAbbreviation="${builtins.getEnv "GIT_ABBREVIATION"}"
    gitAbbreviationEscaped="${builtins.getEnv "GIT_ABBREVIATION_ESCAPED"}"

    mkdir -p $out

    cat <<EOF > $out/git-info.json
      {
        "gitTimestamp": "$gitTimestamp",
        "gitCommit": "$gitCommit",
        "gitAbbreviation": "$gitAbbreviation",
        "gitUrl": "$gitUrl",
        "gitAbbreviationEscaped": "$gitAbbreviationEscaped"
      }
    EOF
  '';
in builtins.fromJSON (builtins.readFile "${gitInfoDerivation}/git-info.json")
