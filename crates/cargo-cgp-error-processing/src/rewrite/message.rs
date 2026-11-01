//! Rewriting CGP wiring messages to name the traits behind a component marker.
//!
//! rustc reports a CGP wiring failure in terms of the internal wiring traits, which name the
//! *component marker* but not the consumer or provider trait a reader thinks in, both in the
//! primary header the error opens with and in the obligation-chain notes:
//!
//! ```text
//! the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle>` is not satisfied
//! required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`
//! required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`
//! ```
//!
//! Given a [`ComponentNameMap`] from a marker's name to the trait names behind it, these
//! rewrites turn both forms into ones that name the traits. The header rewrite also classifies
//! the message with its [CGP error code](crate::code), since an unsatisfied `IsProviderFor` bound
//! is a recognized CGP error class:
//!
//! ```text
//! [CGP-E002] the provider trait `AreaCalculator` with context `Rectangle` is not implemented for provider `RectangleArea`
//! required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle`
//! required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`
//! ```
//!
//! [`rewrite_message`] is the entry point; it dispatches to the note-form rewrite
//! ([`rewrite_required_for`]) and the header rewrite ([`rewrite_trait_bound`]).
//! The codes belong on *main* messages only: the driver applies the full dispatch to a
//! diagnostic's header and [`rewrite_required_for`] to its sub-messages.
//!
//! Each form parses the message *before* consulting `names`, so a message that is not a CGP
//! wiring form returns `None` without ever forcing the map's lazy initializer.

use crate::code::PROVIDER_TRAIT_UNIMPLEMENTED;
use crate::naming::is_reserved_name;
use crate::rewrite::names::ComponentNameMap;
use crate::rewrite::parse::parse_trait_bound;
use crate::rewrite::text::{last_segment, split_generics, split_top_level};

/// Rewrite one diagnostic message into its trait-named form, or return `None` to leave it
/// unchanged. This is the entry point the emitter drives over every message; it tries each
/// recognized CGP wiring form: the obligation-chain notes ([`rewrite_required_for`]) and the
/// primary `the trait bound … is not satisfied` header ([`rewrite_trait_bound`]). The
/// `overflow evaluating the requirement …` header has its own entry point,
/// [`rewrite_wiring_overflow`](crate::rewrite_wiring_overflow), since its wording depends on why the lookup never resolves.
pub fn rewrite_message(message: &str, names: &ComponentNameMap) -> Option<String> {
    rewrite_required_for(message, names).or_else(|| rewrite_trait_bound(message, names))
}

/// Rewrite one `required for … to implement …` obligation-chain note into its trait-named
/// form, or return `None` to leave the message untouched.
///
/// Returns `None` for any message that is not one of the two recognized note forms, and for a
/// recognized form whose component marker is absent from `names`, so a message is only ever
/// rewritten when the replacement names are known. A type is called a context only when it is
/// one: a generated parameter such as `__Context__` stands for every context a provider is generic
/// over, so the provider form leaves its context out and the consumer form names it plainly.
pub fn rewrite_required_for(message: &str, names: &ComponentNameMap) -> Option<String> {
    let rest = message.strip_prefix("required for `")?;
    let (subject, rest) = rest.split_once("` to implement `")?;
    let trait_ref = rest.strip_suffix('`')?;

    let (path, args_str) = split_generics(trait_ref)?;
    let args = split_top_level(args_str);

    match last_segment(path) {
        "IsProviderFor" => {
            // `IsProviderFor<Component, Context, Params?>`: the provider (`subject`)
            // implements the provider trait behind `Component` for `Context`.
            let component = args.first()?.trim();
            let context = args.get(1)?.trim();
            let entry = names.get(component)?;
            if is_reserved_name(context) {
                return Some(format!(
                    "required for the provider `{subject}` to implement the provider trait `{}`",
                    entry.provider
                ));
            }
            Some(format!(
                "required for the provider `{subject}` to implement the provider trait `{}` for the context `{context}`",
                entry.provider
            ))
        }
        "CanUseComponent" => {
            // `CanUseComponent<Component, Params?>`: the context (`subject`) implements
            // the consumer trait behind `Component`.
            let component = args.first()?.trim();
            let entry = names.get(component)?;
            let subject = if is_reserved_name(subject) {
                format!("`{subject}`")
            } else {
                format!("the context `{subject}`")
            };
            Some(format!(
                "required for {subject} to implement the consumer trait `{}`",
                entry.consumer
            ))
        }
        _ => None,
    }
}

/// Rewrite the primary "the trait bound `…` is not satisfied" header a provider check failure
/// opens with, a `Self: IsProviderFor<Marker, Context, Params?>` bound, stamping it with
/// [`PROVIDER_TRAIT_UNIMPLEMENTED`]: the provider fails to implement the provider trait behind the
/// marker for the context. Return `None` to leave any other header unchanged; a check's
/// `CanUseComponent` header is the typed resolver's to word.
///
/// A generic component carries its extra parameters after the context, grouped in a tuple when
/// there is more than one; those parameters are reattached to the named trait so the rewritten
/// message stays accurate: `IsProviderFor<C, Ctx, f64>` names `ProviderTrait<f64>`, and
/// `IsProviderFor<C, Ctx, (u32, u64)>` names `ProviderTrait<u32, u64>`. As with
/// [`rewrite_required_for`], a marker absent from `names` is left unchanged.
pub fn rewrite_trait_bound(message: &str, names: &ComponentNameMap) -> Option<String> {
    let parsed = parse_trait_bound(message)?;
    let subject = parsed.subject;
    let tail = parsed.tail;
    let args = split_top_level(parsed.args);

    match parsed.trait_name {
        "IsProviderFor" if args.len() >= 2 => {
            // `IsProviderFor<Marker, Context, Params?>`: the context is named in prose, and
            // the provider trait's generics are the component's extra parameters.
            let component = args[0].trim();
            let entry = names.get(component)?;
            let context = args[1].trim();
            let generics = render_trait_generics(&[], &args[2..], entry.params);
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
/// `<a, b, c>`, or the empty string when there are no arguments at all.
///
/// The extra parameters arrive as CGP groups them in `IsProviderFor`/`CanUseComponent`: a
/// single parameter appears bare, and two or more are wrapped in a tuple, which is unwrapped
/// here so the reattached list matches how the trait was written. `arity` is the component's
/// parameter count: a component with exactly one parameter keeps it whole, even when that
/// parameter is itself a tuple.
pub fn render_trait_generics(leading: &[&str], params: &[&str], arity: usize) -> String {
    let mut generics: Vec<&str> = leading.to_vec();
    match params {
        [] => {}
        [single] if arity == 1 => generics.push(single.trim()),
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
