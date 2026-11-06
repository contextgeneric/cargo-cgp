//! The shape of an unmet `DelegateComponent<Key>` leaf, decided over facts the driver reads from
//! the compiler, so the decision is tested without one.

use super::Leaf;

/// The facts [`delegate_leaf_shape`] reads about an unmet `DelegateComponent<Key>` leaf. Each is a
/// method, so the driver computes only the facts the decision reaches;
/// [`other_arguments`](Self::other_arguments) runs a solver query.
pub trait DelegateLeafFacts {
    /// Whether the leaf's `Self` is the context the walk resolves.
    fn on_context(&self) -> bool;

    /// Whether the key is a `PathCons` redirect path rather than a component marker.
    fn key_is_path(&self) -> bool;

    /// Whether the leaf has a parent obligation; only the walk's root has none.
    fn has_parent(&self) -> bool;

    /// Whether the owner was reached through its own delegation blanket although it implements the
    /// parent's trait. Read only when [`has_parent`](Self::has_parent) holds.
    fn blanket_dead_end(&self) -> bool;

    /// The leaf naming the arguments the owner implements the parent's trait for instead, when one
    /// argument explains the mismatch. Read only on a blanket dead-end.
    fn other_arguments(&self) -> Option<Leaf>;

    /// Whether the lookup is a dispatch into a separate table. Read only under a parent.
    fn dispatch_lookup(&self) -> bool;

    /// Whether the owner wires some other key, so it is a delegation table. Read only under a parent.
    fn owner_wires_other_key(&self) -> bool;
}

/// Which leaf an unmet `DelegateComponent<Key>` is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelegateLeafShape {
    /// A redirect path the context does not terminate.
    RedirectWiring,
    /// A component the context does not wire.
    Wiring,
    /// A provider that implements the parent's trait for other arguments, with its leaf.
    OtherArguments(Leaf),
    /// An entry missing from a delegation table other than the context.
    DispatchEntry,
    /// A type wired where a provider was expected.
    NotAProvider,
}

/// The shape of an unmet `DelegateComponent<Key>` leaf, or `None` when it is a routing dead-end
/// rather than a root cause. The cases, in the order they are decided:
///
/// - on the **context** itself: the component it does not wire, or for a redirect path, the path;
/// - with **no parent**: a dead-end, since every root of the walk is a consumer, provider, blanket,
///   or data obligation, so a `DelegateComponent` leaf below it has a parent;
/// - on a **provider reached through its own delegation blanket** although it implements the
///   parent's trait: the blanket was tried only because that impl did not unify, so the provider
///   implements the trait for other arguments when one argument explains the mismatch, and the
///   lookup is a dead-end otherwise, as for a pipeline stage fed an input the walk does not know.
///   A redirect path there is an aggregate provider's own `open`
///   redirect, so it names the entry missing from that provider's table in place of the other
///   arguments, and is dropped where those arguments are not found;
/// - on a **delegation table**, a redirect path, a dispatch lookup into a separate table, or an
///   owner that wires some other key: the entry the table lacks;
/// - on **any other type**: a type wired where a provider was expected.
///
/// Postcondition: [`other_arguments`](DelegateLeafFacts::other_arguments), the solver-backed fact,
/// is read at most once, and only on a blanket dead-end under a parent off the context.
pub fn delegate_leaf_shape(facts: &impl DelegateLeafFacts) -> Option<DelegateLeafShape> {
    if facts.on_context() {
        return Some(if facts.key_is_path() {
            DelegateLeafShape::RedirectWiring
        } else {
            DelegateLeafShape::Wiring
        });
    }
    if !facts.has_parent() {
        return None;
    }
    if facts.blanket_dead_end() {
        let other_arguments = facts.other_arguments()?;
        return Some(if facts.key_is_path() {
            DelegateLeafShape::DispatchEntry
        } else {
            DelegateLeafShape::OtherArguments(other_arguments)
        });
    }
    if facts.key_is_path() || facts.dispatch_lookup() || facts.owner_wires_other_key() {
        return Some(DelegateLeafShape::DispatchEntry);
    }
    Some(DelegateLeafShape::NotAProvider)
}
