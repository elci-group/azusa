#!/bin/sh
# Propagate the kaptaind-managed root VERSION into the workspace manifest.
#
# Root VERSION is the single version authority (theosis reconciles against
# it; the binary embeds it). This script mechanically mirrors it into
# Cargo.toml's [workspace.package] version so crate metadata, SBOMs and
# external scanners never disagree with the release version.
#
# CI runs this before the build and fails on `git diff --exit-code`, which
# forces every VERSION bump to land together with its Cargo propagation.

set -eu
cd "$(dirname "$0")/.."

version=$(cat VERSION)
# Only the bare `version = "..."` line (the [workspace.package] field);
# member crates use `version.workspace = true`.
sed -i.bak "s/^version = \".*\"$/version = \"${version}\"/" Cargo.toml
rm -f Cargo.toml.bak

if [ "$(grep -c '^version' Cargo.toml)" -lt 1 ]; then
    echo "error: workspace version line not found" >&2
    exit 1
fi
echo "workspace version synced to ${version}"
