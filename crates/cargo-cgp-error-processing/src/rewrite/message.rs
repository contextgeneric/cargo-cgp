//! Rewriting CGP wiring messages to name the traits behind a component marker.
//!
//! rustc reports a CGP wiring failure in terms of the internal wiring traits, which name the
//! *component marker* but not the consumer or provider trait a reader thinks in — both in the
//! primary header the error opens with and in the obligation-chain notes:
//!
//! ```text
//! the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied
//! required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`
//! required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`
//! ```
//!
//! Given a [`ComponentNameMap`] from a marker's name to the trait names behind it, these
//! rewrites turn both forms into ones that name the traits. The header rewrite also classifies
//! the message with its [CGP error code](crate::code), since an unsatisfied
//! `CanUseComponent`/`IsProviderFor` bound is a recognized CGP error class:
//!
//! ```text
//! [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
//! required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle`
//! required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`
//! ```
//!
//! [`rewrite_message`] is the entry point; it dispatches to the note-form rewrite
//! ([`rewrite_required_for`]) and the header rewrites ([`rewrite_trait_bound`], and
//! [`rewrite_wiring_overflow`] for the `E0275` a wiring cycle produces). The codes
//! belong on *main* messages only — the driver applies the full dispatch to a
//! diagnostic's header and [`rewrite_required_for`] to its sub-messages.
//!
//! Each form parses the message *before* consulting `names`, so a message that is not a CGP
//! wiring form returns `None` without ever forcing the map's lazy initializer.

use crate::code::{CONSUMER_TRAIT_UNIMPLEMENTED, PROVIDER_TRAIT_UNIMPLEMENTED, WIRING_OVERFLOW};
use crate::rewrite::names::ComponentNameMap;
use crate::rewrite::parse::parse_trait_bound;
use crate::rewrite::text::{last_segment, split_generics, split_top_level};

/// Rewrite one diagnostic message into its trait-named form, or return `None` to leave it
/// unchanged. This is the entry point the emitter drives over every message; it tries each
/// recognized CGP wiring form — the obligation-chain notes ([`rewrite_required_for`]), the
/// primary `the trait bound … is not satisfied` header ([`rewrite_trait_bound`]), then the
/// `overflow evaluating the requirement …` header ([`rewrite_wiring_overflow`]).
pub fn rewrite_message(message: &str, names: &ComponentNameMap) -> Option<String> {
    rewrite_required_for(message, names)
        .or_else(|| rewrite_trait_bound(message, names))
        .or_else(|| rewrite_wiring_overflow(message, names))
}

/// Rewrite the `overflow evaluating the requirement …` header (`E0275`) of a wiring cycle —
/// stamping it [`WIRING_OVERFLOW`] — or return `None` for any other overflow. Only an overflow
/// whose requirement is a `Self: CanUseComponent<Marker, …>` bound is a wiring cycle: the lookup
/// for that component recurses without bottoming out, which almost always means the component is
/// delegated back to the context itself (a `UseContext` delegation with no direct consumer-trait
/// impl). The accompanying fix rides in [`wiring_overflow_help`], kept out of the header so the
/// headline stays one sentence.
pub fn rewrite_wiring_overflow(message: &str, names: &ComponentNameMap) -> Option<String> {
    let rest = message.strip_prefix("overflow evaluating the requirement `")?;
    let (bound, _tail) = rest.split_once('`')?;
    let (subject, trait_ref) = bound.split_once(": ")?;
    let (path, args_str) = split_generics(trait_ref)?;
    if last_segment(path) != "CanUseComponent" {
        return None;
    }
    let args = split_top_level(args_str);
    let component = last_segment(args.first()?.trim());
    let entry = names.get(component)?;
    let generics = render_trait_generics(&[], &args[1..]);
    Some(format!(
        "[{WIRING_OVERFLOW}] the wiring for the consumer trait `{}{generics}` on context `{subject}` never resolves — the lookup recurses without terminating",
        entry.consumer
    ))
}

/// The `help` accompanying a rewritten wiring-overflow header ([`rewrite_wiring_overflow`]):
/// what a non-terminating lookup usually means, and the two ways to fix it.
pub fn wiring_overflow_help() -> String {
    "a lookup that recurses usually means the component is delegated back to the context itself \
     — e.g. wired to `UseContext` with no direct implementation of the consumer trait — so wire \
     a real provider, or implement the consumer trait directly on the context"
        .to_owned()
}

