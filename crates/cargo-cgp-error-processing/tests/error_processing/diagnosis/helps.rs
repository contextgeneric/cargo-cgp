//! The `help` lines a plan attaches: the fix each kind of cause calls for, merged per owner.

use cargo_cgp_error_processing::{
    Adapter, AdapterFix, Causes, CoreProviderFix, CoreProviderHint, DeriveFix, EntryFix, FieldFix,
    MethodFix, NearMiss, PartialKind, ProvidedComponent, ProvidesFix, SuggestedCall,
};

use super::*;

/// A missing field `name` on `owner`, confirmed to be fixed by adding it, whose requirement states
/// `ty` as its type.
fn typed_missing_field(name: &str, owner: &str, ty: Option<&str>) -> Leaf {
    Leaf::Field {
        name: name.to_owned(),
        owner: owner.to_owned(),
        issue: FieldIssue::Missing,
        fix: FieldFix {
            confirmed: true,
            ty: ty.map(str::to_owned),
            similar: None,
            index_tag: None,
        },
    }
}

/// A cause with one path from `CanRun` on `App` down to `leaf`.
fn app_cause(leaf: Leaf) -> Cause {
    one_path(leaf, vec![consumer("CanRun", "App")])
}

#[test]
fn plans_a_missing_derive_with_a_help() {
    let leaf = field_leaf("height", "Rectangle", FieldIssue::Present);
    let resolved = cgp_resolved(
        "Rectangle",
        &["CanCalculateArea"],
        vec![one_path(
            leaf,
            vec![consumer("CanCalculateArea", "Rectangle")],
        )],
    );
    let plan = plan_resolved(
        DiagKind::Check,
        Some(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
        ),
        &resolved,
        &empty_names(),
    );

    assert_eq!(
        plan.helps,
        vec!["add `#[derive(HasField)]` to `Rectangle`".to_owned()]
    );
    assert!(notes(&plan)[0].starts_with(
        "root cause: [CGP-E111] `HasField` not derived for field `height` of `Rectangle`"
    ));
}

#[test]
fn a_deref_target_help_points_at_the_target() {
    let leaf = Leaf::Field {
        name: "name".to_owned(),
        owner: "App".to_owned(),
        issue: FieldIssue::PresentViaDeref {
            target: "AppFields".to_owned(),
        },
        fix: FieldFix::default(),
    };
    let resolved = cgp_resolved(
        "App",
        &["CanGreet"],
        vec![one_path(leaf, vec![consumer("CanGreet", "App")])],
    );
    let plan = plan_resolved(DiagKind::Check, None, &resolved, &empty_names());
    assert_eq!(
        plan.helps,
        vec!["add `#[derive(HasField)]` to `AppFields`".to_owned()]
    );
}

#[test]
fn a_missing_field_help_names_the_type_its_requirement_states() {
    let causes = [app_cause(typed_missing_field("height", "App", Some("f64")))];
    assert_eq!(
        fix_help_messages(&causes),
        vec!["add the field `height: f64` to `App`".to_owned()]
    );
}

#[test]
fn missing_fields_on_one_struct_share_one_help_in_written_order() {
    let causes = [
        app_cause(typed_missing_field("alpha", "App", Some("u64"))),
        app_cause(typed_missing_field("beta", "App", None)),
        app_cause(typed_missing_field("gamma", "App", Some("String"))),
        app_cause(typed_missing_field("delta", "Other", Some("u8"))),
    ];
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "add the fields `alpha: u64`, `beta`, and `gamma: String` to `App`".to_owned(),
            "add the field `delta: u8` to `Other`".to_owned(),
        ]
    );
}

#[test]
fn missing_wiring_on_one_context_shares_one_help() {
    let wiring = |component: &str| Leaf::MissingWiring {
        component: component.to_owned(),
        owner: "App".to_owned(),
        similar: NearMiss::default(),
    };
    let causes = [
        app_cause(wiring("FooComponent")),
        app_cause(Leaf::MissingRedirectWiring {
            path: "@app.BarComponent".to_owned(),
            context: "App".to_owned(),
            similar: NearMiss::default(),
        }),
    ];
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "wire `FooComponent` and `@app.BarComponent` to providers in the wiring of `App`"
                .to_owned()
        ]
    );
}

