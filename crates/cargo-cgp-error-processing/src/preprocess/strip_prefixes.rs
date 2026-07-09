//! Preprocessor: strip CGP path prefixes from diagnostic text.

use crate::diagnostic::CgpDiagnostic;
use crate::preprocess::text::map_diagnostic_text;

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

/// Strip every [`CGP_PREFIXES`] occurrence from the diagnostic's text, and set
/// `has_cgp_error` if any was removed. A CGP prefix is a reliable sign the diagnostic
/// involves CGP, so its presence is what flags the diagnostic; its removal is the first
/// step toward readable type names.
pub fn strip_cgp_prefixes(mut diagnostic: CgpDiagnostic) -> CgpDiagnostic {
    if map_diagnostic_text(&mut diagnostic.diagnostic, strip_prefixes_in_text) {
        diagnostic.has_cgp_error = true;
    }
    diagnostic
}

/// Remove all CGP prefixes from `text`, returning the result and whether anything changed.
fn strip_prefixes_in_text(text: &str) -> (String, bool) {
    let mut out = text.to_owned();
    for prefix in CGP_PREFIXES {
        out = out.replace(prefix, "");
    }
    let changed = out != text;
    (out, changed)
}
