//! The `Causes` set and the *one cause per distinct leaf* invariant it enforces by construction.

use cargo_cgp_error_processing::{
    AdapterFix, Cause, Causes, ChainNode, CoreProviderFix, DepNode, FieldIssue, Leaf, ProvidesFix,
    coalesce_underived_fields, root_cause_lead,
};

use super::support::field_leaf;

/// A cause reaching `owner`'s `name` field through the named consumer: the shape one member of a
/// coalesced block contributes, each member reaching the shared cause down its own chain.
fn field_cause_via(consumer: &str, name: &str, owner: &str, issue: FieldIssue) -> Cause {
    let leaf = field_leaf(name, owner, issue);
    Cause {
        paths: vec![vec![
            ChainNode::Hop(DepNode::Consumer {
                trait_ref: consumer.to_owned(),
                context: owner.to_owned(),
                is_context: true,
            }),
            ChainNode::Leaf(leaf.clone()),
        ]],
        leaf,
    }
}

/// The hop an anchor heads a recovered chain with: the wrapper trait the programmer wrote.
fn wrapper_hop() -> DepNode {
    DepNode::Trait {
        trait_ref: "CanGreetChecked".to_owned(),
        self_ty: "App".to_owned(),
    }
}

// Property: for all causes collected into a `Causes`, each distinct leaf appears once, holding
//   every path that reaches it (`Causes`'s invariant).
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
/// The type is what makes that unreachable: there is no constructor that produces the bad list.
// Property: for all collections of causes naming one underived field, the coalesced lead names that
//   field once (`Causes`'s invariant, as the lead relies on it).
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
        "`HasField` not derived for field `name` of `App`"
    );
}

// Property: for all causes with distinct leaves, collecting keeps one cause per leaf (`Causes`'s
//   invariant).
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

// Property: for all causes, an exact repeat of a path is dropped when it is folded in
//   (`Causes::add_cause`'s doc).
#[test]
fn an_exact_repeat_of_a_path_is_dropped() {
    let cause = field_cause_via("CanFoo", "name", "App", FieldIssue::Missing);
    let causes: Causes = [cause.clone(), cause].into_iter().collect();

    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].paths.len(), 1);
}

// Property: for all sub-chains, `from_sub_chains` files the paths that reach one leaf under a
//   single cause.
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
// Property: for all sets a and b, `union([a, b])` holds one cause per distinct leaf of a and b,
//   with every path of either (`Causes::union`'s doc).
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

// Property: for all a, b, and c, `union([union([a, b]), c]) == union([a, union([b, c])])`
//   (`Causes::union`'s doc).
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

// Property: for all causes and every hop h, each path of `headed_by(h)` starts with h
//   (`Causes::headed_by`'s doc).
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
/// hop cannot change any leaf's identity. The two anchors that head a chain rely on this, in opposite
/// orders.
// Property: for all causes and every hop h, heading then grouping equals grouping then heading, so
//   `headed_by` needs no re-merge (`Causes::headed_by`'s doc).
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

fn provider_hop(provider: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Provider {
        trait_ref: "Greeter".to_owned(),
        context: "App".to_owned(),
        provider: provider.to_owned(),
    })
}

fn not_a_provider(provider: &str) -> Leaf {
    Leaf::NotAProvider {
        provider: provider.to_owned(),
        adapter: AdapterFix::default(),
        provides: ProvidesFix::default(),
        core: CoreProviderFix::default(),
        provider_trait: "Greeter".to_owned(),
    }
}

// Property: a provider hop directly above a not-a-provider leaf for the same provider renders as
//   the requirement template of `CGP-E102` in the error-code catalog.
#[test]
fn the_hop_above_a_not_a_provider_leaf_reads_as_a_requirement() {
    let leaf = not_a_provider("NotAGreeter");
    let causes = Causes::from_sub_chains([(
        leaf.clone(),
        vec![
            provider_hop("WrapGreeter<NotAGreeter>"),
            provider_hop("NotAGreeter"),
            ChainNode::Leaf(leaf),
        ],
    )]);

    let rendered: Vec<String> = causes[0].paths[0].iter().map(ChainNode::render).collect();
    assert_eq!(
        rendered[0],
        "[CGP-E102] provider trait impl `Greeter` with context `App` for provider `WrapGreeter<NotAGreeter>`"
    );
    assert_eq!(
        rendered[1],
        "[CGP-E102] provider trait `Greeter` required of provider `NotAGreeter` with context `App`"
    );
}

// Property: a provider hop above a not-a-provider leaf for another provider keeps the impl template
//   of `CGP-E102` in the error-code catalog.
#[test]
fn a_hop_for_another_provider_above_a_not_a_provider_leaf_keeps_its_impl_form() {
    let leaf = not_a_provider("NotAGreeter");
    let causes = Causes::from_sub_chains([(
        leaf.clone(),
        vec![
            provider_hop("WrapGreeter<NotAGreeter>"),
            ChainNode::Leaf(leaf),
        ],
    )]);

    assert_eq!(
        causes[0].paths[0][0].render(),
        "[CGP-E102] provider trait impl `Greeter` with context `App` for provider `WrapGreeter<NotAGreeter>`"
    );
}
