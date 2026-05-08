#!/usr/bin/env bash
# Manage git hooks for this repository.
#
# Usage:
#   ./setup-hooks.sh            # install (default)
#   ./setup-hooks.sh install    # install hooks (sets core.hooksPath)
#   ./setup-hooks.sh uninstall  # disable hooks (unsets core.hooksPath)
#   ./setup-hooks.sh status     # show current state

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

cmd=${1:-install}

case "$cmd" in
    install)
        if [ ! -d .githooks ]; then
            echo "Error: .githooks/ directory not found at $repo_root" >&2
            exit 1
        fi
        chmod +x .githooks/*
        git config core.hooksPath .githooks
        echo "Git hooks installed (core.hooksPath = .githooks)."
        echo "Active hooks:"
        ls -1 .githooks | sed 's/^/  - /'
        ;;
    uninstall|disable)
        if git config --local --get core.hooksPath >/dev/null 2>&1; then
            git config --local --unset core.hooksPath
            echo "Git hooks disabled (core.hooksPath unset)."
        else
            echo "Git hooks already disabled (no core.hooksPath set)."
        fi
        ;;
    status)
        current=$(git config --local --get core.hooksPath || echo "")
        if [ -n "$current" ]; then
            echo "Hooks enabled. core.hooksPath = $current"
        else
            echo "Hooks disabled (no core.hooksPath set; git uses default .git/hooks)."
        fi
        ;;
    -h|--help|help)
        sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
        ;;
    *)
        echo "Unknown command: $cmd" >&2
        echo "Use: install | uninstall | status" >&2
        exit 1
        ;;
esac
