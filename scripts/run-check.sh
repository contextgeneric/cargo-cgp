#!/usr/bin/env bash
#
# Build cargo-cgp and run it on one example CGP source file under tests/examples/.
#
# Usage:
#   scripts/run-check.sh <example> [extra cargo-check args...]
#
#   <example> is either an example name (`greet_ok`) or a path to the file
#   (`tests/examples/greet_ok.rs`); anything after it is forwarded to
#   `cargo cgp check`.
#
# cgp is already a dependency of the tests package, so any example may
# `use cgp::prelude::*;`. That package is excluded from the workspace, so it is only
# ever compiled here — through cargo-cgp's driver, which is the whole point.
#
# The toolchain is whatever rustup selects for this repository (pinned in
# rust-toolchain.toml). Override it for a one-off run with, e.g.,
# `RUSTUP_TOOLCHAIN=nightly scripts/run-check.sh greet_ok`.
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
workspace_dir=$(dirname "$script_dir")
tests_dir="$workspace_dir/tests"

if [ "$#" -lt 1 ]; then
    echo "usage: $0 <example-name-or-path> [extra cargo-check args...]" >&2
    echo "available examples:" >&2
    for f in "$tests_dir"/examples/*.rs; do
        [ -e "$f" ] && echo "  $(basename "$f" .rs)" >&2
    done
    exit 2
fi

# Accept either an example name or a path to the .rs file.
example=$(basename "$1" .rs)
shift

if [ ! -f "$tests_dir/examples/$example.rs" ]; then
    echo "error: no example '$example' under $tests_dir/examples" >&2
    exit 2
fi

# Build both binaries. cargo-cgp locates cargo-cgp-driver as its sibling in the same
# target directory, so both must be built. Running cargo inside the workspace lets
# rustup pick up the pinned toolchain from rust-toolchain.toml.
( cd "$workspace_dir" && cargo build --bin cargo-cgp --bin cargo-cgp-driver )

cargo_cgp="$workspace_dir/target/debug/cargo-cgp"

# Run the tool on the chosen example. Running inside tests/ selects the excluded
# tests package (its own single-package workspace); --example checks just that file.
cd "$tests_dir"
exec "$cargo_cgp" check --example "$example" "$@"
