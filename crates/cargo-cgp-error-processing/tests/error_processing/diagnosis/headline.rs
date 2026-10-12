//! The displayed headline: one cause, causes merged by kind and owner, and mixed causes.

use super::*;

fn missing_field(name: &str, owner: &str, context: &str) -> Cause {
    one_path(
        field_leaf(name, owner, FieldIssue::Missing),
        vec![consumer("CanGreet", context)],
    )
}

fn missing_wiring(component: &str, owner: &str) -> Cause {
    one_path(
        Leaf::MissingWiring {
            component: component.to_owned(),
            owner: owner.to_owned(),
        },
        vec![consumer("CanUseFoo", owner)],
    )
}

#[test]
fn one_cause_is_the_headline() {
    let causes = vec![missing_field("height", "Rectangle", "Rectangle")];
    assert_eq!(
        merged_cause(&causes).as_deref(),
        Some("missing field `height` on `Rectangle`")
    );
    assert_eq!(
        cause_headline("CGP-E001", "unused", &causes),
        "[CGP-E001] missing field `height` on `Rectangle`"
    );
}

#[test]
fn causes_of_one_kind_on_one_owner_merge() {
    let fields = vec![
        missing_field("first_name", "Person", "Person"),
        missing_field("last_name", "Person", "Person"),
    ];
    assert_eq!(
        merged_cause(&fields).as_deref(),
        Some("missing fields `first_name` and `last_name` on `Person`")
    );

    // A plain missing component and an unterminated redirect path are both missing wiring.
    let wiring = vec![
        missing_wiring("BarProviderComponent", "App"),
        one_path(
            Leaf::MissingRedirectWiring {
                path: "@app.Baz".to_owned(),
                context: "App".to_owned(),
            },
            vec![consumer("CanUseFoo", "App")],
        ),
    ];
    assert_eq!(
        merged_cause(&wiring).as_deref(),
        Some("missing wiring for `BarProviderComponent` and `@app.Baz` on `App`")
    );

    let entries: Vec<Cause> = ["A", "B", "C"]
        .into_iter()
        .map(|key| {
            one_path(
                Leaf::MissingDispatchEntry {
                    key: key.to_owned(),
                    table: "Sink".to_owned(),
                },
                vec![consumer("CanHandle", "App")],
            )
        })
        .collect();
    assert_eq!(
        merged_cause(&entries).as_deref(),
        Some("missing entries for `A`, `B`, and `C` in provider `Sink`")
    );
}

#[test]
fn mixed_causes_keep_the_class_sentence_with_a_count() {
    // One kind, but two owners: two fixes, so no single statement.
    let two_owners = vec![
        missing_field("name", "Inner", "Outer"),
        missing_field("name", "Outer", "Outer"),
    ];
    assert_eq!(merged_cause(&two_owners), None);
    // Two kinds on one owner.
    let two_kinds = vec![
        missing_field("name", "App", "App"),
        missing_wiring("BarProviderComponent", "App"),
    ];
    assert_eq!(merged_cause(&two_kinds), None);

    let resolved = cgp_resolved("App", &["CanGreet"], two_kinds);
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `App: CanUseComponent<GreeterComponent>` is not satisfied"),
        &resolved,
        &empty_names(),
    );
    assert_eq!(
        plan.header.as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`: 2 root \
             causes"
        )
    );
}

#[test]
fn a_coalesced_block_names_each_subject_when_its_causes_are_mixed() {
    let inner = cgp_resolved(
        "Inner",
        &["CanCompute"],
        vec![missing_field("name", "Inner", "Inner")],
    );
    let outer = cgp_resolved(
        "Outer",
        &["CanRun"],
        vec![
            missing_field("name", "Inner", "Outer"),
            missing_field("name", "Outer", "Outer"),
        ],
    );
    let causes: Vec<Cause> = outer.causes.to_vec();
    assert_eq!(
        coalesced_headline(&[&inner, &outer], &causes),
        "[CGP-E001] the consumer trait `CanCompute` is not implemented for context `Inner`, and \
         the consumer trait `CanRun` is not implemented for context `Outer`: 2 root causes"
    );
    // Sharing one cause, the same block states it alone.
    assert_eq!(
        coalesced_headline(&[&inner, &outer], &causes[..1]),
        "[CGP-E001] missing field `name` on `Inner`"
    );
}
