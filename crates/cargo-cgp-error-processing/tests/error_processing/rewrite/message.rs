//! Tests for the wiring-message rewrite (`cargo_cgp_error_processing::rewrite`).
//!
//! This crate links no compiler internals, so, unlike the driver that drives this logic, the tests
//! need no `rustc_private` gate and run on any toolchain. The rewrite is exercised from a
//! hand-built name map, the same shape the driver produces from a `TyCtxt`.

use std::collections::HashMap;

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, ComponentTraitNames, rewrite_message, rewrite_required_for,
    rewrite_trait_bound,
};

use super::support::name_map;

// Property: for all `IsProviderFor` notes whose marker the map knows, the note names the provider
// trait and context.
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

// Property: a generated parameter is never called a context: the provider form leaves it out, and
// the consumer form names it plainly.
#[test]
fn a_generated_parameter_is_not_called_a_context() {
    // `__Context__` is the parameter a provider impl is generic over, standing for every context,
    // so the provider form leaves it out and the consumer form names it plainly.
    assert_eq!(
        rewrite_required_for(
            "required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, __Context__>`",
            &name_map(),
        )
        .as_deref(),
        Some("required for the provider `RectangleArea` to implement the provider trait `AreaCalculator`"),
    );
    assert_eq!(
        rewrite_required_for(
            "required for `__Context__` to implement `CanUseComponent<AreaCalculatorComponent>`",
            &name_map(),
        )
        .as_deref(),
        Some("required for `__Context__` to implement the consumer trait `CanCalculateArea`"),
    );
}

// Property: for all `CanUseComponent` notes whose marker the map knows, the note names the consumer
// trait.
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

// Property (assumed from the test's name): a note matches by the trait's last segment, whatever
// prefix rustc printed.
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

// Property (assumed from the test's name): a context's own generic arguments survive the top-level
// comma split.
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

// Property (assumed from the test's name): the provider note names the bare trait, leaving out the
// parameters after the context.
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

// Property (assumed from the test's name): the consumer note names the bare trait, leaving out the
// component's parameters.
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

// Property: for all markers absent from the map, the note gives `None`.
#[test]
fn leaves_unknown_component_untouched() {
    // A marker absent from the map means the names are unknown, so nothing is rewritten.
    let out = rewrite_required_for(
        "required for `Rectangle` to implement `CanUseComponent<UnknownComponent>`",
        &name_map(),
    );
    assert_eq!(out, None);
}

// Property: for all notes that are not one of the two wiring forms, the rewrite gives `None`.
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

// Property: for all `IsProviderFor` headers whose marker the map knows, the header carries
// `[CGP-E002]` and names the provider trait, the context, and the provider.
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

// Property (assumed from the test's name): a header's subject stays whole, whatever generic
// arguments it holds.
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

// Property (assumed from the test's name): a header matches by the trait's last segment, whatever
// prefix rustc printed.
#[test]
fn header_strips_module_prefix() {
    let out = rewrite_trait_bound(
        "the trait bound `RectangleArea: cgp::prelude::IsProviderFor<AreaCalculatorComponent, Rectangle>` is not satisfied",
        &name_map(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator` with context `Rectangle` is not implemented for provider `RectangleArea`"
        )
    );
}

// Property: a `CanUseComponent` header gives `None`, since the typed resolver words it.
#[test]
fn leaves_a_check_header_to_the_resolver() {
    // A check's `CanUseComponent` header is worded by the typed resolver, never by this rewrite.
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
            &name_map(),
        ),
        None
    );
}

// Property: a single parameter reattaches to the named trait as written.
#[test]
fn header_reattaches_a_single_generic_parameter() {
    // A generic component: `IsProviderFor<Marker, Context, f64>` recovers
    // `ProviderTrait<Context, f64>`.
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

// Property: two or more grouped parameters reattach unwrapped.
#[test]
fn header_unwraps_a_multi_parameter_tuple() {
    // Two or more parameters arrive grouped in a tuple, which is unwrapped so the reattached
    // list matches how the trait was written (`ProviderTrait<u32, u64>`, not `<(u32, u64)>`).
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

// Property (assumed from the test's name): the header reattaches three grouped parameters while
// both notes leave them out.
#[test]
fn rewrites_a_three_parameter_generic_component() {
    // A component with three generic parameters: they arrive grouped as `(u32, u64, bool)`
    // and are unwrapped in the header so the trait reads as written, while the notes name the
    // bare trait. The strings mirror what `generic_area_multi.rs` produces end to end.
    let map = name_map();

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

// Property: for all other bounds, and markers absent from the map, the header gives `None`.
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

// Property: `rewrite_message` rewrites both wiring forms and gives `None` for any other message.
#[test]
fn rewrite_message_dispatches_note_and_header() {
    // The entry point handles both the header and the note forms.
    assert_eq!(
        rewrite_message(
            "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle>` is not satisfied",
            &name_map(),
        )
        .as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator` with context `Rectangle` is not implemented for provider `RectangleArea`"
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

/// A component whose one parameter is a tuple (`CanFormatPair<(u32, u64)>`).
fn tuple_param_names() -> HashMap<String, ComponentTraitNames> {
    HashMap::from([(
        "FormatPairComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanFormatPair".to_owned(),
            provider: "PairFormatter".to_owned(),
            params: 1,
        },
    )])
}

// Property: a component with exactly one parameter keeps it whole, even when it is a tuple.
#[test]
fn keeps_a_single_tuple_parameter_whole() {
    // A one-parameter component whose parameter is a tuple arrives exactly as a two-parameter
    // component's grouped list does, so the component's arity decides: one parameter stays whole.
    let map = ComponentNameMap::new(tuple_param_names);
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `FormatBoth: IsProviderFor<FormatPairComponent, App, (u32, u64)>` is not satisfied",
            &map,
        )
        .as_deref(),
        Some(
            "[CGP-E002] the provider trait `PairFormatter<(u32, u64)>` with context `App` is not implemented for provider `FormatBoth`"
        )
    );
}

/// A two-parameter component, for a function-pointer parameter's `->`.
fn fn_pointer_names() -> HashMap<String, ComponentTraitNames> {
    HashMap::from([(
        "ApplyComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanApply".to_owned(),
            provider: "Applier".to_owned(),
            params: 2,
        },
    )])
}

// Property (assumed from the test's name): a grouped list splits at its own commas, and the `>` of
// a `->` closes no bracket.
#[test]
fn keeps_a_function_pointer_parameter_whole() {
    // The `>` of `->` closes no bracket, so the grouped list still splits at its own comma.
    let map = ComponentNameMap::new(fn_pointer_names);
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `ApplyTwice: IsProviderFor<ApplyComponent, App, (fn(u8) -> u8, u32)>` is not satisfied",
            &map,
        )
        .as_deref(),
        Some(
            "[CGP-E002] the provider trait `Applier<fn(u8) -> u8, u32>` with context `App` is not implemented for provider `ApplyTwice`"
        )
    );
}
