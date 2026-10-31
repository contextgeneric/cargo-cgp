//! Tests for the wording of a wiring lookup that never resolves.

use cargo_cgp_error_processing::{
    OverflowStep, WiringOverflow, rewrite_wiring_overflow, wiring_overflow_help,
};

use super::rewrite::support::name_map;

/// A cycle the walk closed at `UseContext`'s `Greeter` step.
fn use_context_cycle() -> WiringOverflow {
    WiringOverflow::Cycle {
        closing: Some(OverflowStep {
            provider: "UseContext".to_owned(),
            provider_trait: "Greeter".to_owned(),
            context: "Person".to_owned(),
            requires: "CanGreet".to_owned(),
            required_of: "Person".to_owned(),
        }),
    }
}

// Property: for all wiring requirements, `WiringOverflow::Cycle` gives the `[CGP-E020]` header.
#[test]
fn rewrites_a_wiring_cycle_into_its_cycle_header() {
    let out = rewrite_wiring_overflow(
        "overflow evaluating the requirement `Person: CanUseComponent<AreaCalculatorComponent>`",
        &name_map(),
        &use_context_cycle(),
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E020] the wiring for the consumer trait `CanCalculateArea` on context `Person` never resolves, because the lookup leads back to itself"
        )
    );
}

// Property: for all wiring requirements, `WiringOverflow::Growing` gives the `[CGP-E021]` header,
// naming the first larger requirement when known.
#[test]
fn rewrites_a_growing_lookup_into_its_own_header() {
    let growing = WiringOverflow::Growing {
        larger: Some("CanCalculateArea<Vec<u32>>".to_owned()),
        growing: None,
    };
    let out = rewrite_wiring_overflow(
        "overflow evaluating the requirement `Person: CanUseComponent<AreaCalculatorComponent>`",
        &name_map(),
        &growing,
    );
    assert_eq!(
        out.as_deref(),
        Some(
            "[CGP-E021] the wiring for the consumer trait `CanCalculateArea` on context `Person` never resolves, because each step requires it for a larger type (`CanCalculateArea<Vec<u32>>`, …)"
        )
    );
    // Without the first larger requirement, the reason names none.
    let unnamed = WiringOverflow::Growing {
        larger: None,
        growing: None,
    };
    assert_eq!(
        rewrite_wiring_overflow(
            "overflow evaluating the requirement `Person: CanUseComponent<AreaCalculatorComponent>`",
            &name_map(),
            &unnamed,
        )
        .as_deref(),
        Some(
            "[CGP-E021] the wiring for the consumer trait `CanCalculateArea` on context `Person` never resolves, because each step requires it for a larger type"
        )
    );
}

// Property: the help names the provider and its requirement when the walk found that step, and
// otherwise says what the replacement must not do.
#[test]
fn the_wiring_overflow_help_names_the_provider_that_closes_or_grows_the_lookup() {
    assert_eq!(
        wiring_overflow_help(&use_context_cycle()),
        "wire a provider in place of `UseContext`, which provides `Greeter` for `Person` by \
         requiring `CanGreet` on `Person` again"
    );
    assert_eq!(
        wiring_overflow_help(&WiringOverflow::Growing {
            larger: Some("CanShow<Vec<u32>>".to_owned()),
            growing: Some(OverflowStep {
                provider: "ShowNested".to_owned(),
                provider_trait: "Shower<u32>".to_owned(),
                context: "App".to_owned(),
                requires: "CanShow<Vec<u32>>".to_owned(),
                required_of: "App".to_owned(),
            }),
        }),
        "wire a provider in place of `ShowNested`, which provides `Shower<u32>` for `App` by \
         requiring `CanShow<Vec<u32>>` on `App`, a larger type at every step"
    );
    // A lookup that grows in the type the consumer trait is required of names that type.
    assert_eq!(
        wiring_overflow_help(&WiringOverflow::Growing {
            larger: Some("Vec<u32>".to_owned()),
            growing: Some(OverflowStep {
                provider: "ShowWrapped".to_owned(),
                provider_trait: "Shower".to_owned(),
                context: "u32".to_owned(),
                requires: "CanShow".to_owned(),
                required_of: "Vec<u32>".to_owned(),
            }),
        }),
        "wire a provider in place of `ShowWrapped`, which provides `Shower` for `u32` by requiring \
         `CanShow` on `Vec<u32>`, a larger type at every step"
    );
    // Without a provider step, the help says what the replacement must not do.
    assert_eq!(
        wiring_overflow_help(&WiringOverflow::Cycle { closing: None }),
        "wire the component to a provider that does not require the consumer trait it provides"
    );
    assert_eq!(
        wiring_overflow_help(&WiringOverflow::Growing {
            larger: Some("CanShow<Vec<u32>>".to_owned()),
            growing: None,
        }),
        "wire the component to a provider whose requirements do not grow with each step"
    );
}

// Property: a requirement on a consumer trait the map knows is a wiring lookup, with or without
// arguments.
#[test]
fn rewrites_a_consumer_overflow_at_a_call_into_its_cycle_header() {
    // A call reports the overflow on the consumer trait itself, with or without its arguments.
    assert_eq!(
        rewrite_wiring_overflow(
            "overflow evaluating the requirement `Person: CanCalculateArea`",
            &name_map(),
            &use_context_cycle(),
        )
        .as_deref(),
        Some(
            "[CGP-E020] the wiring for the consumer trait `CanCalculateArea` on context `Person` never resolves, because the lookup leads back to itself"
        )
    );
    assert_eq!(
        rewrite_wiring_overflow(
            "overflow evaluating the requirement `Person: app::CanCalculateArea<Square>`",
            &name_map(),
            &use_context_cycle(),
        )
        .as_deref(),
        Some(
            "[CGP-E020] the wiring for the consumer trait `CanCalculateArea<Square>` on context `Person` never resolves, because the lookup leads back to itself"
        )
    );
}

// Property: for all overflows on a trait that is neither `CanUseComponent` nor a known consumer
// trait, the rewrite gives `None`.
#[test]
fn a_non_wiring_overflow_is_left_alone() {
    // An `E0275` on an ordinary trait is not a wiring lookup, so it keeps rustc's own header.
    assert_eq!(
        rewrite_wiring_overflow(
            "overflow evaluating the requirement `Foo: Bar<Baz>`",
            &name_map(),
            &use_context_cycle(),
        ),
        None
    );
}
