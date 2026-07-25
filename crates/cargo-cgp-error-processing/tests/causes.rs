//! The `Causes` set and the *one cause per distinct leaf* invariant it enforces by construction.

use cargo_cgp_error_processing::{
    Cause, Causes, ChainNode, DepNode, FieldIssue, Leaf, coalesce_underived_fields, root_cause_lead,
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

/// The hop an anchor heads a recovered chain with — the wrapper trait the programmer wrote.
fn wrapper_hop() -> DepNode {
    DepNode::Trait {
        trait_ref: "CanGreetChecked".to_owned(),
        self_ty: "App".to_owned(),
    }
}

#[test]
fn the_same_leaf_reached_by_several_consumers_becomes_one_cause() {
    let causes: Causes = [
        field_cause_via("CanFoo", "name", "App", FieldIssue::Present),
        field_cause_via("CanBar", "name", "App", FieldIssue::Present),
        field_cause_via("CanBaz", "name", "App", FieldIssue::Present),
    ]
    .into_iter()
    .collect();

    // One cause per distinct leaf, holding every path that reaches it.
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].paths.len(), 3);
}

/// The invariant exists for this: a duplicate-leaf list makes a downstream reader count one mistake
/// several times, most visibly as a lead naming one underived field once per consumer that reads it.
/// The type is what makes that unreachable — there is no constructor that produces the bad list.
#[test]
fn one_underived_field_never_reads_as_three() {
    let causes: Causes = [
        field_cause_via("CanFoo", "name", "App", FieldIssue::Present),
        field_cause_via("CanBar", "name", "App", FieldIssue::Present),
        field_cause_via("CanBaz", "name", "App", FieldIssue::Present),
    ]
    .into_iter()
    .collect();

    let coalesced = coalesce_underived_fields(&causes);
    assert_eq!(
        root_cause_lead(&coalesced[0].leaf),
        "accessor trait `HasField` with field `name` is not implemented for `App`"
    );
}

#[test]
fn distinct_leaves_stay_apart() {
    let causes: Causes = [
        field_cause_via("CanFoo", "name", "App", FieldIssue::Missing),
        field_cause_via("CanFoo", "age", "App", FieldIssue::Missing),
    ]
    .into_iter()
    .collect();

    assert_eq!(causes.len(), 2);
}

#[test]
fn an_exact_repeat_of_a_path_is_dropped() {
    let cause = field_cause_via("CanFoo", "name", "App", FieldIssue::Missing);
    let causes: Causes = [cause.clone(), cause].into_iter().collect();

    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].paths.len(), 1);
}

#[test]
fn sub_chains_group_by_leaf() {
    let first = field_cause_via("CanFoo", "name", "App", FieldIssue::Missing);
    let second = field_cause_via("CanBar", "name", "App", FieldIssue::Missing);

    let causes = Causes::from_sub_chains([
        (first.leaf.clone(), first.paths[0].clone()),
        (second.leaf.clone(), second.paths[0].clone()),
    ]);

    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].paths.len(), 2);
}

/// Union is what the emitter's coalesced block and the by-component use-site anchor need: total, and
/// normalizing, so a cause every member shares is stated once while keeping every member's route.
#[test]
fn union_folds_shared_causes_and_keeps_every_route() {
    let left: Causes = [field_cause_via(
        "CanFoo",
        "name",
        "App",
        FieldIssue::Missing,
    )]
    .into_iter()
    .collect();
    let right: Causes = [
        field_cause_via("CanBar", "name", "App", FieldIssue::Missing),
        field_cause_via("CanBar", "age", "App", FieldIssue::Missing),
    ]
    .into_iter()
    .collect();

    let merged = Causes::union([left, right]);

    assert_eq!(merged.len(), 2);
    // The shared `name` cause keeps both consumers' routes.
    assert_eq!(merged[0].paths.len(), 2);
    assert_eq!(merged[1].paths.len(), 1);
}

#[test]
fn union_is_associative() {
    let a: Causes = [field_cause_via(
        "CanFoo",
        "name",
        "App",
        FieldIssue::Missing,
    )]
    .into_iter()
    .collect();
    let b: Causes = [field_cause_via(
        "CanBar",
        "name",
        "App",
        FieldIssue::Missing,
    )]
    .into_iter()
    .collect();
    let c: Causes = [field_cause_via("CanBaz", "age", "App", FieldIssue::Missing)]
        .into_iter()
        .collect();

    let left = Causes::union([Causes::union([a.clone(), b.clone()]), c.clone()]);
    let right = Causes::union([a, Causes::union([b, c])]);

    assert_eq!(left, right);
}

#[test]
fn heading_a_hop_prefixes_every_path() {
    let causes: Causes = [
        field_cause_via("CanFoo", "name", "App", FieldIssue::Missing),
        field_cause_via("CanBar", "age", "App", FieldIssue::Missing),
    ]
    .into_iter()
    .collect();

    let headed = causes.headed_by(&wrapper_hop());

    assert_eq!(headed.len(), 2);
    for cause in headed.iter() {
        for path in &cause.paths {
            assert_eq!(path.first(), Some(&ChainNode::Hop(wrapper_hop())));
        }
    }
}

/// Heading commutes with the grouping, which is why `headed_by` needs no re-merge: a single constant
/// hop cannot change any leaf's identity. The two anchors that head a chain used to rely on this
/// without stating it, and in opposite orders.
#[test]
fn heading_commutes_with_grouping() {
    let causes = [
        field_cause_via("CanFoo", "name", "App", FieldIssue::Missing),
        field_cause_via("CanBar", "name", "App", FieldIssue::Missing),
    ];

    let grouped_first = causes
        .iter()
        .cloned()
        .collect::<Causes>()
        .headed_by(&wrapper_hop());
    let headed_first: Causes = causes
        .iter()
        .cloned()
        .map(|cause| {
            [cause]
                .into_iter()
                .collect::<Causes>()
                .headed_by(&wrapper_hop())
        })
        .flat_map(|headed| headed.to_vec())
        .collect();

    assert_eq!(grouped_first, headed_first);
}
