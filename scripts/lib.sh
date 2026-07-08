# shellcheck shell=bash
#
# Shared helpers for the cargo-cgp test scripts. Source this file; do not run it.
#
# The functions here build the two binaries and compile a single UI fixture through
# the real cargo-cgp, in a throwaway crate that carries cgp as a dependency. Both
# `run-check.sh` (interactive, raw output) and `ui-test.sh` (snapshot suite) build on
# them, so the tool is exercised end to end exactly the same way in each.

# Absolute path to the repository root (the directory holding this scripts/ folder).
repo_root() {
    cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd
}

# Absolute path to the sibling cgp facade crate, assumed to live at ../cgp.
cgp_crate_dir() {
    ( cd "$(repo_root)/../cgp/crates/main/cgp" && pwd )
}

# The throwaway crate the fixtures are compiled in. It lives under the workspace
# target directory (git-ignored) so cgp is built once and cached across fixtures.
harness_crate_dir() {
    echo "$(repo_root)/target/ui-harness"
}

# Build both binaries and echo the path to the cargo-cgp front-end. Build output goes
# to stderr so the echoed path is the only thing on stdout.
build_binaries() {
    local root
    root=$(repo_root)
    ( cd "$root" && cargo build --bin cargo-cgp --bin cargo-cgp-driver ) 1>&2
    echo "$root/target/debug/cargo-cgp"
}

# Create or refresh the throwaway crate (idempotent). Naming it `ui` keeps cargo's
# "could not compile `ui`" line stable across fixtures; the empty [workspace] table
# stops cargo from treating it as part of the cargo-cgp workspace above it.
ensure_harness_crate() {
    local dir
    dir=$(harness_crate_dir)
    mkdir -p "$dir/src"
    cat > "$dir/Cargo.toml" <<EOF
[package]
name    = "ui"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
cgp = { path = "$(cgp_crate_dir)" }

[workspace]
EOF
    # A placeholder so the crate is always valid before a fixture is copied in.
    [ -f "$dir/src/main.rs" ] || echo "fn main() {}" > "$dir/src/main.rs"
}

# Run `cargo-cgp check` on one fixture file, emitting the tool's combined output and
# returning its exit code. Colour is disabled so snapshots are plain text.
run_fixture() {
    local cargo_cgp=$1 fixture=$2 dir
    dir=$(harness_crate_dir)
    cp "$fixture" "$dir/src/main.rs"
    ( cd "$dir" && "$cargo_cgp" check --color never ) 2>&1
}

# Reduce cargo-cgp output to the stable diagnostic text worth snapshotting: drop
# cargo's own progress/status lines (which carry volatile paths and timings), leaving
# the compiler diagnostics and the final "could not compile" summary. This is where a
# future reformatting of the diagnostics by the driver will show up as a diff.
normalize_output() {
    sed -E '/^[[:space:]]+(Checking|Compiling|Finished|Building|Blocking|Locking|Updating|Downloading|Downloaded|Fresh|Running|Installing|Removing)[[:space:]]/d'
}
