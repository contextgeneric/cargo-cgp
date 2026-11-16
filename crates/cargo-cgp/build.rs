//! Bakes the pinned toolchain channel from the workspace `rust-toolchain.toml` into the front-end
//! as `CARGO_CGP_PINNED_TOOLCHAIN`; see
//! `cgp-knowledge-base/cargo-cgp/implementation/distribution/pinned-nightly-bump.md`.

use std::path::PathBuf;

fn main() {
    let toolchain_file = toolchain_file();
    println!("cargo::rerun-if-changed={}", toolchain_file.display());

    let text = std::fs::read_to_string(&toolchain_file)
        .unwrap_or_else(|e| panic!("reading {}: {e}", toolchain_file.display()));
    let channel = parse_channel(&text)
        .unwrap_or_else(|| panic!("no `channel` in {}", toolchain_file.display()));

    println!("cargo::rustc-env=CARGO_CGP_PINNED_TOOLCHAIN={channel}");
}

/// The crate's `pinned-toolchain.toml`: a link to the workspace `rust-toolchain.toml`, which cargo
/// packages as a copy. It has its own name because docs.rs deletes a crate's `rust-toolchain.toml`
/// before building it.
fn toolchain_file() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("pinned-toolchain.toml")
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
