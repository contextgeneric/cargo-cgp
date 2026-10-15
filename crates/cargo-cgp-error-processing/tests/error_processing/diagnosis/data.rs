//! The extensible-data causes: their sentences and codes, how several merge, the derive and method
//! `help`s, the data class headline, and the partial value a chain hop names in words.

use cargo_cgp_error_processing::{DeriveFix, MethodFix, PartialKind, PartialValue, SuggestedCall};

use super::*;

/// A data-trait leaf `owner` lacks, with the derive a `help` names when one applies.
fn unimplemented(owner: &str, trait_name: &str, derive: Option<&str>) -> Leaf {
    Leaf::DataTraitUnimplemented {
        owner: owner.to_owned(),
        trait_name: trait_name.to_owned(),
        derive: DeriveFix(derive.map(str::to_owned)),
    }
}

/// An extractor of `FooBarBaz` given where the enum itself belongs, with the method a `help` names
/// in place of `downcast`.
fn partial_lacks(kind: PartialKind, replacement: Option<&str>) -> Leaf {
    Leaf::PartialLacksDataTrait {
        kind,
        target: "FooBarBaz".to_owned(),
        trait_name: "HasExtractor".to_owned(),
        replacement: MethodFix(replacement.map(|method| SuggestedCall {
            method: method.to_owned(),
            replaces: Some("downcast".to_owned()),
            trait_path: Some("cgp::core::field::impls::CanDowncastFields".to_owned()),
        })),
    }
}

/// A `Resolved` for a call to one of CGP's data traits on `owner`, a type with no wiring.
fn data_resolved(owner: &str, trait_ref: &str, causes: Vec<Cause>) -> Resolved {
    Resolved {
        context: owner.to_owned(),
        consumers: vec![trait_ref.to_owned()],
        consumers_are_cgp: false,
        subject_is_context: false,
        data_operation: true,
        provider: None,
        causes: causes.into_iter().collect(),
    }
}

#[test]
fn data_leaves_have_their_sentences_and_codes() {
    let cases = [
        (
            Leaf::FieldAlreadySet {
                name: "first_name".to_owned(),
                owner: "Person".to_owned(),
            },
            "[CGP-E118] field `first_name` of `Person` is already set",
        ),
        (
            Leaf::FieldsNotSet {
                names: vec!["port".to_owned()],
                owner: "Config".to_owned(),
            },
            "[CGP-E119] field `port` of `Config` is not set",
        ),
        (
            Leaf::FieldsNotSet {
                names: vec!["host".to_owned(), "port".to_owned()],
                owner: "Config".to_owned(),
            },
            "[CGP-E119] fields `host` and `port` of `Config` are not set",
        ),
        (
            Leaf::MissingVariant {
                name: "Bar".to_owned(),
                owner: "Big".to_owned(),
                similar: NearMiss::default(),
            },
            "[CGP-E120] missing variant `Bar` on `Big`",
        ),
        (
            Leaf::VariantsNotExtracted {
                names: vec!["Rectangle".to_owned()],
                owner: "Shape".to_owned(),
            },
            "[CGP-E121] variant `Rectangle` of `Shape` is not extracted",
        ),
        (
            Leaf::VariantsNotExtracted {
                names: vec!["Circle".to_owned(), "Rectangle".to_owned()],
                owner: "Shape".to_owned(),
            },
            "[CGP-E121] variants `Circle` and `Rectangle` of `Shape` are not extracted",
        ),
        (
            Leaf::VariantTypeMismatch {
                name: "Foo".to_owned(),
                owner: "FooBarBaz".to_owned(),
                expected: "u64".to_owned(),
                actual: "u32".to_owned(),
            },
            "[CGP-E122] variant `Foo` on `FooBarBaz` has type `u32`, but `u64` is required",
        ),
        (
            unimplemented("Shape", "HasFields", Some("HasFields")),
            "[CGP-E123] `Shape` does not implement `HasFields`",
        ),
        (
            Leaf::VariantAlreadyExtracted {
                name: "Circle".to_owned(),
                owner: "Shape".to_owned(),
            },
            "[CGP-E124] variant `Circle` of `Shape` is already extracted",
        ),
        (
            Leaf::FieldNotOptional {
                name: "foo".to_owned(),
                owner: "Context".to_owned(),
                convert: MethodFix::default(),
            },
            "[CGP-E125] field `foo` of `Context` is not optional",
        ),
        (
            partial_lacks(PartialKind::Extractor, None),
            "[CGP-E126] an extractor of `FooBarBaz` does not implement `HasExtractor`",
        ),
        (
            partial_lacks(PartialKind::RefExtractor, None),
            "[CGP-E126] a reference extractor of `FooBarBaz` does not implement `HasExtractor`",
        ),
    ];
    for (leaf, expected) in cases {
        assert_eq!(dependency_tree_leaf(&leaf), expected);
    }
}

