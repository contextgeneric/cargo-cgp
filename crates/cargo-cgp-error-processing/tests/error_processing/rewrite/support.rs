//! The name map the rewrite's tests and the overflow tests share.

use std::collections::HashMap;

use cargo_cgp_error_processing::rewrite::{ComponentNameMap, ComponentTraitNames};

/// The fixed map every test's [`ComponentNameMap`] initializes to.
pub fn names() -> HashMap<String, ComponentTraitNames> {
    let mut map = HashMap::new();
    map.insert(
        "AreaCalculatorComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanCalculateArea".to_owned(),
            provider: "AreaCalculator".to_owned(),
            params: 0,
        },
    );
    map
}

/// A lazily-initialized map over [`names`], the driver-side `ComponentNameMap` stand-in.
pub fn name_map() -> ComponentNameMap {
    ComponentNameMap::new(names)
}
