//! The UI-test entrypoint — a custom harness (`harness = false` in Cargo.toml), like
//! Clippy's `tests/compile-test.rs`. libtest is disabled, so this `fn main` is the test
//! binary: `cargo test -p cargo-cgp-ui-tests` runs it. All logic lives in the library's
//! [`cargo_cgp_ui_tests::run`]; this only forwards the arguments cargo passes after `--`.

fn main() {
    cargo_cgp_ui_tests::run(std::env::args().skip(1).collect());
}