#[test]
fn missing_entries_share_one_help_per_table() {
    let entry = |key: &str, table: &str| Leaf::MissingDispatchEntry {
        key: key.to_owned(),
        table: table.to_owned(),
    };
    let causes = [
        app_cause(entry("u64", "Handlers")),
        app_cause(entry("String", "Handlers")),
        app_cause(entry("u64", "Bundle")),
    ];
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "add entries for `u64` and `String` to `Handlers`".to_owned(),
            "add an entry for `u64` to `Bundle`".to_owned(),
        ]
    );
}

#[test]
fn mixed_causes_get_one_help_per_kind_in_cause_order() {
    let causes = [
        app_cause(Leaf::MissingWiring {
            component: "FooComponent".to_owned(),
            owner: "App".to_owned(),
            similar: NearMiss::default(),
        }),
        app_cause(typed_missing_field("name", "App", Some("String"))),
        app_cause(field_leaf("width", "Rectangle", FieldIssue::Present)),
        app_cause(Leaf::Bound {
            summary: "f64: Eq".to_owned(),
        }),
        app_cause(Leaf::NotAProvider {
            provider: "Request".to_owned(),
            adapter: AdapterFix::default(),
            provides: ProvidesFix::default(),
            core: CoreProviderFix::default(),
            provider_trait: "Handler".to_owned(),
        }),
        app_cause(Leaf::ProviderForOtherArguments {
            provider: "Double".to_owned(),
            provider_trait: "Computer".to_owned(),
            param: Some("Input".to_owned()),
            provided: vec!["u64".to_owned()],
            required: "String".to_owned(),
        }),
    ];
    // An ordinary bound, a non-provider, and a provider for other arguments carry no help of their
    // own.
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "wire `FooComponent` to a provider in the wiring of `App`".to_owned(),
            "add the field `name: String` to `App`".to_owned(),
            "add `#[derive(HasField)]` to `Rectangle`".to_owned(),
        ]
    );
}

#[test]
fn a_field_reached_with_two_required_types_is_one_cause_named_without_a_type() {
    // The required type is not part of the cause's identity, so the two routes stay one cause; they
    // disagree on the type, so the help names the field alone.
    let causes = Causes::from_sub_chains([
        (
            typed_missing_field("height", "App", Some("f64")),
            vec![consumer("CanRun", "App")],
        ),
        (
            typed_missing_field("height", "App", Some("u32")),
            vec![consumer("CanWalk", "App")],
        ),
    ]);
    assert_eq!(causes.len(), 1);
    assert_eq!(
        fix_help_messages(&causes),
        vec!["add the field `height` to `App`".to_owned()]
    );

    // Routes that agree keep the type.
    let causes = Causes::from_sub_chains([
        (
            typed_missing_field("height", "App", Some("f64")),
            vec![consumer("CanRun", "App")],
        ),
        (
            typed_missing_field("height", "App", Some("f64")),
            vec![consumer("CanWalk", "App")],
        ),
    ]);
    assert_eq!(
        fix_help_messages(&causes),
        vec!["add the field `height: f64` to `App`".to_owned()]
    );
}

#[test]
fn a_missing_field_whose_fix_is_unconfirmed_gets_no_help() {
    // Its struct implements `HasField` for no field, so the derive may be missing as well.
    let leaf = Leaf::Field {
        name: "name".to_owned(),
        owner: "App".to_owned(),
        issue: FieldIssue::Missing,
        fix: FieldFix::default(),
    };
    assert!(fix_help_messages(&[app_cause(leaf.clone())]).is_empty());

    // One route that cannot confirm the fix makes the merged cause unconfirmed.
    let causes = Causes::from_sub_chains([
        (
            typed_missing_field("name", "App", Some("String")),
            vec![consumer("CanRun", "App")],
        ),
        (leaf, vec![consumer("CanWalk", "App")]),
    ]);
    assert!(fix_help_messages(&causes).is_empty());
}

