//! Merging duplicate causes by leaf, over hand-built `Cause` values.

use cargo_cgp_error_processing::{
    Cause, ChainNode, DepNode, FieldIssue, Leaf, coalesce_underived_fields, merge_causes_by_leaf,
    prepend_hop, root_cause_lead,
};

/// A cause reaching `owner`'s `name` field through the named consumer — the shape one member of a
/// coalesced block contributes, each member reaching the shared cause down its own chain.
fn field_cause_via(consumer: &str, name: &str, owner: &str, issue: FieldIssue) -> Cause {
    let leaf = Leaf::Field {
        name: name.to_owned(),
        owner: owner.to_owned(),
        issue,
    };
    Cause {
        paths: vec![vec![
            ChainNode::Hop(DepNode::Consumer {
                trait_ref: consumer.to_owned(),
                context: owner.to_owned(),
            }),
            ChainNode::Leaf(leaf.clone()),
        ]],
        leaf,
    }
}

#[test]
fn the_same_leaf_reached_by_several_consumers_merges_into_one_cause() {
    let causes = vec![
        field_cause_via("CanFoo", "name", "App", FieldIssue::Present),
        field_cause_via("CanBar", "name", "App", FieldIssue::Present),
        field_cause_via("CanBaz", "name", "App", FieldIssue::Present),
    ];

    let merged = merge_causes_by_leaf(&causes);

    // One cause per distinct leaf, holding every path that reaches it.
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].leaf, causes[0].leaf);
    assert_eq!(merged[0].paths.len(), 3);
}

#[test]
fn merging_first_is_what_keeps_one_underived_field_from_reading_as_three() {
    let causes = vec![
        field_cause_via("CanFoo", "name", "App", FieldIssue::Present),
        field_cause_via("CanBar", "name", "App", FieldIssue::Present),
        field_cause_via("CanBaz", "name", "App", FieldIssue::Present),
    ];

    // Without the merge, three copies of one underived field read as three fields.
    let unmerged = coalesce_underived_fields(&causes);
    assert_eq!(
        root_cause_lead(&unmerged[0].leaf),
        "accessor trait `HasField` is not implemented for the fields `name`, `name`, and `name` of `App`"
    );

    // With it, the lone underived field keeps its single-field wording.
    let coalesced = coalesce_underived_fields(&merge_causes_by_leaf(&causes));
    assert_eq!(coalesced.len(), 1);
    assert_eq!(
        root_cause_lead(&coalesced[0].leaf),
        "accessor trait `HasField` with field `name` is not implemented for `App`"
    );
}

#[test]
fn every_route_to_a_shared_cause_survives() {
    let causes = vec![
        field_cause_via("CanGreet", "name", "App", FieldIssue::Missing),
        field_cause_via("CanBidFarewell", "name", "App", FieldIssue::Missing),
    ];

    let merged = merge_causes_by_leaf(&causes);

    // De-duplicating by leaf while discarding the duplicate's paths would leave the second
    // consumer named in the header with no chain to show for it.
    assert_eq!(merged.len(), 1);
    assert_eq!(
        merged[0].paths,
        [causes[0].paths[0].clone(), causes[1].paths[0].clone()]
    );
}

#[test]
fn distinct_leaves_stay_apart() {
    let causes = vec![
        field_cause_via("CanFoo", "first_name", "App", FieldIssue::Present),
        field_cause_via("CanBar", "last_name", "App", FieldIssue::Present),
    ];

    let merged = merge_causes_by_leaf(&causes);

    // Two genuinely different underived fields are two causes, so they still coalesce into the
    // multi-field lead that names both — the merge only removes repeats of one leaf.
    assert_eq!(merged.len(), 2);
    let coalesced = coalesce_underived_fields(&merged);
    assert_eq!(
        root_cause_lead(&coalesced[0].leaf),
        "accessor trait `HasField` is not implemented for the fields `first_name` and `last_name` of `App`"
    );
}

#[test]
fn an_exact_repeat_of_a_path_is_dropped() {
    let cause = field_cause_via("CanFoo", "name", "App", FieldIssue::Present);
    let causes = vec![cause.clone(), cause.clone()];

    let merged = merge_causes_by_leaf(&causes);

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].paths, cause.paths);
}

#[test]
fn merging_leaves_a_well_formed_cause_list_untouched() {
    let causes = vec![
        field_cause_via("CanFoo", "name", "App", FieldIssue::Missing),
        field_cause_via("CanBar", "age", "App", FieldIssue::Present),
    ];

    assert_eq!(merge_causes_by_leaf(&causes), causes);
}

/// The hop an anchor heads a recovered chain with — the wrapper trait the programmer wrote.
fn wrapper_hop() -> DepNode {
    DepNode::Trait {
        trait_ref: "CanGreetChecked".to_owned(),
        self_ty: "App".to_owned(),
    }
}

#[test]
fn prepending_a_hop_heads_every_path_and_merges_by_leaf() {
    // Two causes naming one leaf, as two supertraits descending to the same cause produce.
    let causes = vec![
        field_cause_via("CanFoo", "name", "App", FieldIssue::Missing),
        field_cause_via("CanBar", "name", "App", FieldIssue::Missing),
    ];

    let headed = prepend_hop(&causes, &wrapper_hop());

    // Merged to one cause holding both routes, each now headed by the wrapper.
    assert_eq!(headed.len(), 1);
    assert_eq!(headed[0].paths.len(), 2);
    for path in &headed[0].paths {
        assert_eq!(path.first(), Some(&ChainNode::Hop(wrapper_hop())));
    }
}

#[test]
fn prepending_a_hop_leaves_distinct_leaves_apart() {
    let causes = vec![
        field_cause_via("CanFoo", "name", "App", FieldIssue::Missing),
        field_cause_via("CanFoo", "age", "App", FieldIssue::Missing),
    ];

    let headed = prepend_hop(&causes, &wrapper_hop());

    assert_eq!(headed.len(), 2);
    assert_eq!(headed[0].leaf, causes[0].leaf);
    assert_eq!(headed[1].leaf, causes[1].leaf);
}

/// Prepending is order-independent with respect to the merge, which is why the two can be one
/// operation: a single constant hop cannot change any leaf's identity, so heading first and merging
/// first agree. The two anchors used to rely on that without stating it.
#[test]
fn prepending_and_merging_commute() {
    let causes = vec![
        field_cause_via("CanFoo", "name", "App", FieldIssue::Missing),
        field_cause_via("CanBar", "name", "App", FieldIssue::Missing),
    ];

    let merged_first = prepend_hop(&merge_causes_by_leaf(&causes), &wrapper_hop());

    assert_eq!(prepend_hop(&causes, &wrapper_hop()), merged_first);
}
