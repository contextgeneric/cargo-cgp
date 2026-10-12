//! The `root cause:` note: merging causes, stacking chains, eliding shared tails, and the leaf
//! codes and signatures the note is built from.

use super::*;

#[test]
fn dependency_tree_leaf_codes_every_root_cause() {
    // Each root-cause leaf carries its own `CGP-E1xx` code as a tree entry.
    assert_eq!(
        dependency_tree_leaf(&field_leaf("name", "App", FieldIssue::Missing)),
        "[CGP-E110] missing field `name` on `App`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::MissingWiring {
            component: "BarProviderComponent".to_owned(),
            owner: "App".to_owned(),
        }),
        "[CGP-E113] context `App` does not contain any delegate entry for `BarProviderComponent`"
    );
    assert_eq!(
        dependency_tree_leaf(&field_leaf("name", "Person", FieldIssue::Present)),
        "[CGP-E111] accessor trait `HasField` with field `name` is not implemented for `Person`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::FieldTypeMismatch {
            name: "height".to_owned(),
            owner: "Rectangle".to_owned(),
            expected: "f64".to_owned(),
            expected_normalized: None,
            actual: "i32".to_owned(),
        }),
        "[CGP-E112] field `height` on `Rectangle` has type `i32`, but `f64` is required"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::MissingDispatchEntry {
            key: "Tagged<Bytes>".to_owned(),
            table: "SinkHandlers".to_owned(),
        }),
        "[CGP-E114] provider `SinkHandlers` does not contain any delegate entry for `Tagged<Bytes>`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::NotAProvider {
            provider: "QueryBalanceRequest".to_owned(),
            provider_trait: "ApiHandler".to_owned(),
        }),
        "[CGP-E115] the provider trait `ApiHandler` is not implemented for `QueryBalanceRequest`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::AssocTypeMismatch {
            assoc: "Error".to_owned(),
            trait_name: "HasErrorType".to_owned(),
            owner: "App".to_owned(),
            expected: "AppError".to_owned(),
            expected_normalized: None,
            actual: "String".to_owned(),
            component: Some("ErrorTypeProviderComponent".to_owned()),
        }),
        "[CGP-E116] abstract type `Error` of `HasErrorType` on `App` is `String`, but `AppError` is required"
    );
    // An ordinary bound keeps the compiler's phrasing, under its own code.
    assert_eq!(
        dependency_tree_leaf(&Leaf::Bound {
            summary: "f64: Eq".to_owned(),
        }),
        "[CGP-E117] the trait bound `f64: Eq` is not satisfied"
    );

    // The `root cause:` lead carries the same code as the leaf it names.
    assert_eq!(
        dependency_leaf_code(&field_leaf("name", "App", FieldIssue::Missing)),
        "CGP-E110"
    );
    assert_eq!(
        dependency_leaf_code(&Leaf::Bound {
            summary: "f64: Eq".to_owned(),
        }),
        "CGP-E117"
    );
}

#[test]
fn cause_signature_matches_re_reports_and_separates_distinct_failures() {
    let missing_name = || {
        one_path(
            field_leaf("name", "App", FieldIssue::Missing),
            vec![consumer("CanGreet", "App")],
        )
    };
    // Same context, consumer, and cause reached at two sites — a re-report — shares a signature,
    // even though the dependency paths differ (the path is not part of the signature).
    let at_check = cgp_resolved("App", &["CanGreet"], vec![missing_name()]);
    let at_call = cgp_resolved(
        "App",
        &["CanGreet"],
        vec![one_path(
            field_leaf("name", "App", FieldIssue::Missing),
            vec![consumer("CanGreet", "App"), trait_hop("Different", "App")],
        )],
    );
    assert_eq!(cause_signature(&at_check), cause_signature(&at_call));

    // A different consumer is a distinct failure — never merged, so no trait is hidden.
    let other_consumer = cgp_resolved("App", &["CanShout"], vec![missing_name()]);
    assert_ne!(cause_signature(&at_check), cause_signature(&other_consumer));

    // A different cause is a distinct fix — never merged.
    let other_cause = cgp_resolved(
        "App",
        &["CanGreet"],
        vec![one_path(
            field_leaf("age", "App", FieldIssue::Missing),
            vec![consumer("CanGreet", "App")],
        )],
    );
    assert_ne!(cause_signature(&at_check), cause_signature(&other_cause));
}

