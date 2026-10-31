//! Finding the cargo executable.

/// The cargo executable, from the `CARGO` cargo sets for us, falling back to `cargo`.
pub fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}
