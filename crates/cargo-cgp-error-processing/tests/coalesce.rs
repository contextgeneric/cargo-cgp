//! Coalescing underived-field causes, over hand-built `Cause` values.

use cargo_cgp_error_processing::{
    Cause, ChainNode, DepNode, FieldIssue, Leaf, coalesce_underived_fields, derive_help_messages,
    root_cause_lead,
};

/// A cause whose single path is `HasFields` hop → the field leaf, with the given issue.
fn field_cause(name: &str, owner: &str, issue: FieldIssue) -> Cause {
    let leaf = Leaf::Field {
        name: name.to_owned(),
        owner: owner.to_owned(),
        issue,
    };
    Cause {
        paths: vec![vec![
            ChainNode::Hop(DepNode::Trait {
                trait_ref: "HasFields".to_owned(),
                self_ty: owner.to_owned(),
            }),
            ChainNode::Leaf(leaf.clone()),
        ]],
        leaf,
    }
}

#[test]
fn two_underived_fields_on_one_struct_become_one_cause() {
    let causes = vec![
        field_cause("height", "Rectangle", FieldIssue::Present),
        field_cause("width", "Rectangle", FieldIssue::Present),
    ];

    let coalesced = coalesce_underived_fields(&causes);

    assert_eq!(coalesced.len(), 1);
    let Leaf::UnderivedFields { names, owner } = &coalesced[0].leaf else {
        panic!(
            "expected an UnderivedFields leaf, got {:?}",
            coalesced[0].leaf
        );
    };
    assert_eq!(names, &["height".to_owned(), "width".to_owned()]);
    assert_eq!(owner, "Rectangle");
    // The merged cause keeps every field's path, so the graph still branches to each per-field leaf.
    assert_eq!(coalesced[0].paths.len(), 2);
    // One lead, listing both fields; one derive help, naming the one fix.
    assert_eq!(
        root_cause_lead(&coalesced[0].leaf),
        "accessor trait `HasField` is not implemented for the fields `height` and `width` of `Rectangle`"
    );
    assert_eq!(
        derive_help_messages(&coalesced),
        vec!["make sure that `#[derive(HasField)]` is used for `Rectangle`".to_owned()]
    );
}

#[test]
fn a_lone_underived_field_keeps_its_single_field_wording() {
    let causes = vec![field_cause("name", "App", FieldIssue::Present)];
    let coalesced = coalesce_underived_fields(&causes);
    assert_eq!(coalesced, causes);
}

#[test]
fn genuinely_missing_fields_are_never_coalesced() {
    // A struct missing several fields needs several fixes — one per field — and a fieldless
    // struct's derive (which emits no impls) reads as missing fields, so both stay apart.
    let causes = vec![
        field_cause("height", "Rectangle", FieldIssue::Missing),
        field_cause("width", "Rectangle", FieldIssue::Missing),
    ];
    let coalesced = coalesce_underived_fields(&causes);
    assert_eq!(coalesced, causes);
}

#[test]
fn underived_fields_on_different_structs_stay_apart() {
    let causes = vec![
        field_cause("height", "Rectangle", FieldIssue::Present),
        field_cause("radius", "Circle", FieldIssue::Present),
    ];
    let coalesced = coalesce_underived_fields(&causes);
    assert_eq!(coalesced, causes);
}

#[test]
fn an_underived_group_beside_a_missing_field_coalesces_only_the_group() {
    let causes = vec![
        field_cause("height", "Rectangle", FieldIssue::Present),
        field_cause("depth", "Rectangle", FieldIssue::Missing),
        field_cause("width", "Rectangle", FieldIssue::Present),
    ];

    let coalesced = coalesce_underived_fields(&causes);

    assert_eq!(coalesced.len(), 2);
    // The merged group takes the position of its first member.
    assert!(matches!(&coalesced[0].leaf, Leaf::UnderivedFields { .. }));
    assert!(matches!(
        &coalesced[1].leaf,
        Leaf::Field {
            issue: FieldIssue::Missing,
            ..
        }
    ));
}