#[test]
fn wording_helpers_format_directly() {
    assert_eq!(quoted_list(&[]), "");
    assert_eq!(quoted_list(&["a".to_owned()]), "`a`");
    assert_eq!(
        quoted_list(&["a".to_owned(), "b".to_owned()]),
        "`a` and `b`"
    );
    assert_eq!(
        quoted_list(&["a".to_owned(), "b".to_owned(), "c".to_owned()]),
        "`a`, `b`, and `c`"
    );

    assert_eq!(
        field_mismatch_header("height", "Rectangle", "f64", None, "i32"),
        "[CGP-E004] expected a `height` field of type `f64` on `Rectangle`, but found `i32`"
    );

    // A required type that projects through the context's own wiring carries both forms: the
    // projection says where the requirement comes from, the reduction says what it resolves to.
    assert_eq!(
        field_mismatch_header(
            "database",
            "App",
            "Pool<<App as HasDbType>::Db>",
            Some("Pool<Postgres>"),
            "Pool<Sqlite>"
        ),
        "[CGP-E004] expected a `database` field of type `Pool<<App as HasDbType>::Db>` \
         (`Pool<Postgres>`) on `App`, but found `Pool<Sqlite>`"
    );

    // The leaf states the same requirement the same way, so the header and the tree cannot drift.
    assert_eq!(
        dependency_tree_leaf(&Leaf::FieldTypeMismatch {
            name: "database".to_owned(),
            owner: "App".to_owned(),
            expected: "Pool<<App as HasDbType>::Db>".to_owned(),
            expected_normalized: Some("Pool<Postgres>".to_owned()),
            actual: "Pool<Sqlite>".to_owned(),
        }),
        "[CGP-E112] field `database` on `App` has type `Pool<Sqlite>`, but \
         `Pool<<App as HasDbType>::Db>` (`Pool<Postgres>`) is required"
    );

    let cause = missing_field_cause();
    assert!(
        cause_note(&cause, None)
            .starts_with("root cause: [CGP-E110] missing field `height` on `Rectangle`")
    );
    assert!(derive_help_messages(std::slice::from_ref(&cause)).is_empty());
}

/// Two causes sharing a dependency prefix collapse into a single `root causes:` note: the shared
/// prefix appears once, each cause is listed up front, and the merged graph branches to the two
/// distinct leaves. A lone cause keeps its own `root cause:` note.
#[test]
fn merges_two_causes_sharing_a_root_into_one_note() {
    let causes = vec![
        field_cause("HasFirstName", "first_name"),
        field_cause("HasLastName", "last_name"),
    ];

    let notes = cause_notes(&causes, None);
    assert_eq!(notes.len(), 1, "the two causes merge into one note");
    assert_eq!(
        notes[0],
        "root causes:\n\
         \x20 - [CGP-E110] missing field `first_name` on `Person`\n\
         \x20 - [CGP-E110] missing field `last_name` on `Person`\n\
         this is required through the dependency chain:\n\
         \x20 [CGP-E101] consumer trait impl `CanGreet` for context `Person`\n\
         \x20 └─ [CGP-E102] provider trait impl `Greeter` with context `Person` for provider `GreetFullName`\n\
         \x20   ├─ [CGP-E104] trait impl `HasFirstName` for `Person`\n\
         \x20   │ └─ [CGP-E110] missing field `first_name` on `Person`\n\
         \x20   └─ [CGP-E104] trait impl `HasLastName` for `Person`\n\
         \x20     └─ [CGP-E110] missing field `last_name` on `Person`"
    );

    // A single cause is not merged — it keeps the `root cause:` (singular) form.
    let single = cause_notes(std::slice::from_ref(&causes[0]), None);
    assert_eq!(single.len(), 1);
    assert!(single[0].starts_with("root cause: [CGP-E110] missing field `first_name`"));
}

