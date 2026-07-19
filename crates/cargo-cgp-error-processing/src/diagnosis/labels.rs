//! Pure constructors for the dependency-tree node labels.
//!
//! Each function words one kind of dependency-chain hop and stamps it with its `CGP-E1xx`
//! code, so the driver's tree builder only supplies the names it reads off the compiler and
//! every label template lives here, rustc-free and unit-tested. The trait arguments arrive
//! already rendered (`CanCalculateArea<f64>`), since reading and resugaring them is the
//! driver's compiler-coupled half.

use crate::code::{
    DEP_CONSUMER_TRAIT_IMPL, DEP_FIELD_TRAIT_IMPL, DEP_PROVIDER_TRAIT_IMPL, DEP_REDIRECT_LOOKUP,
    DEP_TRAIT_IMPL,
};

/// The label of a hop through the context's own consumer-trait impl (`CGP-E101`).
pub fn consumer_impl_label(consumer: &str, context: &str) -> String {
    format!("[{DEP_CONSUMER_TRAIT_IMPL}] consumer trait impl `{consumer}` for context `{context}`")
}

/// The label of a hop through a provider's provider-trait impl (`CGP-E102`).
pub fn provider_impl_label(provider_trait: &str, context: &str, provider: &str) -> String {
    format!(
        "[{DEP_PROVIDER_TRAIT_IMPL}] provider trait impl `{provider_trait}` with context `{context}` for provider `{provider}`"
    )
}

/// The label of a hop through a `HasField` accessor impl (`CGP-E103`).
pub fn field_impl_label(field: &str, owner: &str) -> String {
    format!(
        "[{DEP_FIELD_TRAIT_IMPL}] field trait impl `HasField` with field `{field}` for `{owner}`"
    )
}

/// The label of a hop through a namespace/`open` `RedirectLookup` (`CGP-E104`).
pub fn redirect_label(path: &str, context: &str) -> String {
    format!("[{DEP_REDIRECT_LOOKUP}] redirect lookup to `{path}` in `{context}`")
}

/// The label of a hop through any other trait — a user capability trait, a wrapper the
/// programmer wrote, or an ordinary bound restated as an impl (`CGP-E105`).
pub fn trait_impl_label(trait_name: &str, self_ty: &str) -> String {
    format!("[{DEP_TRAIT_IMPL}] trait impl `{trait_name}` for `{self_ty}`")
}

/// Elide the generic arguments of a chain label whose quoted trait exactly repeats its
/// predecessor's — `Handler<Prog<…step list…>, _>` on first appearance, `Handler<…>` after — so a
/// dispatch chain whose every hop restates a program-sized `Code` type reads as its meaningful
/// steps. A pipeline's plumbing hops (`PipeHandlers` unfolding into `ComposeHandlers` into each
/// stage) all carry the *same* trait and parameters, so only the first spells them out; a hop
/// whose parameters genuinely change (a nested data type descending level by level) never
/// matches its predecessor and keeps its full form.
///
/// Each label's first back-quoted segment is the trait the hop is about (the shape every
/// [label template](self) shares), and only a segment carrying a generic list (`Trait<…>`)
/// participates; the comparison is against the predecessor's *original* segment, so a run of
/// three identical hops elides the second and third alike. The elision keeps merged trees
/// coherent: chains that share a prefix produce identical labels — and so identical elisions —
/// up to their divergence point.
pub fn elide_repeated_generics(labels: Vec<String>) -> Vec<String> {
    let mut previous: Option<String> = None;
    labels
        .into_iter()
        .map(|label| {
            let Some(quoted) = first_backquoted(&label) else {
                previous = None;
                return label;
            };
            let quoted = quoted.to_owned();
            let repeat = previous.as_deref() == Some(quoted.as_str());
            previous = Some(quoted.clone());
            let Some(open) = quoted.find('<') else {
                return label;
            };
            if !quoted.ends_with('>') || !repeat {
                return label;
            }
            let elided = format!("{}<…>", &quoted[..open]);
            label.replacen(&format!("`{quoted}`"), &format!("`{elided}`"), 1)
        })
        .collect()
}

/// The content of the first back-quoted `` `…` `` segment of a label, or `None` when the label
/// carries none.
fn first_backquoted(label: &str) -> Option<&str> {
    let start = label.find('`')? + 1;
    let len = label[start..].find('`')?;
    Some(&label[start..start + len])
}
