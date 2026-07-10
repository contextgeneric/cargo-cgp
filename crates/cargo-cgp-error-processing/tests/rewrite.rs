//! Tests for the compiler-free wiring-message rewrite (`cargo_cgp_error_processing::rewrite`).
//!
//! This crate links no compiler internals, so — unlike the driver that drives this logic —
//! the tests need no `rustc_private` gate and run on any toolchain. The rewrite is exercised
//! from a hand-built name map, the same shape the driver produces from a `TyCtxt`.

use std::collections::HashMap;

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, ComponentTraitNames, rewrite_message, rewrite_required_for,
    rewrite_trait_bound,
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
        Some("the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied")
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
            "the provider trait bound `RectangleArea: AreaCalculator<Rectangle>` is not satisfied"
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
            "the provider trait bound `RedirectLookup<App, Nil>: AreaCalculator<Rectangle>` is not satisfied"
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
        Some("the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied")
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
        Some("the consumer trait bound `Rectangle: CanCalculateArea<f64>` is not satisfied")
    );
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle, f64>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "the provider trait bound `RectangleArea: AreaCalculator<Rectangle, f64>` is not satisfied"
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
        Some("the consumer trait bound `Rectangle: CanCalculateArea<u32, u64>` is not satisfied")
    );
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle, (u32, u64)>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "the provider trait bound `RectangleArea: AreaCalculator<Rectangle, u32, u64>` is not satisfied"
        )
    );
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
        Some("the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied")
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
