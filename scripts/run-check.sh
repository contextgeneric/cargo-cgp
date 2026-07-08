#!/usr/bin/env bash
#
# Build cargo-cgp and run it on one UI fixture under tests/ui/, printing the tool's
# raw output. This is the interactive counterpart to the snapshot suite in
# ui-test.sh: same end-to-end path through cargo-cgp, but the output is shown as-is
# (not normalized or compared) so you can read exactly what the tool emits.
#
# Usage:
#   scripts/run-check.sh <fixture>
#
#   <fixture> is a path to a `.rs` file under tests/ui/, or a substring that uniquely
#   identifies one (e.g. `unsatisfied_dependency`). With no argument the available
#   fixtures are listed.
#
# The toolchain is whatever rustup selects for this repository (pinned in
# rust-toolchain.toml). Override it for a one-off with, e.g.,
# `RUSTUP_TOOLCHAIN=nightly scripts/run-check.sh greet`.
set -euo pipefail

# shellcheck source=lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

root=$(repo_root)
ui_dir="$root/tests/ui"

list_fixtures() {
    find "$ui_dir" -name '*.rs' | sed "s#^$ui_dir/##" | sort
}

if [ "$#" -lt 1 ]; then
    echo "usage: $0 <fixture-path-or-name>" >&2
    echo "fixtures:" >&2
    list_fixtures | sed 's/^/  /' >&2
    exit 2
fi

# Resolve the fixture: an existing path, or a unique substring match under tests/ui/.
if [ -f "$1" ]; then
    fixture="$1"
else
    mapfile -t hits < <(find "$ui_dir" -name '*.rs' | grep -F -- "$1" || true)
    if [ "${#hits[@]}" -ne 1 ]; then
        echo "error: '$1' does not name exactly one fixture under $ui_dir" >&2
        exit 2
    fi
    fixture="${hits[0]}"
fi

cargo_cgp=$(build_binaries)
ensure_harness_crate
run_fixture "$cargo_cgp" "$fixture"