#[test]
fn a_key_holding_a_reference_gets_no_help() {
    // The lifetime is erased, so the key cannot be pasted as written.
    let causes = [
        app_cause(Leaf::MissingRedirectWiring {
            path: "@ItemEncoderComponent.&u64".to_owned(),
            context: "App".to_owned(),
            similar: NearMiss::default(),
        }),
        app_cause(Leaf::MissingDispatchEntry {
            key: "&str".to_owned(),
            table: "Handlers".to_owned(),
        }),
    ];
    assert!(fix_help_messages(&causes).is_empty());

    // An erased lifetime argument cannot be pasted either, while a `'_'` spelled out in a
    // `Symbol!` is a character of the key, which reads as written once resugared.
    let causes = [
        app_cause(Leaf::MissingDispatchEntry {
            key: "Wrap<'_>".to_owned(),
            table: "Handlers".to_owned(),
        }),
        app_cause(Leaf::MissingRedirectWiring {
            path: "PathCons<Symbol<3, Chars<'a', Chars<'_', Chars<'b', Nil>>>>, Nil>".to_owned(),
            context: "App".to_owned(),
            similar: NearMiss::default(),
        }),
    ];
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "wire `PathCons<Symbol<3, Chars<'a', Chars<'_', Chars<'b', Nil>>>>, Nil>` to a \
             provider in the wiring of `App`"
                .to_owned()
        ]
    );
}

#[test]
fn a_near_miss_is_named_before_the_fix() {
    let field = Leaf::Field {
        name: "frist_name".to_owned(),
        owner: "Person".to_owned(),
        issue: FieldIssue::Missing,
        fix: FieldFix {
            confirmed: true,
            ty: Some("String".to_owned()),
            similar: Some("first_name".to_owned()),
            index_tag: None,
        },
    };
    let path = Leaf::MissingRedirectWiring {
        path: "@app.GreeterComponent".to_owned(),
        context: "App".to_owned(),
        similar: NearMiss(Some("@App.GreeterComponent".to_owned())),
    };
    let wiring = Leaf::MissingWiring {
        component: "GreeterComponent".to_owned(),
        owner: "App".to_owned(),
        similar: NearMiss(Some("@app.GreeterComponent".to_owned())),
    };
    let causes = [app_cause(field), app_cause(path), app_cause(wiring)];
    assert_eq!(
        fix_help_messages(&causes),
        vec![
            "a similar field exists on `Person`: `first_name`".to_owned(),
            "add the field `frist_name: String` to `Person`".to_owned(),
            "a similar key exists in the wiring of `App`: `@App.GreeterComponent`".to_owned(),
            "wire `@app.GreeterComponent` and `GreeterComponent` to providers in the wiring of \
             `App`"
                .to_owned(),
            "a similar key exists in the wiring of `App`: `@app.GreeterComponent`".to_owned(),
        ]
    );
}

#[test]
fn a_near_miss_needs_no_confirmed_fix() {
    // A struct that does not derive `HasField` gets no add-field help, but its similar field is
    // still named.
    let field = Leaf::Field {
        name: "widht".to_owned(),
        owner: "Shape".to_owned(),
        issue: FieldIssue::Missing,
        fix: FieldFix {
            similar: Some("width".to_owned()),
            ..FieldFix::default()
        },
    };
    assert_eq!(
        fix_help_messages(&[app_cause(field)]),
        vec!["a similar field exists on `Shape`: `width`".to_owned()]
    );
}

fn not_a_provider(adapters: Vec<(&str, Option<&str>)>) -> Leaf {
    Leaf::NotAProvider {
        provider: "Double".to_owned(),
        provider_trait: "TryComputer".to_owned(),
        provides: ProvidesFix::default(),
        core: CoreProviderFix::default(),
        adapter: AdapterFix(
            adapters
                .into_iter()
                .map(|(ty, import)| Adapter {
                    ty: ty.to_owned(),
                    import: import.map(str::to_owned),
                })
                .collect(),
        ),
    }
}

#[test]
fn a_non_provider_names_the_adapter_that_makes_it_one_with_its_import() {
    let leaf = not_a_provider(vec![(
        "Promote<Double>",
        Some("cgp::extra::handler::Promote"),
    )]);
    assert_eq!(
        fix_help_messages(&[app_cause(leaf)]),
        vec!["wire `Promote<Double>` in place of `Double`, with `use cgp::extra::handler::Promote;` in scope"
            .to_owned()]
    );
}

