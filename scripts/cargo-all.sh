#!/usr/bin/env bash
# Run a cargo subcommand in every crate of this repo.
#
# The repo is a collection of standalone book exercises rather than one
# workspace, so each Cargo.toml gets its own invocation. The manifest is
# inserted straight after the subcommand so that trailing arguments meant
# for rustfmt/clippy (after `--`) stay where they belong.
#
#   scripts/cargo-all.sh clippy --all-targets -- -D warnings
set -euo pipefail

cd "$(dirname "$0")/.."

subcommand=$1
shift

mapfile -t manifests < <(
  find . -name Cargo.toml -not -path '*/target/*' -not -path './.git/*' | sort
)

if [ ${#manifests[@]} -eq 0 ]; then
  echo "no Cargo.toml found" >&2
  exit 0
fi

status=0
for manifest in "${manifests[@]}"; do
  echo "==> ${manifest%/Cargo.toml} : cargo $subcommand $*"
  cargo "$subcommand" --manifest-path "$manifest" "$@" || status=1
done
exit $status
