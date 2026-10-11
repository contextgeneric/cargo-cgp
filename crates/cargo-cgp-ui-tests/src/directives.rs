//! The `//@` header directives a fixture carries, modeled on Clippy's `//@` annotations.
//!
//! A directive is an ordinary line comment, so the compiler ignores it when the fixture is copied
//! in as `main.rs`; only the harness reads it. The recognized directives are:
//!
//! - `//@aux-build: <crate>`: build an auxiliary crate and add it as a dependency (see
//!   [`crate::aux`]).
//! - `//@rewrites-falsely: <reason>`: a `pass-through/` fixture whose tool output still differs
//!   from plain rustc's, so its snapshots pin today's output instead of the equality check.
//! - `//@json`: also run the JSON pass, pinning the diagnostics an editor reads.
//!
//! A misspelled directive would be a silent no-op, so an unknown `//@name` is rejected, and so is
//! a known name written in another comment form (`// @aux-build:` with a space, or `//! @json`):
//! a stray space after `//` is the mistake that has actually happened. A comment that merely
//! starts with an `@`-path, such as commented-out wiring, stays legal.

use std::fs;
use std::path::Path;

/// The directive names, as they follow `//@`.
const NAMES: &[&str] = &["aux-build", "rewrites-falsely", "json"];

/// The directives one fixture declares.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Directives {
    /// Auxiliary crates, from `//@aux-build:` lines, in declaration order.
    pub aux_build: Vec<String>,
    /// Why a `pass-through/` fixture still rewrites falsely (`//@rewrites-falsely:`).
    pub rewrites_falsely: Option<String>,
    /// Whether the JSON pass runs (`//@json`).
    pub json: bool,
}

/// Read a fixture's directives. A missing or unreadable fixture declares none.
pub fn read(fixture: &Path) -> Directives {
    let text = fs::read_to_string(fixture).unwrap_or_default();
    parse(&text, &fixture.display().to_string())
}

/// Parse the directives in a fixture's `text`; `name` identifies the fixture in a panic.
pub fn parse(text: &str, name: &str) -> Directives {
    let mut directives = Directives::default();

    for line in text.lines() {
        let line = line.trim();

        if let Some(rest) = line.strip_prefix("//@") {
            apply(&mut directives, rest, line, name);
            continue;
        }

        let after_slashes = line.trim_start_matches('/').trim_start_matches('!');
        let looks_like_directive = after_slashes
            .trim_start()
            .strip_prefix('@')
            .is_some_and(|rest| NAMES.iter().any(|known| rest.starts_with(known)));
        assert!(
            !(line.starts_with("//") && looks_like_directive),
            "malformed `//@` directive in {name}:\n  {line}\n\
             a directive is written `//@name` with no space between `//` and `@`",
        );
    }

    directives
}

/// Record one `//@` directive, whose text after `//@` is `rest`.
fn apply(directives: &mut Directives, rest: &str, line: &str, name: &str) {
    if let Some(value) = rest.strip_prefix("aux-build:") {
        let value = value.trim();
        if !value.is_empty() {
            directives.aux_build.push(value.to_owned());
        }
    } else if let Some(reason) = rest.strip_prefix("rewrites-falsely:") {
        let reason = reason.trim();
        assert!(
            !reason.is_empty(),
            "`//@rewrites-falsely:` needs a reason in {name}:\n  {line}"
        );
        directives.rewrites_falsely = Some(reason.to_owned());
    } else if rest.trim_end() == "json" {
        directives.json = true;
    } else {
        panic!("unknown `//@` directive in {name}:\n  {line}");
    }
}
