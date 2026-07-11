//! Preprocessor: turn an unmet `HasField` bound into a field-oriented message.
//!
//! rustc reports a getter's unmet dependency as
//! `the trait `HasField<Symbol!("name")>` is not implemented for `Context``. This rewrites
//! that to `[CGP0001] missing field `name` in `Context``, and extracts a
//! [`CgpDiagnosticDetail`]. Because the whole message is replaced, it is tagged with a CGP
//! error code (see `docs/error-code.md`); the [`code`](CgpDiagnosticDetail::code) is prefixed
//! in a `[CGPxxxx]` form chosen to look nothing like rustc's `E0277`.
//!
//! It distinguishes two cases, because their fixes differ, and the tell is within the same
//! diagnostic (see the CGP `check-trait-failure` error-catalog document):
//!
//! - The context implements `HasField` for *some* field but not the one asked for — a
//!   single missing field (`CGP0001`). rustc shows a "similar impl" landmark, either inline
//!   (`but trait `HasField<…>` is implemented for it`, one other field) or as a separate
//!   `` `Context` implements trait `HasField<…>` `` note (several other fields).
//! - The context implements `HasField` for *no* field — the whole `#[derive(HasField)]` is
//!   missing (`CGP0002`). Neither landmark appears. The message instead points at the derive:
//!   `` [CGP0002] `#[derive(HasField)]` is required to access field `name` in `Context` ``.
//!
//! Runs after [`strip_cgp_prefixes`](super::strip_cgp_prefixes) and
//! [`resugar_symbol`](super::resugar_symbol), so it matches the bare, resugared
//! `HasField<Symbol!("…")>` form.

use crate::diagnostic::{CgpDiagnostic, CgpDiagnosticDetail};

/// The text that opens the unmet-`HasField` clause, once prefixes are stripped and the
/// symbol resugared.
const ANCHOR: &str = "the trait `HasField<Symbol!(\"";

/// Rewrite each unmet-`HasField` clause in the diagnostic to a field-oriented message and
/// record a [`CgpDiagnosticDetail`] for it. Sets `has_cgp_error` when anything matched.
pub fn extract_missing_fields(mut diagnostic: CgpDiagnostic) -> CgpDiagnostic {
    // Whether the context implements `HasField` for any field is a property of the whole
    // diagnostic (the landmark can sit far from the clause), so decide it once up front.
    let has_field_impls = {
        let diag = &diagnostic.diagnostic;
        context_has_hasfield_impls(diag.rendered.as_deref().unwrap_or(&diag.message))
    };

    // Rewrite the message; its details are only a fallback for a diagnostic with no
    // rendered form, since `rendered` is the full text and normally carries every clause.
    let (message, message_details) = rewrite(&diagnostic.diagnostic.message, has_field_impls);
    diagnostic.diagnostic.message = message;

    let details = match diagnostic.diagnostic.rendered.take() {
        Some(rendered) => {
            let (rendered, details) = rewrite(&rendered, has_field_impls);
            diagnostic.diagnostic.rendered = Some(rendered);
            details
        }
        None => message_details,
    };

    if !details.is_empty() {
        diagnostic.has_cgp_error = true;
        diagnostic.details.extend(details);
    }

    diagnostic
}

/// Does the diagnostic show the context implementing `HasField` for at least one field?
/// Both of rustc's "similar impl" phrasings count; their absence means no impls at all.
fn context_has_hasfield_impls(text: &str) -> bool {
    text.contains("is implemented for it") || text.contains("implements trait `HasField")
}

/// Rewrite every unmet-`HasField` clause in `text`, returning the result and the details.
/// `has_field_impls` selects the message and detail for each clause.
fn rewrite(text: &str, has_field_impls: bool) -> (String, Vec<CgpDiagnosticDetail>) {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut details = Vec::new();

    while let Some(index) = rest.find(ANCHOR) {
        out.push_str(&rest[..index]);
        let candidate = &rest[index..];

        let Some((field, context, after_clause)) = parse_clause(candidate) else {
            out.push_str(ANCHOR);
            rest = &candidate[ANCHOR.len()..];
            continue;
        };

        // A fully rewritten CGP message is tagged with its error code (see
        // `docs/error-code.md`); the `[CGPxxxx]` prefix is deliberately unlike rustc's
        // `E0277` so a reader never confuses the two.
        let after = if has_field_impls {
            out.push_str(&format!(
                "[{}] missing field `{field}` in `{context}`",
                CgpDiagnosticDetail::MISSING_FIELD_CODE
            ));
            details.push(CgpDiagnosticDetail::MissingField {
                field_name: field,
                context,
            });
            // Absorb the inline single-impl landmark when it directly follows the clause,
            // so it does not dangle after the rewrite. The separate multi-impl note (when
            // present instead) is left in place.
            consume_inline_landmark(after_clause).unwrap_or(after_clause)
        } else {
            out.push_str(&format!(
                "[{}] `#[derive(HasField)]` is required to access field `{field}` in `{context}`",
                CgpDiagnosticDetail::MISSING_DERIVE_CODE
            ));
            details.push(CgpDiagnosticDetail::MissingDeriveHasField {
                field_name: field,
                context,
            });
            after_clause
        };

        rest = &candidate[candidate.len() - after.len()..];
    }

    out.push_str(rest);
    (out, details)
}

/// Parse `the trait `HasField<Symbol!("FIELD")>` is not implemented for `CONTEXT`` at the
/// start of `input`, returning the field, the context, and the text after the clause.
fn parse_clause(input: &str) -> Option<(String, String, &str)> {
    let rest = input.strip_prefix(ANCHOR)?;
    let (field, rest) = take_until(rest, "\")")?;
    let rest = rest.strip_prefix(">` is not implemented for `")?;
    let (context, rest) = take_until(rest, "`")?;
    Some((field.to_owned(), context.to_owned(), rest))
}

/// If the inline single-impl landmark (`but trait `HasField<…>` is implemented for it`)
/// directly follows, return the text after it; otherwise `None`.
fn consume_inline_landmark(after_clause: &str) -> Option<&str> {
    let trimmed = after_clause.trim_start();
    if !trimmed.starts_with("but trait `HasField<") {
        return None;
    }
    const END: &str = "is implemented for it";
    let index = trimmed.find(END)?;
    Some(&trimmed[index + END.len()..])
}

/// Split `s` at the first occurrence of `delimiter`, returning the text before it and the
/// text after it.
fn take_until<'a>(s: &'a str, delimiter: &str) -> Option<(&'a str, &'a str)> {
    let index = s.find(delimiter)?;
    Some((&s[..index], &s[index + delimiter.len()..]))
}