/// Rewrite one `required for … to implement …` obligation-chain note into its trait-named
/// form, or return `None` to leave the message untouched.
///
/// Returns `None` for any message that is not one of the two recognized note forms, and for a
/// recognized form whose component marker is absent from `names` — so a message is only ever
/// rewritten when the replacement names are known.
pub fn rewrite_required_for(message: &str, names: &ComponentNameMap) -> Option<String> {
    let rest = message.strip_prefix("required for `")?;
    let (subject, rest) = rest.split_once("` to implement `")?;
    let trait_ref = rest.strip_suffix('`')?;

    let (path, args_str) = split_generics(trait_ref)?;
    let args = split_top_level(args_str);

    match last_segment(path) {
        "IsProviderFor" => {
            // `IsProviderFor<Component, Context, Params?>` — the provider (`subject`)
            // implements the provider trait behind `Component` for `Context`.
            let component = last_segment(args.first()?.trim());
            let context = args.get(1)?.trim();
            let entry = names.get(component)?;
            Some(format!(
                "required for the provider `{subject}` to implement the provider trait `{}` for the context `{context}`",
                entry.provider
            ))
        }
        "CanUseComponent" => {
            // `CanUseComponent<Component, Params?>` — the context (`subject`) implements
            // the consumer trait behind `Component`.
            let component = last_segment(args.first()?.trim());
            let entry = names.get(component)?;
            Some(format!(
                "required for the context `{subject}` to implement the consumer trait `{}`",
                entry.consumer
            ))
        }
        _ => None,
    }
}

/// Rewrite the primary "the trait bound `…` is not satisfied" header a wiring failure opens
/// with — stamping it with its [CGP error code](crate::code), since both recognized forms are
/// classified CGP error classes — or return `None` to leave it unchanged:
///
/// - a `Self: CanUseComponent<Marker, Params?>` bound is a check-trait failure
///   ([`CONSUMER_TRAIT_UNIMPLEMENTED`]): the context fails to implement the consumer trait
///   behind the marker, so the message says exactly that.
/// - a `Self: IsProviderFor<Marker, Context, Params?>` bound is a provider check failure
///   ([`PROVIDER_TRAIT_UNIMPLEMENTED`]): the provider fails to implement the provider trait
///   behind the marker for the context.
///
/// A generic component carries its extra parameters after the marker (and, for the provider,
/// after the context), grouped in a tuple when there is more than one; those parameters are
/// reattached to the named trait so the rewritten message stays accurate —
/// `CanUseComponent<C, f64>` names `ConsumerTrait<f64>`, and `CanUseComponent<C, (u32, u64)>`
/// names `ConsumerTrait<u32, u64>`. As with [`rewrite_required_for`], a marker absent from
/// `names`, or any message that is not one of these two forms, is left unchanged.
pub fn rewrite_trait_bound(message: &str, names: &ComponentNameMap) -> Option<String> {
    let parsed = parse_trait_bound(message)?;
    let subject = parsed.subject;
    let tail = parsed.tail;
    let args = split_top_level(parsed.args);

    match parsed.trait_name {
        "CanUseComponent" if !args.is_empty() && !args[0].trim().is_empty() => {
            // `CanUseComponent<Marker, Params?>` — the consumer trait's generics are exactly
            // the component's extra parameters.
            let component = last_segment(args[0].trim());
            let entry = names.get(component)?;
            let generics = render_trait_generics(&[], &args[1..]);
            Some(format!(
                "[{CONSUMER_TRAIT_UNIMPLEMENTED}] the consumer trait `{}{generics}` is not implemented for context `{subject}`{tail}",
                entry.consumer
            ))
        }
        "IsProviderFor" if args.len() >= 2 => {
            // `IsProviderFor<Marker, Context, Params?>` — the context is named in prose, and
            // the provider trait's generics are the component's extra parameters.
            let component = last_segment(args[0].trim());
            let entry = names.get(component)?;
            let context = args[1].trim();
            let generics = render_trait_generics(&[], &args[2..]);
            Some(format!(
                "[{PROVIDER_TRAIT_UNIMPLEMENTED}] the provider trait `{}{generics}` with context `{context}` is not implemented for provider `{subject}`{tail}",
                entry.provider
            ))
        }
        _ => None,
    }
}

/// Render a trait's generic-argument list from a `leading` run (the provider's context, or
/// nothing for a consumer) followed by a component's extra parameters, and return it as
/// `<a, b, c>` — or the empty string when there are no arguments at all.
///
/// The extra parameters arrive as CGP groups them in `IsProviderFor`/`CanUseComponent`: a
/// single parameter appears bare, and two or more are wrapped in a tuple, which is unwrapped
/// here so the reattached list matches how the trait was written.
fn render_trait_generics(leading: &[&str], params: &[&str]) -> String {
    let mut generics: Vec<&str> = leading.to_vec();
    match params {
        [] => {}
        [grouped] => match grouped
            .trim()
            .strip_prefix('(')
            .and_then(|inner| inner.strip_suffix(')'))
        {
            Some(inner) => generics.extend(
                split_top_level(inner)
                    .into_iter()
                    .map(str::trim)
                    .filter(|param| !param.is_empty()),
            ),
            None => generics.push(grouped.trim()),
        },
        many => generics.extend(many.iter().map(|param| param.trim())),
    }

    if generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", generics.join(", "))
    }
}
