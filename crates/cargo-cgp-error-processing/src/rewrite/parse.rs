//! Parsing rustc's "the trait bound `…` is not satisfied" header.

use crate::rewrite::text::{last_segment, split_generics};

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
