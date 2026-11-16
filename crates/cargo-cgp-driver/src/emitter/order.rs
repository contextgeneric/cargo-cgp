//! Reading a diagnostic's place in the canonical order, and sorting by it.
//!
//! The order itself is rustc-free ([`cargo_cgp_error_processing::order`]); this module reads its
//! key off a `DiagInner`: the primary span's file and offsets within that file, the code, the main
//! message, and the rest of the diagnostic's content as the detail that breaks the last tie.

use cargo_cgp_error_processing::byte_range::ByteRange;
use cargo_cgp_error_processing::order::{OrderKey, SpanPlace, Spanless, sort_by_order_key};
use rustc_errors::{DiagInner, DiagMessage, MultiSpan, Suggestions};
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::emitter::edit::main_message_text;

/// The place of `diag` in the canonical order, read against `source_map`. A dummy primary span
/// counts as none.
pub fn order_key(diag: &DiagInner, source_map: &SourceMap) -> OrderKey {
    let message = match main_message_text(diag) {
        Some(text) => text.to_owned(),
        None => format!("{:?}", diag.messages.first()),
    };
    OrderKey {
        place: primary_place(&diag.span, source_map),
        code: diag.code.map(|code| code.to_string()),
        message,
        detail: detail(diag, source_map),
    }
}

/// `items` in the canonical order of the diagnostic `diag_of` gives each, items whose diagnostics
/// tie keeping their order; `spanless` says how the diagnostics without a span compare.
pub fn sort_canonically<T>(
    items: Vec<T>,
    diag_of: impl Fn(&T) -> &DiagInner,
    source_map: &SourceMap,
    spanless: Spanless,
) -> Vec<T> {
    sort_by_order_key(items, |item| order_key(diag_of(item), source_map), spanless)
}

/// The place of `span`'s primary span, or `None` when it has none or a dummy one.
fn primary_place(span: &MultiSpan, source_map: &SourceMap) -> Option<SpanPlace> {
    span.primary_span()
        .filter(|span| !span.is_dummy())
        .map(|span| place(span, source_map))
}

/// Where `span` sits: its file's name and its offsets within that file.
fn place(span: Span, source_map: &SourceMap) -> SpanPlace {
    let start = source_map.lookup_byte_offset(span.lo());
    let lo = start.pos.0;
    let hi = span.hi().0.saturating_sub(start.sf.start_pos.0).max(lo);
    SpanPlace {
        file: start.sf.name.prefer_local_unconditionally().to_string(),
        range: ByteRange { lo, hi },
    }
}

/// `place` as a string, or `-` for a dummy span.
fn place_text(span: Span, source_map: &SourceMap) -> String {
    if span.is_dummy() {
        return "-".to_owned();
    }
    let place = place(span, source_map);
    format!("{}:{}-{}", place.file, place.range.lo, place.range.hi)
}

/// A message's text, whichever form it takes.
fn message_text(message: &DiagMessage) -> &str {
    match message {
        DiagMessage::Str(text) | DiagMessage::Inline(text) => text,
    }
}

/// The content of `diag` beyond its primary span, code, and main message, in a fixed order and
/// with every span read within its file: each span label, each child with its level, messages,
/// and primary span, each suggestion with its message and edits, the message arguments sorted by
/// name, and then the parts added after those, so they break only a tie the earlier parts leave:
/// the level, the lint, each child's span labels, each suggestion's style and applicability, every
/// part of the main message with its style, each child message's style, each suggestion's variants
/// apart, and the emission site and long-type file rustc can print.
///
/// Postcondition (assumed from what rustc's human and JSON emitters print): two diagnostics with
/// the same primary span, code, and main message, whose results are equal, print the same.
fn detail(diag: &DiagInner, source_map: &SourceMap) -> Vec<String> {
    let mut parts = Vec::new();
    for label in diag.span.span_labels() {
        parts.push(format!(
            "label {} {} {}",
            label.is_primary,
            place_text(label.span, source_map),
            label.label.as_ref().map_or("", message_text)
        ));
    }
    for child in &diag.children {
        let messages: Vec<&str> = child
            .messages
            .iter()
            .map(|(message, _)| message_text(message))
            .collect();
        let span = child
            .span
            .primary_span()
            .map_or_else(|| "-".to_owned(), |span| place_text(span, source_map));
        parts.push(format!(
            "child {:?} {span} {}",
            child.level,
            messages.join("\n")
        ));
    }
    let suggestions: &[_] = match &diag.suggestions {
        Suggestions::Enabled(suggestions) => suggestions,
        Suggestions::Sealed(suggestions) => suggestions,
        Suggestions::Disabled => &[],
    };
    for suggestion in suggestions {
        let edits: Vec<String> = suggestion
            .substitutions
            .iter()
            .flat_map(|substitution| &substitution.parts)
            .map(|part| format!("{} {}", place_text(part.span, source_map), part.snippet))
            .collect();
        parts.push(format!(
            "suggestion {} {}",
            message_text(&suggestion.msg),
            edits.join("\n")
        ));
    }
    let mut args: Vec<String> = diag
        .args
        .iter()
        .map(|(name, value)| format!("arg {name} {value:?}"))
        .collect();
    args.sort();
    parts.extend(args);
    parts.push(format!("level {:?}", diag.level()));
    parts.push(format!("lint {:?}", diag.is_lint));
    for (index, child) in diag.children.iter().enumerate() {
        for label in child.span.span_labels() {
            parts.push(format!(
                "child {index} label {} {} {}",
                label.is_primary,
                place_text(label.span, source_map),
                label.label.as_ref().map_or("", message_text)
            ));
        }
    }
    for (index, suggestion) in suggestions.iter().enumerate() {
        parts.push(format!(
            "suggestion {index} {:?} {:?}",
            suggestion.style, suggestion.applicability
        ));
    }
    for (index, (message, style)) in diag.messages.iter().enumerate() {
        parts.push(format!(
            "message {index} {style:?} {}",
            message_text(message)
        ));
    }
    for (index, child) in diag.children.iter().enumerate() {
        let styles: Vec<String> = child
            .messages
            .iter()
            .map(|(_, style)| format!("{style:?}"))
            .collect();
        parts.push(format!("child {index} styles {}", styles.join(" ")));
    }
    for (index, suggestion) in suggestions.iter().enumerate() {
        let variants: Vec<String> = suggestion
            .substitutions
            .iter()
            .map(|substitution| substitution.parts.len().to_string())
            .collect();
        parts.push(format!(
            "suggestion {index} variants {}",
            variants.join(" ")
        ));
    }
    parts.push(format!("emitted at {:?}", diag.emitted_at));
    parts.push(format!("long type {:?}", diag.long_ty_path));
    parts
}
