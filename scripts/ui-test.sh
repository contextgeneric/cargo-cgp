#!/usr/bin/env bash
#
# The UI snapshot suite. For each fixture under tests/ui/, compile it through
# cargo-cgp, normalize the output, and compare it against the committed `.stderr`
# snapshot beside the fixture. Modeled on Clippy's UI tests: a tree of `.rs` files
# each paired with a blessed `.stderr`.
#
# Usage:
#   scripts/ui-test.sh [--bless] [filter...]
#
#   --bless      Rewrite the `.stderr` snapshots from the current output instead of
#                comparing. Use after an intended change to what cargo-cgp emits, and
#                review the diff before committing.
#   filter...    Only run fixtures whose path (relative to tests/ui/) contains one of
#                these substrings. With no filter, every fixture runs.
#
# The output is deliberately the tool's own — the fixtures are compiled through
# cargo-cgp, not plain rustc — so when the driver starts reformatting diagnostics the
# snapshots here are what change. Today the driver is a passthrough, so a snapshot is
# the normalized rustc diagnostic (empty for a fixture that compiles cleanly).
#
# Toolchain selection matches run-check.sh (rust-toolchain.toml, overridable with
# RUSTUP_TOOLCHAIN).
set -euo pipefail

# shellcheck source=lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

bless=0
filters=()
for arg in "$@"; do
    case "$arg" in
        --bless) bless=1 ;;
        *) filters+=("$arg") ;;
    esac
done

root=$(repo_root)
ui_dir="$root/tests/ui"

cargo_cgp=$(build_binaries)
ensure_harness_crate

mapfile -t fixtures < <(find "$ui_dir" -name '*.rs' | sort)

matches_filter() {
    [ "${#filters[@]}" -eq 0 ] && return 0
    local f
    for f in "${filters[@]}"; do
        [[ "$1" == *"$f"* ]] && return 0
    done
    return 1
}

failed=0
ran=0
for fixture in "${fixtures[@]}"; do
    name=${fixture#"$ui_dir"/}
    matches_filter "$name" || continue
    ran=$((ran + 1))

    snapshot="${fixture%.rs}.stderr"
    actual=$(run_fixture "$cargo_cgp" "$fixture" | normalize_output || true)

    if [ "$bless" -eq 1 ]; then
        printf '%s\n' "$actual" > "$snapshot"
        echo "blessed  $name"
        continue
    fi

    expected=""
    [ -f "$snapshot" ] && expected=$(cat "$snapshot")

    if [ "$actual" == "$expected" ]; then
        echo "ok       $name"
    else
        echo "MISMATCH $name"
        diff --label "expected ($name.stderr)" --label "actual" -u \
            <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") || true
        failed=1
    fi
done

if [ "$ran" -eq 0 ]; then
    echo "no fixtures matched" >&2
    exit 2
fi

if [ "$bless" -eq 0 ] && [ "$failed" -ne 0 ]; then
    echo "snapshot mismatch — re-run with --bless to update after an intended change" >&2
fi
exit "$failed"
