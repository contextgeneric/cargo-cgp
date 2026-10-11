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
//! - `//@codes: <CGP-Exxx ...>` (or `//@codes: none`): the exact set of `[CGP-Exxx]` codes the
//!   tool's output carries, checked against it.
//! - `//@quality: acceptable|usability|hidden`: how well the tool presents the error today:
//!   the cause is presented well, present but buried, or absent from the output.
//!
//! A misspelled directive would be a silent no-op, so an unknown `//@name` is rejected, and so is
//! a known name written in another comment form (`// @aux-build:` with a space, or `//! @json`):
//! a stray space after `//` is the mistake that has actually happened. A comment that merely
//! starts with an `@`-path, such as commented-out wiring, stays legal.

use std::fs;
use std::path::Path;

/// The directive names, as they follow `//@`.
const NAMES: &[&str] = &["aux-build", "rewrites-falsely", "json", "codes", "quality"];

/// The values `//@quality:` accepts.
pub const QUALITIES: &[&str] = &["acceptable", "usability", "hidden"];

/// The directives one fixture declares.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Directives {
    /// Auxiliary crates, from `//@aux-build:` lines, in declaration order.
    pub aux_build: Vec<String>,
    /// Why a `pass-through/` fixture still rewrites falsely (`//@rewrites-falsely:`).
    pub rewrites_falsely: Option<String>,
    /// Whether the JSON pass runs (`//@json`).
    pub json: bool,
    /// The `[CGP-Exxx]` codes the tool's output must carry, exactly (`//@codes:`); `None` when the
    /// fixture does not declare them, and an empty list for `//@codes: none`.
    pub codes: Option<Vec<String>>,
    /// How well the tool presents the error (`//@quality:`), one of [`QUALITIES`].
    pub quality: Option<String>,
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
    } else if let Some(codes) = rest.strip_prefix("codes:") {
        let codes: Vec<String> = codes.split_whitespace().map(str::to_owned).collect();
        let valid =
            codes == ["none"] || (!codes.is_empty() && codes.iter().all(|code| is_cgp_code(code)));
        assert!(
            valid,
            "`//@codes:` takes `none` or `CGP-Exxx` codes in {name}:\n  {line}"
        );
        directives.codes = Some(if codes == ["none"] { Vec::new() } else { codes });
    } else if let Some(quality) = rest.strip_prefix("quality:") {
        let quality = quality.trim();
        assert!(
            QUALITIES.contains(&quality),
            "`//@quality:` takes one of {QUALITIES:?} in {name}:\n  {line}"
        );
        directives.quality = Some(quality.to_owned());
    } else {
        panic!("unknown `//@` directive in {name}:\n  {line}");
    }
}

/// Whether `text` is a `CGP-Exxx` code: `CGP-E` followed by three digits.
fn is_cgp_code(text: &str) -> bool {
    text.strip_prefix("CGP-E")
        .is_some_and(|digits| digits.len() == 3 && digits.bytes().all(|b| b.is_ascii_digit()))
}
