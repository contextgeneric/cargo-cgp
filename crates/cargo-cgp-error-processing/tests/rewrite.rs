//! Tests for the compiler-free wiring-message rewrite (`cargo_cgp_error_processing::rewrite`).
//!
//! This crate links no compiler internals, so — unlike the driver that drives this logic —
//! the tests need no `rustc_private` gate and run on any toolchain. The rewrite is exercised
//! from a hand-built name map, the same shape the driver produces from a `TyCtxt`.

use std::collections::HashMap;

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, ComponentTraitNames, parse_trait_bound, rewrite_message,
    rewrite_required_for, rewrite_trait_bound, rewrite_wiring_overflow,
};

/// The fixed map every test's [`ComponentNameMap`] initializes to.
fn names() -> HashMap<String, ComponentTraitNames> {
    let mut map = HashMap::new();
    map.insert(
        "AreaCalculatorComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanCalculateArea".to_owned(),
            provider: "AreaCalculator".to_owned(),
        },
    );
    map
}

/// A lazily-initialized map over [`names`], the driver-side `ComponentNameMap` stand-in.
fn name_map() -> ComponentNameMap {
    ComponentNameMap::new(names)
}

#[test]
fn rewrites_is_provider_for_note() {
    let out = rewrite_required_for(
        "required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle`"
        )
    );
}

#[test]
fn rewrites_can_use_component_note() {
    let out = rewrite_required_for(
        "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
}

#[test]
fn strips_module_prefix_on_trait_and_marker() {
    // At driver time the notes still carry the `cgp::prelude::` re-export prefix; the
    // trait is matched by its last segment and the marker keyed by its last segment.
    let out = rewrite_required_for(
        "required for `Rectangle` to implement `cgp::prelude::CanUseComponent<cgp::prelude::AreaCalculatorComponent>`",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
}

#[test]
fn keeps_a_generic_context_whole() {
    // A context with its own generic arguments must survive the top-level comma split.
    let out = rewrite_required_for(
        "required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, App<Foo, Bar>>`",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `App<Foo, Bar>`"
        )
    );
}

#[test]
fn ignores_a_params_tuple_after_the_context() {
    let out = rewrite_required_for(
        "required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle, (u32, u64)>`",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle`"
        )
    );
}

#[test]
fn consumer_note_elides_generic_parameters() {
    // A generic component's consumer note still names the trait; the extra parameters are
    // elided in the descriptive prose (the header carries the fully-parameterized bound).
    let single = rewrite_required_for(
        "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent, f64>`",
        &name_map(),
    );
    assert_eq!(
        single.as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
    let tuple = rewrite_required_for(
        "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent, (u32, u64)>`",
        &name_map(),
    );
    assert_eq!(
        tuple.as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
}

#[test]
fn leaves_unknown_component_untouched() {
    // A marker absent from the map means the names are unknown, so nothing is rewritten.
    let out = rewrite_required_for(
        "required for `Rectangle` to implement `CanUseComponent<UnknownComponent>`",
        &name_map(),
    );
    assert_eq!(out, None);
}

#[test]
fn leaves_unrelated_notes_untouched() {
    assert_eq!(
        rewrite_required_for(
            "required for `Rectangle` to implement `HasRectangleFields`",
            &name_map(),
        ),
        None
    );
    assert_eq!(
        rewrite_required_for("required by a bound in `__CheckRectangle`", &name_map()),
        None
    );
}

#[test]
fn rewrites_can_use_component_header() {
    let out = rewrite_trait_bound(
        "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`"
        )
    );
}

#[test]
fn rewrites_is_provider_for_header() {
    let out = rewrite_trait_bound(
        "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle>` is not satisfied",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator` with context `Rectangle` is not implemented for provider `RectangleArea`"
        )
    );
}

#[test]
fn header_keeps_a_generic_subject_whole() {
    // The `: ` split must find the self/trait separator, not a colon inside the subject's
    // own generic arguments.
    let out = rewrite_trait_bound(
        "the trait bound `RedirectLookup<App, Nil>: IsProviderFor<AreaCalculatorComponent, Rectangle>` is not satisfied",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator` with context `Rectangle` is not implemented for provider `RedirectLookup<App, Nil>`"
        )
    );
}

#[test]
fn header_strips_module_prefix() {
    let out = rewrite_trait_bound(
        "the trait bound `Rectangle: cgp::prelude::CanUseComponent<AreaCalculatorComponent>` is not satisfied",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`"
        )
    );
}

#[test]
fn header_reattaches_a_single_generic_parameter() {
    // A generic component: `CanUseComponent<Marker, f64>` recovers `ConsumerTrait<f64>`, and
    // `IsProviderFor<Marker, Context, f64>` recovers `ProviderTrait<Context, f64>`.
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent, f64>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanCalculateArea<f64>` is not implemented for context `Rectangle`"
        )
    );
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle, f64>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator<f64>` with context `Rectangle` is not implemented for provider `RectangleArea`"
        )
    );
}

#[test]
fn header_unwraps_a_multi_parameter_tuple() {
    // Two or more parameters arrive grouped in a tuple, which is unwrapped so the reattached
    // list matches how the trait was written (`ConsumerTrait<u32, u64>`, not `<(u32, u64)>`).
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent, (u32, u64)>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanCalculateArea<u32, u64>` is not implemented for context `Rectangle`"
        )
    );
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle, (u32, u64)>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator<u32, u64>` with context `Rectangle` is not implemented for provider `RectangleArea`"
        )
    );
}

#[test]
fn rewrites_a_three_parameter_generic_component() {
    // A component with three generic parameters: they arrive grouped as `(u32, u64, bool)`
    // and are unwrapped in the header so the trait reads as written, while the notes name the
    // bare trait. The strings mirror what `generic_area_multi.rs` produces end to end.
    let map = name_map();

    // Consumer header: `CanUseComponent<Marker, (u32, u64, bool)>` → `CanCalculateArea<u32, u64, bool>`.
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent, (u32, u64, bool)>` is not satisfied",
            &map,
        )
        .as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanCalculateArea<u32, u64, bool>` is not implemented for context `Rectangle`"
        )
    );

    // Provider header: the context is named in prose, then the three parameters reattach.
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle, (u32, u64, bool)>` is not satisfied",
            &map,
        )
        .as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator<u32, u64, bool>` with context `Rectangle` is not implemented for provider `RectangleArea`"
        )
    );

    // Notes name the bare trait and elide the three parameters.
    assert_eq!(
        rewrite_required_for(
            "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent, (u32, u64, bool)>`",
            &map,
        )
        .as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
    assert_eq!(
        rewrite_required_for(
            "required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle, (u32, u64, bool)>`",
            &map,
        )
        .as_deref(),
        Some(
            "required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle`"
        )
    );
}