#[test]
fn several_adapters_are_named_alternatives_and_a_prelude_adapter_needs_no_import() {
    let leaf = not_a_provider(vec![
        ("WithProvider<Double>", None),
        ("Promote<Double>", None),
    ]);
    assert_eq!(
        fix_help_messages(&[app_cause(leaf)]),
        vec!["wire `WithProvider<Double>` or `Promote<Double>` in place of `Double`".to_owned()]
    );
}

#[test]
fn a_non_provider_that_provides_another_component_names_its_key() {
    let leaf = Leaf::NotAProvider {
        provider: "SayBye".to_owned(),
        provider_trait: "Greeter".to_owned(),
        adapter: AdapterFix::default(),
        provides: ProvidesFix(vec![ProvidedComponent {
            provider_trait: "Farewell".to_owned(),
            component: "FarewellComponent".to_owned(),
        }]),
        core: CoreProviderFix::default(),
    };
    assert_eq!(
        fix_help_messages(&[app_cause(leaf)]),
        vec![
            "`SayBye` provides `Farewell`: wire it under `FarewellComponent`, or wire a provider of `Greeter` here"
                .to_owned()
        ]
    );
}

fn core_hint(hint: CoreProviderHint) -> Leaf {
    Leaf::NotAProvider {
        provider: "Wired".to_owned(),
        provider_trait: "Slot".to_owned(),
        adapter: AdapterFix::default(),
        provides: ProvidesFix::default(),
        core: CoreProviderFix(Some(hint)),
    }
}

#[test]
fn a_misused_core_provider_names_what_serves_the_slot() {
    assert_eq!(
        fix_help_messages(&[app_cause(core_hint(CoreProviderHint::UseFields))]),
        vec!["wire `UseFields` in place of `Wired`, which reads each method's field by the method's name"
            .to_owned()]
    );
    let open = CoreProviderHint::OpenDispatch {
        component: "AreaCalculatorComponent".to_owned(),
        params: vec!["Shape".to_owned()],
    };
    assert_eq!(
        fix_help_messages(&[app_cause(core_hint(open))]),
        vec!["dispatch with `open AreaCalculatorComponent;` and one `@AreaCalculatorComponent.Shape: <provider>` entry per type, in place of `Wired`"
            .to_owned()]
    );
}

#[test]
fn a_non_provider_with_no_confirmed_adapter_gets_no_help() {
    assert!(fix_help_messages(&[app_cause(not_a_provider(Vec::new()))]).is_empty());
}

#[test]
fn a_tuple_field_tagged_by_a_string_names_its_index_tag() {
    // A tuple struct cannot gain a named field, so only the `Index` tag is named.
    let field = Leaf::Field {
        name: "0".to_owned(),
        owner: "Pair".to_owned(),
        issue: FieldIssue::Missing,
        fix: FieldFix {
            index_tag: Some("Index<0>".to_owned()),
            ..FieldFix::default()
        },
    };
    assert_eq!(
        fix_help_messages(&[app_cause(field)]),
        vec![
            "a field of the tuple struct `Pair` is keyed by `Index<0>`, not by `Symbol!(\"0\")`"
                .to_owned()
        ]
    );
}

#[test]
fn a_hand_written_wiring_names_the_entry_to_write_when_its_delegate_is_known() {
    let leaf = |delegate: Option<&str>| Leaf::WiringWithoutProviderFor {
        component: "GreeterComponent".to_owned(),
        owner: "App".to_owned(),
        delegate: EntryFix(delegate.map(str::to_owned)),
    };
    assert_eq!(
        dependency_tree_leaf(&leaf(Some("GreetHello"))),
        "[CGP-E128] the wiring of `GreeterComponent` on `App` has no `IsProviderFor` impl"
    );
    assert_eq!(
        fix_help_messages(&[app_cause(leaf(Some("GreetHello")))]),
        vec![
            "replace the hand-written `DelegateComponent` impl with `delegate_components! { App { GreeterComponent: GreetHello } }`"
                .to_owned()
        ]
    );
    // A `DelegateComponent` impl a macro wrote, or one from another crate, names no delegate, so
    // the cause carries no help.
    assert!(fix_help_messages(&[app_cause(leaf(None))]).is_empty());
}

