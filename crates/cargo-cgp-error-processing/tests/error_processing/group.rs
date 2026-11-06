//! Grouping coalescible failures by the root causes they share, over hand-built `Resolved` values.

use cargo_cgp_error_processing::{
    Cause, Causes, ChainNode, DepNode, FieldIssue, Resolved, ResolvedClass, group_by_shared_cause,
};

use super::support::field_leaf;

/// A failure of `consumer` on `context` bottoming out on one missing field per name in `fields`:
/// the shape a check entry or a use-site call recovers, with the field names standing for the
/// distinct root causes a failure reaches.
fn failure(context: &str, consumer: &str, fields: &[&str]) -> Resolved {
    let causes: Causes = fields
        .iter()
        .map(|name| {
            let leaf = field_leaf(name, context, FieldIssue::Missing);
            Cause {
                paths: vec![vec![
                    ChainNode::Hop(DepNode::Consumer {
                        trait_ref: consumer.to_owned(),
                        context: context.to_owned(),
                        is_context: true,
                    }),
                    ChainNode::Leaf(leaf.clone()),
                ]],
                leaf,
            }
        })
        .collect();

    Resolved {
        context: context.to_owned(),
        consumers: vec![consumer.to_owned()],
        class: ResolvedClass::Consumer,
        subject_is_context: true,
        causes,
    }
}

// Property: for all failures a and b that share a root cause, `group_by_shared_cause` puts them in
//   one group.
#[test]
fn failures_sharing_a_cause_group_together() {
    let first = failure("App", "CanFoo", &["name"]);
    let second = failure("App", "CanBar", &["name"]);

    let resolveds = [&first, &second];

    assert_eq!(group_by_shared_cause(&resolveds), vec![vec![0, 1]]);
}

// Property: for all failures with disjoint cause sets and no failure linking them, each forms its
//   own group.
#[test]
fn failures_with_disjoint_causes_stay_apart() {
    let first = failure("App", "CanFoo", &["name"]);
    let second = failure("App", "CanBar", &["age"]);

    let resolveds = [&first, &second];

    assert_eq!(group_by_shared_cause(&resolveds), vec![vec![0], vec![1]]);
}

/// The shape one omitted wiring entry produces: each check entry reaches one of the causes while
/// the use-site call reaches both, so the three cause sets overlap without any two being equal.
/// Demanding equal sets left this as three separate blocks for one mistake; grouping on a *shared*
/// cause folds it into one.
// Property: the groups are the connected components of the shares-a-cause relation, so overlapping
//   cause sets that are never equal still form one group (`group_by_shared_cause`'s doc).
#[test]
fn a_failure_whose_causes_cover_two_others_joins_them_all() {
    let first_check = failure("App", "CanFoo", &["name"]);
    let second_check = failure("App", "CanBar", &["age"]);
    let use_site = failure("App", "CanFoo", &["name", "age"]);

    let resolveds = [&first_check, &second_check, &use_site];

    assert_eq!(group_by_shared_cause(&resolveds), vec![vec![0, 1, 2]]);
}

/// Grouping is transitive, which makes it a partition: a failure covered by two others has
/// one group to join rather than an ambiguous choice between them.
// Property: for all failures a, b, and c, if a shares a cause with b and b with c, all three group
//   together (`group_by_shared_cause`'s doc).
#[test]
fn grouping_is_transitive_through_a_shared_cause() {
    let left = failure("App", "CanFoo", &["name"]);
    let bridge = failure("App", "CanBar", &["name", "age"]);
    let right = failure("App", "CanBaz", &["age"]);

    let resolveds = [&left, &bridge, &right];

    assert_eq!(group_by_shared_cause(&resolveds), vec![vec![0, 1, 2]]);
}

/// The context is part of every cause key, so one field name missing on two different contexts is
/// two mistakes and never groups.
// Property: a cause is keyed by its leaf, which names its owner, so one field name missing on two
//   different structs is two causes and never groups.
#[test]
fn the_same_field_on_two_contexts_never_groups() {
    let inner = failure("Inner", "CanFoo", &["name"]);
    let outer = failure("Outer", "CanFoo", &["name"]);

    let resolveds = [&inner, &outer];

    assert_eq!(group_by_shared_cause(&resolveds), vec![vec![0], vec![1]]);
}

/// A failure with no recovered cause shares nothing, so it forms its own group rather than being
/// swept into another's.
// Property: a failure with no recovered cause shares none, so it forms a group of its own.
#[test]
fn a_causeless_failure_forms_its_own_group() {
    let causeless = failure("App", "CanFoo", &[]);
    let other = failure("App", "CanBar", &["name"]);

    let resolveds = [&causeless, &other];

    assert_eq!(group_by_shared_cause(&resolveds), vec![vec![0], vec![1]]);
}

/// Groups are ordered by their first member and each group holds its members in arrival order, so
/// the emitter can emit a group where it first appeared and keep the overall diagnostic ordering.
// Property: groups are ordered by their first member, and each group lists its members in arrival
//   order (`group_by_shared_cause`'s doc).
#[test]
fn groups_and_members_keep_arrival_order() {
    let first = failure("App", "CanFoo", &["age"]);
    let second = failure("App", "CanBar", &["name"]);
    let third = failure("App", "CanBaz", &["age"]);

    let resolveds = [&first, &second, &third];

    assert_eq!(group_by_shared_cause(&resolveds), vec![vec![0, 2], vec![1]]);
}

// Property: an empty input yields no groups.
#[test]
fn an_empty_input_yields_no_groups() {
    assert!(group_by_shared_cause(&[]).is_empty());
}
