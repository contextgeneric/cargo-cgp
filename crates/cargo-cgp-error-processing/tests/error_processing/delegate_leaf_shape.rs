//! The shape of an unmet `DelegateComponent` leaf, decided over a stand-in for the compiler's facts.
//!
//! The specification is the case list on `delegate_leaf_shape`, the decision the driver's
//! `delegate_component_leaf` made before it moved here. The domain is six booleans and an optional
//! finding, so each property is checked over every combination its precondition admits.

use std::cell::Cell;

use cargo_cgp_error_processing::{DelegateLeafFacts, DelegateLeafShape, Leaf, delegate_leaf_shape};

/// A stand-in for the compiler: each fact is a field, and the solver-backed finder counts its reads.
struct Facts {
    on_context: bool,
    key_is_path: bool,
    has_parent: bool,
    blanket_dead_end: bool,
    other_arguments: Option<Leaf>,
    dispatch_lookup: bool,
    owner_wires_other_key: bool,
    other_arguments_reads: Cell<usize>,
}

impl DelegateLeafFacts for Facts {
    fn on_context(&self) -> bool {
        self.on_context
    }

    fn key_is_path(&self) -> bool {
        self.key_is_path
    }

    fn has_parent(&self) -> bool {
        self.has_parent
    }

    fn blanket_dead_end(&self) -> bool {
        self.blanket_dead_end
    }

    fn other_arguments(&self) -> Option<Leaf> {
        self.other_arguments_reads
            .set(self.other_arguments_reads.get() + 1);
        self.other_arguments.clone()
    }

    fn dispatch_lookup(&self) -> bool {
        self.dispatch_lookup
    }

    fn owner_wires_other_key(&self) -> bool {
        self.owner_wires_other_key
    }
}

/// A leaf the other-arguments finder could return; the decision passes it through unread.
fn other_arguments_leaf() -> Leaf {
    Leaf::Bound {
        summary: "GreetPerson: Greeter<App>".to_owned(),
    }
}

/// Every combination of the six booleans and of a found or absent other-arguments leaf.
fn all_facts() -> impl Iterator<Item = Facts> {
    (0u8..64).flat_map(|bits| {
        [None, Some(other_arguments_leaf())].map(move |other_arguments| Facts {
            on_context: bits & 1 != 0,
            key_is_path: bits & 2 != 0,
            has_parent: bits & 4 != 0,
            blanket_dead_end: bits & 8 != 0,
            other_arguments,
            dispatch_lookup: bits & 16 != 0,
            owner_wires_other_key: bits & 32 != 0,
            other_arguments_reads: Cell::new(0),
        })
    })
}

// Property: for all facts with `on_context`, the shape is `RedirectWiring` when the key is a path
// and `Wiring` otherwise, whatever the other facts say.
#[test]
fn on_the_context_the_key_alone_decides() {
    for facts in all_facts().filter(|facts| facts.on_context) {
        let expected = if facts.key_is_path {
            DelegateLeafShape::RedirectWiring
        } else {
            DelegateLeafShape::Wiring
        };
        assert_eq!(delegate_leaf_shape(&facts), Some(expected));
    }
}

// Property: for all facts off the context with no parent, the leaf is a dead-end (`None`).
#[test]
fn off_the_context_without_a_parent_is_a_dead_end() {
    for facts in all_facts().filter(|facts| !facts.on_context && !facts.has_parent) {
        assert_eq!(delegate_leaf_shape(&facts), None);
    }
}

// Property: for all facts off the context, under a parent, on a blanket dead-end, the shape is
// `None` when no other arguments explain the mismatch, a `DispatchEntry` for a path key, and the
// other-arguments leaf otherwise, whatever the table facts say.
#[test]
fn a_blanket_dead_end_names_the_other_arguments() {
    for facts in
        all_facts().filter(|facts| !facts.on_context && facts.has_parent && facts.blanket_dead_end)
    {
        let expected = facts.other_arguments.clone().map(|other_arguments| {
            if facts.key_is_path {
                DelegateLeafShape::DispatchEntry
            } else {
                DelegateLeafShape::OtherArguments(other_arguments)
            }
        });
        assert_eq!(delegate_leaf_shape(&facts), expected);
    }
}

// Property: for all facts off the context, under a parent, and not on a blanket dead-end, the shape
// is `DispatchEntry` when the owner is a delegation table (a path key, a dispatch lookup, or an
// owner wiring another key) and `NotAProvider` otherwise.
#[test]
fn a_delegation_table_misses_an_entry_and_anything_else_is_not_a_provider() {
    for facts in
        all_facts().filter(|facts| !facts.on_context && facts.has_parent && !facts.blanket_dead_end)
    {
        let expected = if facts.key_is_path || facts.dispatch_lookup || facts.owner_wires_other_key
        {
            DelegateLeafShape::DispatchEntry
        } else {
            DelegateLeafShape::NotAProvider
        };
        assert_eq!(delegate_leaf_shape(&facts), Some(expected));
    }
}

// Concern: the other-arguments finder runs a solver query in the driver, so it must be read at most
// once, and only on a blanket dead-end under a parent off the context.
// Handled: the decision reads it in that branch alone.
#[test]
fn the_solver_backed_finder_runs_only_on_a_blanket_dead_end() {
    for facts in all_facts() {
        delegate_leaf_shape(&facts);
        let reached = !facts.on_context && facts.has_parent && facts.blanket_dead_end;
        assert_eq!(facts.other_arguments_reads.get(), usize::from(reached));
    }
}