#[test]
fn parses_a_trait_bound_header() {
    // The classification parse the driver uses on its own: subject, whole bound, trait name.
    let parsed = parse_trait_bound(
        "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
    )
    .expect("a trait-bound header must parse");
    assert_eq!(parsed.subject, "Rectangle");
    assert_eq!(
        parsed.bound,
        "Rectangle: CanUseComponent<AreaCalculatorComponent>"
    );
    assert_eq!(parsed.trait_name, "CanUseComponent");

    // A bound without generics parses too, with empty args.
    let parsed = parse_trait_bound("the trait bound `f64: std::cmp::Eq` is not satisfied")
        .expect("an ordinary bound must parse");
    assert_eq!(parsed.subject, "f64");
    assert_eq!(parsed.bound, "f64: std::cmp::Eq");
    assert_eq!(parsed.trait_name, "Eq");
    assert_eq!(parsed.args, "");

    assert!(parse_trait_bound("mismatched types").is_none());
}

#[test]
fn header_leaves_non_cgp_and_unknown_bounds_untouched() {
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `f64: std::cmp::Eq` is not satisfied",
            &name_map()
        ),
        None
    );
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `Rectangle: CanUseComponent<UnknownComponent>` is not satisfied",
            &name_map(),
        ),
        None
    );
}

#[test]
fn rewrite_message_dispatches_note_and_header() {
    // The entry point handles both the header and the note forms.
    assert_eq!(
        rewrite_message(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`"
        )
    );
    assert_eq!(
        rewrite_message(
            "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`",
            &name_map(),
        )
        .as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
    assert_eq!(rewrite_message("some unrelated message", &name_map()), None);
}

/// A map keyed by a marker's *full path*, the shape the driver builds from `def_path_str`.
fn full_path_names() -> HashMap<String, ComponentTraitNames> {
    let mut map = HashMap::new();
    map.insert(
        "my_crate::area::AreaCalculatorComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanCalculateArea".to_owned(),
            provider: "AreaCalculator".to_owned(),
        },
    );
    map
}

#[test]
fn resolves_a_full_path_key_by_name() {
    let map = ComponentNameMap::new(full_path_names);

    // The text rewrite's name lookup matches the full-path key by its last segment, so a note
    // that carries only the unqualified marker still rewrites.
    assert_eq!(
        map.get("AreaCalculatorComponent").map(|n| n.consumer),
        Some("CanCalculateArea".to_owned())
    );
    assert_eq!(
        rewrite_required_for(
            "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`",
            &map,
        )
        .as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
}

#[test]
fn rewrites_a_wiring_overflow_into_its_cycle_header() {
    let out = rewrite_wiring_overflow(
        "overflow evaluating the requirement `Person: CanUseComponent<AreaCalculatorComponent>`",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E010] the wiring for the consumer trait `CanCalculateArea` on context `Person` never resolves — the lookup recurses without terminating"
        )
    );
}

#[test]
fn a_non_wiring_overflow_is_left_alone() {
    // An `E0275` on an ordinary trait is not a wiring cycle, so it keeps rustc's own header.
    assert_eq!(
        rewrite_wiring_overflow(
            "overflow evaluating the requirement `Foo: Bar<Baz>`",
            &name_map(),
        ),
        None
    );
}

/// A `ComponentNameMap` whose initializer panics, to prove it is never forced.
fn panicking_init() -> HashMap<String, ComponentTraitNames> {
    panic!("the name map must not be built when no message is rewritten");
}

#[test]
fn does_not_force_the_map_without_a_matching_message() {
    // The lazy build must not run for a message that is not a CGP wiring form — even one that
    // is a trait-bound header for an unrelated trait, or a `required for` note for a
    // non-wiring trait. If any of these forced the map, `panicking_init` would panic.
    let map = ComponentNameMap::new(panicking_init);
    assert_eq!(rewrite_message("some unrelated message", &map), None);
    assert_eq!(
        rewrite_message("the trait bound `f64: std::cmp::Eq` is not satisfied", &map),
        None
    );
    assert_eq!(
        rewrite_required_for(
            "required for `Rectangle` to implement `HasRectangleFields`",
            &map
        ),
        None
    );
}
