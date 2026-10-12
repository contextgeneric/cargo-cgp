//! The rustc-free model and wording for a wiring lookup that never resolves.
//!
//! An `E0275` overflow at a `check_components!` entry means the compiler gave up resolving the
//! component's wiring. The driver's walk tells the two ways that happens apart: the lookup leads
//! back to an obligation it already met (a cycle, such as a component wired to `UseContext`), or
//! each step requires the consumer trait again for a larger type (a growing lookup, such as a
//! provider of `CanShow<T>` that requires `CanShow<Vec<T>>`). [`WiringOverflow`] records which, and
//! the provider step that causes it when the walk can name one; [`wiring_overflow_reason`] and
//! [`wiring_overflow_help`] word the headline's reason and the fix.

/// Why a wiring lookup never resolves, as the driver's walk found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WiringOverflow {
    /// The lookup leads back to an obligation it already met. `closing` is the provider step that
    /// requires it again, when that step is a provider's.
    Cycle { closing: Option<OverflowStep> },
    /// Each step requires the consumer trait again for a larger type, so the lookup grows until the
    /// compiler gives up. `larger` is the first larger requirement (`CanShow<Vec<u32>>`), and
    /// `growing` the provider step that requires it.
    Growing {
        larger: Option<String>,
        growing: Option<OverflowStep>,
    },
}

/// A provider step of a lookup that never resolves: `provider` implements `provider_trait` (its
/// arguments after the context, as `ErrorRaiser<String>`) for `context` by requiring `requires`
/// (a consumer trait with its arguments, as `CanRaiseError<String>`) of `context` again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverflowStep {
    pub provider: String,
    pub provider_trait: String,
    pub context: String,
    pub requires: String,
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
            step.provider, step.provider_trait, step.context, step.requires, step.context
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
            step.provider, step.provider_trait, step.context, step.requires, step.context
        ),
        WiringOverflow::Growing { growing: None, .. } => {
            "wire the component to a provider whose requirements do not grow with each step"
                .to_owned()
        }
    }
}
