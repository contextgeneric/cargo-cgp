//! The pure dependency-tree label constructors and the text signals, over plain strings.

use cargo_cgp_error_processing::{
    consumer_impl_label, elide_repeated_generics, field_impl_label, is_method_bounds_text,
    is_method_probe_advice_text, is_question_mark_cascade_text, mentions_wiring_text,
    provider_impl_label, redirect_label, trait_impl_label,
};

#[test]
fn each_label_template_carries_its_code() {
    assert_eq!(
        consumer_impl_label("CanCalculateArea<f64>", "Rectangle"),
        "[CGP-E101] consumer trait impl `CanCalculateArea<f64>` for context `Rectangle`"
    );
    assert_eq!(
        provider_impl_label("AreaCalculator", "Rectangle", "RectangleArea"),
        "[CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`"
    );
    assert_eq!(
        field_impl_label("height", "Rectangle"),
        "[CGP-E103] field trait impl `HasField` with field `height` for `Rectangle`"
    );
    assert_eq!(
        redirect_label("@app.GreeterComponent", "App"),
        "[CGP-E104] redirect lookup to `@app.GreeterComponent` in `App`"
    );
    assert_eq!(
        trait_impl_label("HasName", "App"),
        "[CGP-E105] trait impl `HasName` for `App`"
    );
}

#[test]
fn wiring_signal_matches_each_wiring_trait_and_nothing_else() {
    assert!(mentions_wiring_text(
        "the trait bound `App: CanUseComponent<GreeterComponent>` is not satisfied"
    ));
    assert!(mentions_wiring_text(
        "required for `GreetHello` to implement `IsProviderFor<GreeterComponent, App>`"
    ));
    assert!(mentions_wiring_text(
        "doesn't satisfy `_: HasField<Symbol!(\"name\")>`"
    ));
    assert!(!mentions_wiring_text("mismatched types"));
}

#[test]
fn method_bounds_signal_tells_the_two_e0599_shapes_apart() {
    assert!(is_method_bounds_text(
        "the method `greet` exists for struct `Person`, but its trait bounds were not satisfied"
    ));
    // A resolution-class `E0599` must not match: running the resolver on it re-enters the
    // diagnostic context.
    assert!(!is_method_bounds_text(
        "no variant named `Blue` found for enum `Color`"
    ));
}

#[test]
fn a_hop_repeating_its_predecessors_generics_is_elided() {
    let labels = vec![
        consumer_impl_label("CanHandle<Prog<Product![A, B]>, _>", "App"),
        redirect_label("@cgp.extra.handler.HandlerComponent", "App"),
        provider_impl_label(
            "Handler<Prog<Product![A, B]>, _>",
            "App",
            "PipeHandlers<Product![A, B]>",
        ),
        provider_impl_label(
            "Handler<Prog<Product![A, B]>, _>",
            "App",
            "ComposeHandlers<A, B>",
        ),
        provider_impl_label("Handler<Prog<Product![A, B]>, _>", "App", "A"),
        trait_impl_label("HasName", "App"),
    ];

    let elided = elide_repeated_generics(labels);

    // The first `Handler` hop keeps the full parameters (its predecessor is the redirect, whose
    // quoted segment differs); the second and third — repeating it exactly — elide. The consumer
    // (a different trait) and the generic-less labels are untouched.
    assert!(elided[0].contains("`CanHandle<Prog<Product![A, B]>, _>`"));
    assert!(elided[2].contains("`Handler<Prog<Product![A, B]>, _>`"));
    assert!(elided[3].contains("`Handler<…>`"));
    assert!(elided[3].contains("`ComposeHandlers<A, B>`"));
    assert!(elided[4].contains("`Handler<…>`"));
    assert_eq!(elided[5], trait_impl_label("HasName", "App"));
}

#[test]
fn a_hop_whose_generics_change_keeps_its_full_form() {
    let labels = vec![
        provider_impl_label("ValueEncoder<Outer>", "App", "EncodeRecord"),
        provider_impl_label("ValueEncoder<Vec<Mid>>", "App", "EncodeIterator"),
    ];
    assert_eq!(elide_repeated_generics(labels.clone()), labels);
}

#[test]
fn method_probe_advice_signal_matches_each_artifact() {
    assert!(is_method_probe_advice_text(
        "this is an associated function, not a method"
    ));
    assert!(is_method_probe_advice_text(
        "found the following associated functions; to be used as methods, functions must have a `self` parameter"
    ));
    assert!(is_method_probe_advice_text(
        "the candidate is defined in the trait `PairFormatter`"
    ));
    assert!(!is_method_probe_advice_text(
        "trait bound `App: HasField<Symbol!(\"separator\")>` was not satisfied"
    ));
}

#[test]
fn question_mark_signal_matches_rustc_try_wording() {
    assert!(is_question_mark_cascade_text(
        "the `?` operator can only be applied to values that implement `Try`"
    ));
    assert!(!is_question_mark_cascade_text("mismatched types"));
}
