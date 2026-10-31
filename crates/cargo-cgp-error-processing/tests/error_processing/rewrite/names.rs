//! Tests for looking a component marker's traits up by the name rustc printed.

use std::collections::HashMap;

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, ComponentTraitNames, rewrite_message, rewrite_required_for,
};

/// A map keyed by a marker's *full path*, the shape the driver builds from `def_path_str`.
fn full_path_names() -> HashMap<String, ComponentTraitNames> {
    let mut map = HashMap::new();
    map.insert(
        "my_crate::area::AreaCalculatorComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanCalculateArea".to_owned(),
            provider: "AreaCalculator".to_owned(),
            params: 0,
        },
    );
    map
}

// Property: `get` matches a full-path key by its trailing segments, so a bare marker still finds it.
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

/// Two components in different modules whose markers share the name `MeasurerComponent`.
fn same_name_names() -> HashMap<String, ComponentTraitNames> {
    let mut map = HashMap::new();
    for (module, suffix) in [("shapes_a", "A"), ("shapes_b", "B")] {
        map.insert(
            format!("my_crate::{module}::MeasurerComponent"),
            ComponentTraitNames {
                consumer: format!("CanMeasure{suffix}"),
                provider: format!("Measurer{suffix}"),
                params: 0,
            },
        );
    }
    map
}

// Property: when the keys a printed path matches stand for different traits, the lookup finds nothing; a qualified
// path that picks out one finds it.
#[test]
fn declines_a_marker_name_two_components_share() {
    // The bare name rustc printed matches both entries, and the text cannot tell which one it
    // meant, so the rename declines rather than pick one.
    let map = ComponentNameMap::new(same_name_names);
    assert_eq!(map.get("MeasurerComponent"), None);
    assert_eq!(
        rewrite_required_for(
            "required for `MeasureWidthA` to implement `IsProviderFor<MeasurerComponent, App>`",
            &map,
        ),
        None
    );
    // rustc qualifies a shared name, and the qualified path picks out one component.
    assert_eq!(
        rewrite_required_for(
            "required for `MeasureWidthA` to implement `IsProviderFor<shapes_a::MeasurerComponent, App>`",
            &map,
        )
        .as_deref(),
        Some(
            "required for the provider `MeasureWidthA` to implement the provider trait `MeasurerA` for the context `App`"
        )
    );
}

/// A `ComponentNameMap` whose initializer panics, to prove it is never forced.
fn panicking_init() -> HashMap<String, ComponentTraitNames> {
    panic!("the name map must not be built when no message is rewritten");
}

// Property: for all messages that are not a wiring form, the rewrite gives `None` without forcing the map.
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
