//! The rustc-free model and wording for a wiring lookup that never resolves.
//!
//! An `E0275` overflow at a `check_components!` entry means the compiler gave up resolving the
//! component's wiring. The driver's walk tells the two ways that happens apart: the lookup leads
//! back to an obligation it already met (a cycle, such as a component wired to `UseContext`), or
//! each step requires the consumer trait again for a larger type (a growing lookup, such as a
//! provider of `CanShow<T>` that requires `CanShow<Vec<T>>`). [`WiringOverflow`] records which, and
//! the provider step that causes it when the walk can name one; [`rewrite_wiring_overflow`] and
//! [`wiring_overflow_reason`] word the headline, and [`wiring_overflow_help`] the fix.

use crate::code::{GROWING_LOOKUP, WIRING_CYCLE};
use crate::rewrite::{
    ComponentNameMap, last_segment, render_trait_generics, split_generics, split_top_level,
};
use crate::signals::overflow_requirement_rest;

/// Why a wiring lookup never resolves, as the driver's walk found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WiringOverflow {
    /// The lookup leads back to an obligation it already met. `closing` is the provider step that
    /// requires it again, when a wiring entry chose that provider.
    Cycle {
        /// The provider step that requires the obligation again, when a wiring entry chose it.
        closing: Option<OverflowStep>,
    },
    /// Each step requires the consumer trait again for a larger type, so the lookup grows until the
    /// compiler gives up. `larger` is the first larger requirement (`CanShow<Vec<u32>>`), or the
    /// larger type itself when the consumer trait is required of a type that grows (`Vec<u32>`),
    /// and `growing` the provider step that requires it, when a wiring entry chose that provider.
    Growing {
        /// The first larger requirement, or the larger type when the type the trait is required of
        /// grows.
        larger: Option<String>,
        /// The provider step that requires it, when a wiring entry chose that provider.
        growing: Option<OverflowStep>,
    },
}

/// A provider step of a lookup that never resolves: `provider` implements `provider_trait` (its
/// arguments after the context, as `ErrorRaiser<String>`) for `context` by requiring `requires`
/// (a consumer trait with its arguments, as `CanRaiseError<String>`) of `required_of`: the context
/// again, or a larger type when the lookup grows in the type the trait is required of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverflowStep {
    /// The provider the wiring chose.
    pub provider: String,
    /// The provider trait it implements, with its arguments after the context.
    pub provider_trait: String,
    /// The context it implements the provider trait for.
    pub context: String,
    /// The consumer trait it requires, with its arguments.
    pub requires: String,
    /// The type it requires the consumer trait of: the context again, or a larger type.
    pub required_of: String,
}

/// The clause that ends a never-resolving headline, after "never resolves, because": the lookup
/// leads back to itself, or grows with each step (naming the first larger requirement when known).
pub fn wiring_overflow_reason(overflow: &WiringOverflow) -> String {
    match overflow {
        WiringOverflow::Cycle { .. } => "the lookup leads back to itself".to_owned(),
        WiringOverflow::Growing {
            larger: Some(larger),
            ..
        } => format!("each step requires it for a larger type (`{larger}`, …)"),
        WiringOverflow::Growing { larger: None, .. } => {
            "each step requires it for a larger type".to_owned()
        }
    }
}

/// The `help` for a lookup that never resolves: replace the provider whose requirement leads back
/// or grows, naming it and what it requires when the walk found that step, and otherwise say what
/// the replacement must not do without blaming an entry.
pub fn wiring_overflow_help(overflow: &WiringOverflow) -> String {
    match overflow {
        WiringOverflow::Cycle {
            closing: Some(step),
        } => format!(
            "wire a provider in place of `{}`, which provides `{}` for `{}` by requiring `{}` on `{}` again",
            step.provider, step.provider_trait, step.context, step.requires, step.required_of
        ),
        WiringOverflow::Cycle { closing: None } => {
            "wire the component to a provider that does not require the consumer trait it provides"
                .to_owned()
        }
        WiringOverflow::Growing {
            growing: Some(step),
            ..
        } => format!(
            "wire a provider in place of `{}`, which provides `{}` for `{}` by requiring `{}` on `{}`, a larger type at every step",
            step.provider, step.provider_trait, step.context, step.requires, step.required_of
        ),
        WiringOverflow::Growing { growing: None, .. } => {
            "wire the component to a provider whose requirements do not grow with each step"
                .to_owned()
        }
    }
}

/// Rewrite the `overflow evaluating the requirement …` header (`E0275`) of a wiring lookup that
/// never resolves, or return `None` for any other overflow. Only an overflow whose requirement is a
/// `Self: CanUseComponent<Marker, …>` bound, as a check reports it, or a consumer trait the map
/// knows, as a call reports it (`Person: CanGreet`), is a wiring lookup; `overflow` says how it
/// never resolves, as the driver's walk found it: a cycle ([`WIRING_CYCLE`]) or a growing lookup
/// ([`GROWING_LOOKUP`]). The fix rides in [`wiring_overflow_help`], kept out of the header so the
/// headline stays one sentence.
pub fn rewrite_wiring_overflow(
    message: &str,
    names: &ComponentNameMap,
    overflow: &WiringOverflow,
) -> Option<String> {
    let rest = overflow_requirement_rest(message)?;
    let (bound, _tail) = rest.split_once('`')?;
    let (subject, trait_ref) = bound.split_once(": ")?;
    let (path, args_str) = split_generics(trait_ref).unwrap_or((trait_ref, ""));
    let consumer = if last_segment(path) == "CanUseComponent" {
        let args = split_top_level(args_str);
        let component = args.first()?.trim();
        let entry = names.get(component)?;
        let generics = render_trait_generics(&[], &args[1..], entry.params);
        format!("{}{generics}", entry.consumer)
    } else {
        let name = last_segment(path);
        if !names.is_consumer(name) {
            return None;
        }
        if args_str.is_empty() {
            name.to_owned()
        } else {
            format!("{name}<{args_str}>")
        }
    };
    let code = match overflow {
        WiringOverflow::Cycle { .. } => WIRING_CYCLE,
        WiringOverflow::Growing { .. } => GROWING_LOOKUP,
    };
    Some(format!(
        "[{code}] the wiring for the consumer trait `{consumer}` on context `{subject}` never resolves, because {}",
        wiring_overflow_reason(overflow)
    ))
}
