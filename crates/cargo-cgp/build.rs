//! Bakes the pinned toolchain channel into the front-end at build time.
//!
//! The channel lives in the workspace `rust-toolchain.toml` (the single source of truth);
//! this reads it and exposes it as `CARGO_CGP_PINNED_TOOLCHAIN`, which `config.rs` reads
//! back with `env!`. Keeping the derivation here means the constant can never drift from
//! the toolchain file — the same approach `rustc_plugin` uses for its `CHANNEL`.

use std::path::PathBuf;

fn main() {
    let toolchain_file = find_toolchain_file();
    println!("cargo::rerun-if-changed={}", toolchain_file.display());

    let text = std::fs::read_to_string(&toolchain_file)
        .unwrap_or_else(|e| panic!("reading {}: {e}", toolchain_file.display()));
    let channel = parse_channel(&text)
        .unwrap_or_else(|| panic!("no `channel` in {}", toolchain_file.display()));

    println!("cargo::rustc-env=CARGO_CGP_PINNED_TOOLCHAIN={channel}");
}

/// Locate the `rust-toolchain.toml` that pins the channel. A source checkout keeps the one
/// canonical file at the workspace root; a published crate ships its own copy in the crate
/// root (see the crate's `rust-toolchain.toml`), because the workspace file is not part of
/// the package tarball. Walking up from the manifest directory takes the first match, so the
/// same build script serves both layouts — including the isolated verification build cargo
/// runs under `target/package/` during `cargo publish`.
fn find_toolchain_file() -> PathBuf {
    let start = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    for dir in start.ancestors() {
        let candidate = dir.join("rust-toolchain.toml");
        if candidate.exists() {
            return candidate;
        }
    }
    panic!(
        "could not find rust-toolchain.toml at or above {}",
        start.display()
    );
}

/// Extract the `channel = "..."` value from a `rust-toolchain.toml`, ignoring comments.
/// A deliberately minimal scan rather than a full TOML parse, so the build script pulls in
/// no dependency for one well-known field in a file we own.
fn parse_channel(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some(rest) = line.strip_prefix("channel") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let value = rest.trim();
        let value = value.strip_prefix('"')?.split('"').next()?;
        return Some(value.to_owned());
    }
    None
}
