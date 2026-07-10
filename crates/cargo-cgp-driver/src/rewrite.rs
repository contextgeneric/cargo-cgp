//! The message rewrite — the pure half of the driver's diagnostic transform.
//!
//! rustc reports a CGP wiring failure through obligation-chain notes phrased in terms of
//! the internal wiring traits, which name the *component marker* but not the consumer or
//! provider trait a reader thinks in:
//!
//! ```text
//! required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`
//! required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`
//! ```
//!
//! Given a map from a component marker's name to the consumer and provider trait names
//! behind it (built from the compiler in [`crate::component_map`]), this rewrites those two
//! note forms into ones that name the traits:
//!
//! ```text
//! required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle`
//! required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`
//! ```
//!
//! This module is deliberately free of any compiler dependency: it is a string-to-string
//! function over the name map, so it can be unit-tested without a `TyCtxt`. The compiler
//! query that produces the map is the other half, in [`crate::component_map`].

use std::collections::HashMap;

/// The consumer and provider trait names behind one component marker, recovered from the
/// compiler. Keyed in the map by the marker's own name (e.g. `AreaCalculatorComponent`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentTraitNames {
    /// The consumer trait a context implements (e.g. `CanCalculateArea`).
    pub consumer: String,
    /// The provider trait a provider implements (e.g. `AreaCalculator`).
    pub provider: String,
}

/// Rewrite one `required for … to implement …` note into its trait-named form, or return
/// `None` to leave the message untouched.
///
/// Returns `None` for any message that is not one of the two recognized wiring-note forms,
/// and for a recognized form whose component marker is absent from `names` — so a message
/// is only ever rewritten when the replacement names are known.
pub fn rewrite_required_for(
    message: &str,
    names: &HashMap<String, ComponentTraitNames>,
) -> Option<String> {
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

/// Whether a message is worth handing to [`rewrite_required_for`] — a cheap pre-filter that
/// avoids building the name map for a diagnostic that carries no CGP wiring note.
pub fn is_wiring_note(message: &str) -> bool {
    message.starts_with("required for `")
        && message.contains("` to implement `")
        && (message.contains("IsProviderFor<") || message.contains("CanUseComponent<"))
}

/// Split `Path<args>` into its path (`Path`) and the raw argument text (`args`), requiring
/// the string to be a single generic application closed by a trailing `>`.
fn split_generics(s: &str) -> Option<(&str, &str)> {
    let lt = s.find('<')?;
    let args = s.strip_suffix('>')?.get(lt + 1..)?;
    Some((&s[..lt], args))
}

/// The last `::`-separated segment of a path, so `cgp::prelude::IsProviderFor` becomes
/// `IsProviderFor` and a component key matches the compiler's unqualified item name.
fn last_segment(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path).trim()
}

/// Split a generic argument list on its top-level commas, so a nested `<…>`, `(…)`, or
/// `[…]` argument (e.g. a generic context or a `Params` tuple) is kept whole.
fn split_top_level(args: &str) -> Vec<&str> {
    let mut depth: i32 = 0;
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, c) in args.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&args[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&args[start..]);
    parts
}
