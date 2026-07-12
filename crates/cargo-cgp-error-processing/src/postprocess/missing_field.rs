//! Post-processor: turn an unmet `HasField` bound into a field-oriented message.
//!
//! rustc reports a getter's unmet dependency as
//! `the trait `HasField<Symbol!("name")>` is not implemented for `Context``. This rewrites
//! that to `missing field `name` on `Context``. The rewritten clause carries no CGP error
//! code: this acts on whatever sub-message the clause appears in, and codes classify only a
//! rewritten *main* message (see `crate::code`).
//!
//! It distinguishes two cases, because their fixes differ, and the tell is within the same
//! diagnostic (see the CGP `check-trait-failure` error-catalog document):
//!
//! - The context implements `HasField` for *some* field but not the one asked for — a
//!   single missing field. rustc shows a "similar impl" landmark, either inline
//!   (`but trait `HasField<…>` is implemented for it`, one other field) or as a separate
//!   `` `Context` implements trait `HasField<…>` `` note (several other fields).
//! - The context implements `HasField` for *no* field — the whole `#[derive(HasField)]` is
//!   missing. Neither landmark appears. The message instead points at the derive:
//!   `` `#[derive(HasField)]` is required to access field `name` on `Context` ``.
//!
//! Whether the context has any `HasField` impl ([`context_has_hasfield_impls`]) is a
//! property of the *whole* diagnostic — the landmark can sit far from the clause — so a
//! caller decides it once across every message before rewriting each in turn.
//!
//! Meant to run after [`strip_cgp_prefixes`](super::strip_cgp_prefixes) and
//! [`resugar_symbol`](super::resugar_symbol), so it matches the bare, resugared
//! `HasField<Symbol!("…")>` form.

/// The text that opens the unmet-`HasField` clause, once prefixes are stripped and the
/// symbol resugared.
const ANCHOR: &str = "the trait `HasField<Symbol!(\"";

/// Does the text show the context implementing `HasField` for at least one field? Both of
/// rustc's "similar impl" phrasings count; their absence across the whole diagnostic means
/// no impls at all.
pub fn context_has_hasfield_impls(text: &str) -> bool {
    text.contains("is implemented for it") || text.contains("implements trait `HasField")
}

/// Rewrite every unmet-`HasField` clause in `text` to a field-oriented message, returning
/// the rewritten text when any clause matched (and `None` otherwise). `has_field_impls`
/// selects the single-missing-field wording from the missing-derive wording — the caller
/// computes it across the whole diagnostic with [`context_has_hasfield_impls`].
pub fn rewrite_missing_fields(text: &str, has_field_impls: bool) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;

    while let Some(index) = rest.find(ANCHOR) {
        out.push_str(&rest[..index]);
        let candidate = &rest[index..];

        let Some((field, context, after_clause)) = parse_clause(candidate) else {
            out.push_str(ANCHOR);
            rest = &candidate[ANCHOR.len()..];
            continue;
        };

        changed = true;
        let after = if has_field_impls {
            out.push_str(&format!("missing field `{field}` on `{context}`"));
            // Absorb the inline single-impl landmark when it directly follows the clause,
            // so it does not dangle after the rewrite. The separate multi-impl note (when
            // present instead) is left in place.
            consume_inline_landmark(after_clause).unwrap_or(after_clause)
        } else {
            out.push_str(&format!(
                "`#[derive(HasField)]` is required to access field `{field}` on `{context}`"
            ));
            after_clause
        };

        rest = &candidate[candidate.len() - after.len()..];
    }

    out.push_str(rest);
    changed.then_some(out)
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