#[test]
fn an_unbounded_parameter_names_the_bound_to_add() {
    let leaf = Leaf::UnboundedParameter {
        param: "Context".to_owned(),
        trait_name: "CanGreet".to_owned(),
        item: "run".to_owned(),
    };
    assert_eq!(
        dependency_tree_leaf(&leaf),
        "[CGP-E129] the type parameter `Context` of `run` is not bounded by `CanGreet`"
    );
    assert_eq!(
        fix_help_messages(&[app_cause(leaf)]),
        vec!["add the bound `Context: CanGreet` to `run`".to_owned()]
    );
}

/// The `help`s of one cause reached by two routes, one ending at each of `leaves`.
fn helps_of_two_routes(leaves: [Leaf; 2]) -> Vec<String> {
    let [first, second] = leaves;
    let causes = Causes::from_sub_chains([
        (first, vec![consumer("CanRun", "App")]),
        (second, vec![consumer("CanWalk", "App")]),
    ]);
    assert_eq!(causes.len(), 1, "the two routes are one cause");
    fix_help_messages(&causes)
}

#[test]
fn routes_that_disagree_on_a_fix_drop_its_help() {
    let provides = |component: &str| Leaf::NotAProvider {
        provider: "SayBye".to_owned(),
        provider_trait: "Greeter".to_owned(),
        adapter: AdapterFix::default(),
        provides: ProvidesFix(vec![ProvidedComponent {
            provider_trait: "Farewell".to_owned(),
            component: component.to_owned(),
        }]),
        core: CoreProviderFix::default(),
    };
    let derive = |derive: &str| Leaf::DataTraitUnimplemented {
        owner: "Shape".to_owned(),
        trait_name: "HasFields".to_owned(),
        derive: DeriveFix(Some(derive.to_owned())),
    };
    let delegate = |provider: &str| Leaf::WiringWithoutProviderFor {
        component: "GreeterComponent".to_owned(),
        owner: "App".to_owned(),
        delegate: EntryFix(Some(provider.to_owned())),
    };
    let call = |method: &str| {
        MethodFix(Some(SuggestedCall {
            method: method.to_owned(),
            replaces: None,
            trait_path: None,
        }))
    };
    let not_optional = |method: &str| Leaf::FieldNotOptional {
        name: "port".to_owned(),
        owner: "Config".to_owned(),
        convert: call(method),
    };
    let partial = |method: &str| Leaf::PartialLacksDataTrait {
        kind: PartialKind::Extractor,
        target: "Shape".to_owned(),
        trait_name: "HasExtractor".to_owned(),
        replacement: call(method),
    };
    let open = CoreProviderHint::OpenDispatch {
        component: "AreaCalculatorComponent".to_owned(),
        params: vec!["Shape".to_owned()],
    };

    // Each pair differs only in the fix one route found, so a help naming either would not fix the
    // other route's requirement.
    let disagreeing = [
        [
            not_a_provider(vec![("Promote<Double>", None)]),
            not_a_provider(vec![("WithProvider<Double>", None)]),
        ],
        [provides("FarewellComponent"), provides("ByeComponent")],
        [core_hint(CoreProviderHint::UseFields), core_hint(open)],
        [derive("HasFields"), derive("CgpData")],
        [delegate("GreetHello"), delegate("GreetBye")],
        [not_optional("to_optional"), not_optional("into_optional")],
        [partial("downcast_fields"), partial("downcast_variants")],
    ];
    for leaves in disagreeing {
        let helps = helps_of_two_routes(leaves.clone());
        assert!(helps.is_empty(), "{leaves:?} kept {helps:?}");
    }

    // Routes that agree keep the fix.
    assert_eq!(
        helps_of_two_routes([derive("HasFields"), derive("HasFields")]),
        vec!["add `#[derive(HasFields)]` to `Shape`".to_owned()]
    );
}
