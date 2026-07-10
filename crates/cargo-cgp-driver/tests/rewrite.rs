//! Tests for the compiler-free message rewrite ([`cargo_cgp_driver::rewrite`]).
//!
//! Like the driver binary, this links `cargo-cgp-driver` (which links `rustc_driver`), so
//! it carries the `#![feature(rustc_private)]` gate. The rewrite itself touches no compiler
//! API — it is driven here from a hand-built name map, the same shape
//! [`cargo_cgp_driver::component_map`] produces from a `TyCtxt`.

#![feature(rustc_private)]

use std::collections::HashMap;

use cargo_cgp_driver::rewrite::{
    ComponentTraitNames, is_cgp_wiring_message, is_trait_bound_header, is_wiring_note,
    rewrite_message, rewrite_required_for, rewrite_trait_bound,
};

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

#[test]
fn rewrites_is_provider_for_note() {
    let out = rewrite_required_for(
        "required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`",
        &names(),
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
        &names(),
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
        &names(),
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
        &names(),
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
        &names(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle`"
        )
    );
}

#[test]
fn leaves_unknown_component_untouched() {
    // A marker absent from the map means the names are unknown, so nothing is rewritten.
    let out = rewrite_required_for(
        "required for `Rectangle` to implement `CanUseComponent<UnknownComponent>`",
        &names(),
    );
    assert_eq!(out, None);
}

#[test]
fn leaves_unrelated_notes_untouched() {
    assert_eq!(
        rewrite_required_for(
            "required for `Rectangle` to implement `HasRectangleFields`",
            &names(),
        ),
        None
    );
    assert_eq!(
        rewrite_required_for("required by a bound in `__CheckRectangle`", &names()),
        None
    );
}

#[test]
fn is_wiring_note_matches_only_the_two_forms() {
    assert!(is_wiring_note(
        "required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`"
    ));
    assert!(is_wiring_note(
        "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`"
    ));
    assert!(!is_wiring_note(
        "required for `Rectangle` to implement `HasRectangleFields`"
    ));
    assert!(!is_wiring_note(
        "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied"
    ));
}

#[test]
fn rewrites_can_use_component_header() {
    let out = rewrite_trait_bound(
        "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
        &names(),
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
        &names(),
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
        &names(),
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
        &names(),
    );
    assert_eq!(
        out.as_deref(),
        Some("the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied")
    );
}

#[test]
fn header_leaves_parameterized_form_untouched() {
    // A component with extra generic parameters would reduce to an inaccurate bound, so the
    // header is left raw rather than dropping the parameters.
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent, (u32, u64)>` is not satisfied",
            &names(),
        ),
        None
    );
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle, (u32, u64)>` is not satisfied",
            &names(),
        ),
        None
    );
}

#[test]
fn header_leaves_non_cgp_and_unknown_bounds_untouched() {
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `f64: std::cmp::Eq` is not satisfied",
            &names()
        ),
        None
    );
    assert_eq!(
        rewrite_trait_bound(
            "the trait bound `Rectangle: CanUseComponent<UnknownComponent>` is not satisfied",
            &names(),
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
            &names(),
        )
        .as_deref(),
        Some("the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied")
    );
    assert_eq!(
        rewrite_message(
            "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`",
            &names(),
        )
        .as_deref(),
        Some(
            "required for the context `Rectangle` to implement the consumer trait `CanCalculateArea`"
        )
    );
    assert_eq!(rewrite_message("some unrelated message", &names()), None);
}

#[test]
fn is_cgp_wiring_message_matches_notes_and_headers() {
    assert!(is_trait_bound_header(
        "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied"
    ));
    assert!(is_cgp_wiring_message(
        "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied"
    ));
    assert!(is_cgp_wiring_message(
        "required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`"
    ));
    assert!(!is_cgp_wiring_message(
        "the trait bound `f64: std::cmp::Eq` is not satisfied"
    ));
}