/// Two causes reached through *independent* consumer chains that share no node still collapse into
/// one note (the `parallel_consumers` shape), not two: the `root causes:` heading lists each distinct
/// leaf, and the graph renders each root chain stacked. This is the behavior the graph adds over the
/// old shared-prefix-only merge, which would have emitted two separate notes here.
#[test]
fn independent_root_causes_render_as_one_note_with_stacked_chains() {
    let causes = vec![
        one_path(
            field_leaf("height", "App", FieldIssue::Missing),
            vec![consumer("CanCalculateArea", "App")],
        ),
        one_path(
            field_leaf("width", "App", FieldIssue::Missing),
            vec![consumer("CanReportWidth", "App")],
        ),
    ];

    let notes = cause_notes(&causes, None);
    assert_eq!(
        notes.len(),
        1,
        "independent causes still merge into one note"
    );
    assert_eq!(
        notes[0],
        "root causes:\n\
         \x20 - [CGP-E110] missing field `height` on `App`\n\
         \x20 - [CGP-E110] missing field `width` on `App`\n\
         this is required through the dependency chain:\n\
         \x20 [CGP-E101] consumer trait impl `CanCalculateArea` for context `App`\n\
         \x20 └─ [CGP-E110] missing field `height` on `App`\n\
         \x20 [CGP-E101] consumer trait impl `CanReportWidth` for context `App`\n\
         \x20 └─ [CGP-E110] missing field `width` on `App`"
    );
}

/// A block whose every root an earlier block already drew still renders its own chain.
///
/// The cross-block elision exists to trim a long shared *tail* hanging under a block's own distinct
/// prefix. A block with no distinct prefix at all is the degenerate case rather than the case it was
/// built for: eliding there removes the whole derivation and leaves the block asserting a cause with
/// no account of where it came from. Such a block has already survived de-duplication, so it is a
/// distinct failure that merely runs through ground another one covered, and it earns its chain.
#[test]
fn a_wholly_contained_block_still_renders_its_chain() {
    let leaf = missing_field_leaf();
    // A wrapper block whose chain runs *through* the consumer's own chain.
    let wrapper = Cause {
        leaf: leaf.clone(),
        paths: vec![vec![
            trait_hop("CanCalculateAreaChecked", "Rectangle"),
            consumer("CanCalculateArea", "Rectangle"),
            ChainNode::Leaf(leaf.clone()),
        ]],
    };
    // The consumer block: a strict suffix of it, so every one of its roots was already drawn.
    let inner = one_path(
        leaf.clone(),
        vec![consumer("CanCalculateArea", "Rectangle")],
    );

    let mut seen = std::collections::HashSet::new();
    let first = PendingNote {
        causes: [wrapper].into_iter().collect(),
        header_leaf: None,
    }
    .render(&mut seen);
    let second = PendingNote {
        causes: [inner].into_iter().collect(),
        header_leaf: None,
    }
    .render(&mut seen);

    assert!(first[0].contains("this is required through the dependency chain:"));
    assert!(
        second[0].contains("this is required through the dependency chain:"),
        "the contained block keeps a chain rather than degenerating to a bare lead"
    );
    assert!(
        second[0].contains("consumer trait impl `CanCalculateArea` for context `Rectangle`"),
        "and renders its nodes in full"
    );
    assert!(
        !second[0].contains("(*)"),
        "rendered against a fresh set, so nothing points at another block"
    );
}

/// The partial case is untouched: a block with its own prefix still elides the shared tail, keeping
/// the `(*)` marker and the leaves beneath it.
#[test]
fn a_partly_contained_block_still_elides_its_shared_tail() {
    let leaf = missing_field_leaf();
    let inner = one_path(
        leaf.clone(),
        vec![consumer("CanCalculateArea", "Rectangle")],
    );
    let outer = Cause {
        leaf: leaf.clone(),
        paths: vec![vec![
            trait_hop("CanCalculateAreaChecked", "Rectangle"),
            consumer("CanCalculateArea", "Rectangle"),
            ChainNode::Leaf(leaf.clone()),
        ]],
    };

    let mut seen = std::collections::HashSet::new();
    let _first = PendingNote {
        causes: [inner].into_iter().collect(),
        header_leaf: None,
    }
    .render(&mut seen);
    let second = PendingNote {
        causes: [outer].into_iter().collect(),
        header_leaf: None,
    }
    .render(&mut seen);

    assert!(
        second[0].contains("(*)"),
        "the shared tail is still elided under the block's own prefix"
    );
    assert!(
        second[0].contains("missing field `height` on `Rectangle`"),
        "and the elided branch still bottoms out at the root cause"
    );
}
