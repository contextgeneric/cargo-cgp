//! Post-processor: strip CGP path prefixes from diagnostic text.

/// The CGP module paths rustc prints in front of CGP type names. Stripping any of them
/// turns `cgp::prelude::Chars` into the `Chars` a user writes. Kept as a list so more
/// forms can be added as they are found — the compiler qualifies CGP items through
/// several re-export paths.
pub const CGP_PREFIXES: &[&str] = &[
    "cgp::prelude::",
    "cgp::macro_prelude::",
    "cgp::cgp_core::",
    "cgp::cgp_extra::",
];

/// Remove every [`CGP_PREFIXES`] occurrence from `text`, returning the rewritten text when
/// any was removed (and `None` when the text carries no CGP prefix). Stripping is the first
/// step toward readable type names, and a CGP prefix is a reliable sign the diagnostic
/// involves CGP.
pub fn strip_cgp_prefixes(text: &str) -> Option<String> {
    let mut out = text.to_owned();
    for prefix in CGP_PREFIXES {
        out = out.replace(prefix, "");
    }
    (out != text).then_some(out)
}