#[test]
fn data_traits_one_type_lacks_merge_into_one_headline_and_help() {
    // Two traits a derive would supply, both unmet on one enum: one headline names both, and one
    // help names both derives in one attribute.
    let causes = vec![
        one_path(
            unimplemented("Shape", "HasFields", Some("HasFields")),
            vec![trait_hop("HasArea", "Shape")],
        ),
        one_path(
            unimplemented("Shape", "HasExtractor", Some("ExtractField")),
            vec![trait_hop("HasArea", "Shape")],
        ),
    ];
    assert_eq!(
        merged_cause(&causes).as_deref(),
        Some("`Shape` does not implement `HasFields` and `HasExtractor`")
    );
    assert_eq!(
        fix_help_messages(&causes),
        vec!["add `#[derive(HasFields, ExtractField)]` to `Shape`".to_owned()]
    );
}

#[test]
fn a_derive_that_cannot_be_added_gets_no_help() {
    // A type the crate does not define, or of a kind the derive does not accept, carries no derive.
    let causes = vec![one_path(
        unimplemented("Vec<u8>", "HasFields", None),
        vec![trait_hop("Describe", "Vec<u8>")],
    )];
    assert!(fix_help_messages(&causes).is_empty());
}

#[test]
fn a_missing_variant_names_a_similar_one() {
    let causes = vec![one_path(
        Leaf::MissingVariant {
            name: "circle".to_owned(),
            owner: "Shape".to_owned(),
            similar: NearMiss(Some("Circle".to_owned())),
        },
        vec![],
    )];
    assert_eq!(
        fix_help_messages(&causes),
        vec!["a similar variant exists on `Shape`: `Circle`".to_owned()]
    );
}

#[test]
fn a_method_help_names_the_call_and_its_import() {
    // The extractor takes `downcast_fields` in place of `downcast`, and the help writes the `use`
    // its trait needs, since the data traits are not in the prelude.
    let causes = vec![one_path(
        partial_lacks(PartialKind::Extractor, Some("downcast_fields")),
        vec![],
    )];
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "call `downcast_fields` in place of `downcast` on an extractor, with \
             `use cgp::core::field::impls::CanDowncastFields;` in scope"
                .to_owned()
        ]
    );
    // A core builder converts to an optional one first.
    let causes = vec![one_path(
        Leaf::FieldNotOptional {
            name: "foo".to_owned(),
            owner: "Context".to_owned(),
            convert: MethodFix(Some(SuggestedCall {
                method: "to_optional".to_owned(),
                replaces: None,
                trait_path: Some("cgp::extra::field::impls::ToOptional".to_owned()),
            })),
        },
        vec![],
    )];
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "call `to_optional` on the builder first to hold each field as optional, with \
             `use cgp::extra::field::impls::ToOptional;` in scope"
                .to_owned()
        ]
    );
    // Without a method that holds, the cause carries no help.
    let causes = vec![one_path(
        partial_lacks(PartialKind::Extractor, None),
        vec![],
    )];
    assert!(fix_help_messages(&causes).is_empty());
}

#[test]
fn the_data_states_are_told_apart_from_other_causes() {
    assert!(
        Leaf::VariantAlreadyExtracted {
            name: "Circle".to_owned(),
            owner: "Shape".to_owned(),
        }
        .is_data_state()
    );
    assert!(partial_lacks(PartialKind::Extractor, None).is_data_state());
    assert!(!unimplemented("Shape", "HasFields", Some("HasFields")).is_data_state());
    assert!(
        !Leaf::Bound {
            summary: "Port: Default".to_owned()
        }
        .is_data_state()
    );
}

