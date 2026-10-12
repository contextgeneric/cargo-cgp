//! Post-processor: turn an unmet `HasField` bound into a missing-field message, where the text shows
//! the field is missing.
//!
//! rustc reports a getter's unmet dependency as
//! `the trait `HasField<Symbol!("name")>` is not implemented for `Context``. This rewrites
//! that to `missing field `name` on `Context`` when the same diagnostic shows the context
//! implementing `HasField` for some other field: rustc's "similar impl" landmark, either inline
//! (`but trait `HasField<…>` is implemented for it`, one other field) or as a separate
//! `` `Context` implements trait `HasField<…>` `` note (several other fields). A struct that
//! implements `HasField` for its other fields derives it, so the one field it lacks is missing.
//! The rewritten clause carries no CGP error code: this acts on whatever sub-message the clause
//! appears in, and codes classify only a rewritten *main* message (see `crate::code`).
//!
//! Every other clause keeps rustc's wording, because the text cannot tell what the fix is:
//!
//! - **No landmark:** the context implements `HasField` for no field, and the text cannot tell a
//!   struct that lacks the derive from one that lacks the field, or a fieldless struct whose derive
//!   generates nothing.
//! - **A numeric tag:** `Symbol!("0")` never names a field, since a field name cannot start with a
//!   digit; a tuple struct's fields are keyed `Index<0>`, so the field is not missing.
//! - **A generated parameter:** `__Context__` is the context a macro-generated impl is generic
//!   over, not a struct, so the fix is a bound on it (`#[uses(…)]`), not a field.
//!
//! Whether the context has any `HasField` impl ([`context_has_hasfield_impls`]) is a property of
//! the *whole* diagnostic, since the landmark can sit far from the clause, so a caller decides it
//! once across every message before rewriting each in turn.
//!
//! Meant to run after [`strip_cgp_prefixes`](super::strip_cgp_prefixes) and
//! [`resugar_symbol`](super::resugar_symbol), so it matches the bare, resugared
//! `HasField<Symbol!("…")>` form.

use crate::postprocess::strip_modules::strip_module_paths;

/// The text that opens the unmet-`HasField` clause, once prefixes are stripped and the
/// symbol resugared.
const ANCHOR: &str = "the trait `HasField<Symbol!(\"";

/// Does the text show the context implementing `HasField` for at least one field? Both of
/// rustc's "similar impl" phrasings count; their absence across the whole diagnostic means
/// no impls at all. Module paths are stripped first, since rustc prints a glob-imported trait
/// qualified (`implements trait `cgp::prelude::HasField<Tag>``). A caller passes a message's
/// fragments joined, since rustc highlights the `is` of `is implemented for it` apart.
pub fn context_has_hasfield_impls(text: &str) -> bool {
    let text = strip_module_paths(text).unwrap_or_else(|| text.to_owned());
    text.contains("is implemented for it") || text.contains("implements trait `HasField")
}

/// Rewrite every unmet-`HasField` clause in `text` that names a missing field into
/// `missing field `name` on `Context``, returning the rewritten text when any clause matched (and
/// `None` otherwise). `has_field_impls` says whether the diagnostic shows the context implementing
/// `HasField` for another field, which the caller computes across the whole diagnostic with
/// [`context_has_hasfield_impls`]; without it, or for a numeric tag or a generated context, the
/// clause keeps rustc's wording (see the module documentation).
pub fn rewrite_missing_fields(text: &str, has_field_impls: bool) -> Option<String> {
    if !has_field_impls {
        return None;
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;

    while let Some(index) = rest.find(ANCHOR) {
        out.push_str(&rest[..index]);
        let candidate = &rest[index..];

        let Some((field, context, after_clause)) = parse_clause(candidate)
            .filter(|(field, context, _)| names_a_missing_field(field, context))
        else {
            out.push_str(ANCHOR);
            rest = &candidate[ANCHOR.len()..];
            continue;
        };

        changed = true;
        out.push_str(&format!("missing field `{field}` on `{context}`"));
        // Absorb the inline single-impl landmark when it directly follows the clause, so it does
        // not dangle after the rewrite. The separate multi-impl note (when present instead) is
        // left in place.
        let after = consume_inline_landmark(after_clause).unwrap_or(after_clause);
        rest = &candidate[candidate.len() - after.len()..];
    }

    out.push_str(rest);
    changed.then_some(out)
}

/// Whether a clause's tag and context can name a missing struct field: a tag a field could be
/// named (not starting with a digit) on a context the program wrote (not a `__Name__` parameter a
/// macro generated).
fn names_a_missing_field(field: &str, context: &str) -> bool {
    let numeric = field.starts_with(|c: char| c.is_ascii_digit());
    let generated = context.len() > 4 && context.starts_with("__") && context.ends_with("__");
    !numeric && !generated
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
