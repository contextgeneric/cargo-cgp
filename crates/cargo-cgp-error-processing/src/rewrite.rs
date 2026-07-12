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
//! Given a [`ComponentNameMap`] from a marker's name to the trait names behind it, this
//! rewrites both forms into ones that name the traits. The header rewrite also classifies
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
//! ([`rewrite_required_for`]) and the header rewrite ([`rewrite_trait_bound`]). The codes
//! belong on *main* messages only — the driver applies [`rewrite_trait_bound`] to a
//! diagnostic's header and [`rewrite_required_for`] to its sub-messages.
//!
//! This module lives in the rustc-free error-processing crate on purpose. The rewrite is a
//! plain string-to-string transform over the name map, so it is unit-tested on any toolchain
//! without a `TyCtxt`. The compiler-coupled half — walking the trait graph to *build* the map
//! — lives in the driver (`cargo-cgp-driver`), which hands the result in through
//! [`ComponentNameMap`]'s `fn`-pointer initializer. It is used by the driver's diagnostic
//! emitter, alongside the [`postprocess`](crate::postprocess) fallback transforms.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::code::{CONSUMER_TRAIT_UNIMPLEMENTED, PROVIDER_TRAIT_UNIMPLEMENTED};

/// The consumer and provider trait names behind one component marker, recovered from the
/// compiler. Keyed in the map by the marker's *full path* (e.g.
/// `my_crate::area::AreaCalculatorComponent`), so two markers that share a name in different
/// modules never collide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentTraitNames {
    /// The consumer trait a context implements (e.g. `CanCalculateArea`).
    pub consumer: String,
    /// The provider trait a provider implements (e.g. `AreaCalculator`).
    pub provider: String,
}

/// A lazily-built map from a component-marker's full path to the trait names behind it.
///
/// Building the map is expensive — the driver walks the whole trait graph through a
/// `TyCtxt` — so it is wrapped in a [`LazyLock`]: the initializer runs at most once, on the
/// first lookup, and *not at all* if no message is ever rewritten. That is what lets the
/// emitter drop a separate "does this diagnostic mention CGP?" pre-filter: the rewrite
/// functions look a marker up only after a message parses as a wiring form, so an ordinary
/// diagnostic never forces the map.
///
/// The map has two lookup paths for its two callers. The driver's typed resolver holds a
/// marker's `DefId` and looks it up by [`get_by_path`](Self::get_by_path) — an exact full-path
/// match that can never confuse two same-named markers from different modules. The text rewrite
/// has only the marker *name* rustc printed (rarely a full path), so it uses [`get`](Self::get),
/// which matches a key by its last path segment; that is inherently ambiguous when two markers
/// share a name, a residual the text form cannot resolve and the typed path avoids.
///
/// The initializer is a plain `fn` pointer, not a closure, so this type captures no compiler
/// state and can live in this rustc-free crate. The driver supplies a `fn` that reads the
/// `TyCtxt` from thread-local scope and builds the map (valid because a wiring message is
/// emitted during trait solving, when a `TyCtxt` is in scope); the tests supply a `fn` that
/// returns a fixed map.
pub struct ComponentNameMap {
    /// The underlying lazily-initialized map, keyed by each marker's full path. Public so the
    /// driver and tests can also construct one directly, though [`new`](Self::new) is the usual
    /// way.
    pub name_map: LazyLock<HashMap<String, ComponentTraitNames>>,
}

impl ComponentNameMap {
    /// Wrap a map initializer. `init` is a function pointer, so no state is captured here; it
    /// runs lazily on the first lookup.
    pub fn new(init: fn() -> HashMap<String, ComponentTraitNames>) -> Self {
        Self {
            name_map: LazyLock::new(init),
        }
    }

    /// Look up the trait names behind a marker by its full path — the exact, collision-free
    /// lookup the driver's typed resolver uses. Forces the lazy build on first use.
    pub fn get_by_path(&self, path: &str) -> Option<ComponentTraitNames> {
        self.name_map.get(path).cloned()
    }

    /// Look up the trait names behind a marker by its bare name, matching a full-path key by its
    /// last segment — the lookup the text rewrite uses, since rustc prints the marker unqualified.
    /// When two markers share a name (in different modules) the match is arbitrary; the text form
    /// cannot tell them apart, whereas the typed resolver keys on the full path via
    /// [`get_by_path`](Self::get_by_path). Forces the lazy build on first use.
    pub fn get(&self, name: &str) -> Option<ComponentTraitNames> {
        self.name_map
            .iter()
            .find(|(path, _)| last_segment(path) == name)
            .map(|(_, entry)| entry.clone())
    }
}

/// Rewrite one diagnostic message into its trait-named form, or return `None` to leave it
/// unchanged. This is the entry point the emitter drives over every message; it tries each
/// recognized CGP wiring form — the obligation-chain notes ([`rewrite_required_for`]), then
/// the primary `the trait bound … is not satisfied` header ([`rewrite_trait_bound`]).
///
/// Each form parses the message *before* consulting `names`, so a message that is not a CGP
/// wiring form returns `None` without ever calling [`ComponentNameMap::get`] — which is what
/// keeps the map's lazy initializer from running for an ordinary diagnostic.
pub fn rewrite_message(message: &str, names: &ComponentNameMap) -> Option<String> {
    rewrite_required_for(message, names).or_else(|| rewrite_trait_bound(message, names))
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

/// The parsed pieces of a "the trait bound `S: Trait<…>` is not satisfied" message — the form
/// rustc opens a trait-bound failure with. The driver uses the parse on its own to *classify*
/// a main message (is this a `CanUseComponent` failure? does the header already name the
/// leaf bound?) before deciding how to transform the diagnostic around it.
pub struct ParsedTraitBound<'a> {
    /// The bound's self type, e.g. `Rectangle` or `f64`.
    pub subject: &'a str,
    /// The whole bound as printed, e.g. `f64: std::cmp::Eq`.
    pub bound: &'a str,
    /// The trait path's last segment, e.g. `CanUseComponent` or `Eq`.
    pub trait_name: &'a str,
    /// The raw generic-argument text of the trait, e.g. `AreaCalculatorComponent, Rectangle`
    /// — empty when the trait has no generics.
    pub args: &'a str,
    /// Whatever follows `is not satisfied`, usually empty.
    pub tail: &'a str,
}

/// Parse a "the trait bound `…` is not satisfied" message, or return `None` for any other
/// message shape.
pub fn parse_trait_bound(message: &str) -> Option<ParsedTraitBound<'_>> {
    let rest = message.strip_prefix("the trait bound `")?;
    let (bound, tail) = rest.split_once("` is not satisfied")?;
    // `Self: Trait<…>` — the first `: ` separates the self type from the trait, and neither a
    // path (`::`) nor a generic argument inside the self type contains a colon *followed by a
    // space*, so this split cannot land inside either.
    let (subject, trait_ref) = bound.split_once(": ")?;

    let (path, args) = match split_generics(trait_ref) {
        Some((path, args)) => (path, args),
        None => (trait_ref, ""),
    };
    Some(ParsedTraitBound {
        subject,
        bound,
        trait_name: last_segment(path),
        args,
        tail,
    })
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