#[test]
fn a_call_to_a_data_trait_takes_the_data_class() {
    // A call to `upcast` resolved to a variant the target lacks: the block is `[CGP-E050]`, its
    // headline states the cause, and its class sentence names the trait plainly.
    let leaf = Leaf::MissingVariant {
        name: "Bar".to_owned(),
        owner: "Big".to_owned(),
        similar: NearMiss::default(),
    };
    let resolved = data_resolved(
        "Small",
        "CanUpcast<Big>",
        vec![one_path(leaf, vec![trait_hop("CanUpcast<Big>", "Small")])],
    );
    let plan = plan_resolved(
        DiagKind::MethodNotFound,
        Some("the trait bound `Big: FromVariant<Symbol!(\"Bar\")>` is not satisfied"),
        &resolved,
        &empty_names(),
    );
    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E050] missing variant `Bar` on `Big`")
    );
    assert_eq!(
        plan.class_header.as_deref(),
        Some("[CGP-E050] the trait `CanUpcast<Big>` is not implemented for `Small`")
    );
}

#[test]
fn a_variant_payload_mismatch_is_its_blocks_headline() {
    // An `E0271` traced to a variant's payload states the mismatch as its headline, under the
    // anchor's class code, and the note carries the chain alone.
    let leaf = Leaf::VariantTypeMismatch {
        name: "Foo".to_owned(),
        owner: "FooBarBaz".to_owned(),
        expected: "u64".to_owned(),
        actual: "u32".to_owned(),
    };
    let resolved = data_resolved(
        "FooBar",
        "CanUpcast<FooBarBaz>",
        vec![one_path(
            leaf,
            vec![trait_hop("CanUpcast<FooBarBaz>", "FooBar")],
        )],
    );
    let plan = plan_resolved(DiagKind::TypeMismatch, None, &resolved, &empty_names());
    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E050] variant `Foo` on `FooBarBaz` has type `u32`, but `u64` is required")
    );
    assert!(!notes(&plan)[0].contains("root cause:"));
}

#[test]
fn a_partial_value_is_named_in_words() {
    let builder = |marked: &[&str], optional: &[&str]| PartialValue {
        kind: PartialKind::Builder,
        target: "Person".to_owned(),
        marked: marked.iter().map(|name| (*name).to_owned()).collect(),
        optional: optional.iter().map(|name| (*name).to_owned()).collect(),
    };
    assert_eq!(
        builder(&[], &[]).describe(),
        "the builder of `Person` with no field set"
    );
    assert_eq!(
        builder(&["first_name"], &[]).describe(),
        "the builder of `Person` with `first_name` set"
    );
    assert_eq!(
        builder(&["first_name"], &["last_name"]).describe(),
        "the builder of `Person` with `first_name` set and `last_name` optional"
    );
    let extractor = |kind, marked: &[&str]| PartialValue {
        kind,
        target: "Shape".to_owned(),
        marked: marked.iter().map(|name| (*name).to_owned()).collect(),
        optional: Vec::new(),
    };
    assert_eq!(
        extractor(PartialKind::Extractor, &[]).describe(),
        "the extractor of `Shape`"
    );
    assert_eq!(
        extractor(PartialKind::Extractor, &["Circle"]).describe(),
        "the extractor of `Shape` with `Circle` ruled out"
    );
    assert_eq!(
        extractor(PartialKind::RefExtractor, &["Circle"]).describe(),
        "the reference extractor of `Shape` with `Circle` ruled out"
    );
    assert_eq!(
        ChainNode::Hop(DepNode::PartialTrait {
            trait_ref: "BuildField<Symbol!(\"host\")>".to_owned(),
            value: builder(&["host"], &[]),
        })
        .render(),
        "[CGP-E104] trait impl `BuildField<Symbol!(\"host\")>` for the builder of `Person` with `host` set"
    );
}
