//! The names `#[cgp_auto_dispatch]` gives a method's helper and computer, which the driver pairs to
//! recognize two dispatch traits sharing a method name.

use cargo_cgp_error_processing::{dispatch_computer_name, dispatch_helper_method};

#[test]
fn a_helper_name_gives_its_method() {
    assert_eq!(dispatch_helper_method("__compute_area__"), Some("area"));
    assert_eq!(
        dispatch_helper_method("__compute_scaled_area__"),
        Some("scaled_area")
    );
}

#[test]
fn other_names_are_no_helper() {
    assert_eq!(dispatch_helper_method("__compute___"), None);
    assert_eq!(dispatch_helper_method("compute_area"), None);
    assert_eq!(dispatch_helper_method("__compute_area"), None);
    assert_eq!(dispatch_helper_method("ComputeArea"), None);
}

#[test]
fn the_computer_name_is_the_method_in_pascal_case() {
    assert_eq!(dispatch_computer_name("area"), "ComputeArea");
    assert_eq!(dispatch_computer_name("scaled_area"), "ComputeScaledArea");
    assert_eq!(dispatch_computer_name("to_HTML"), "ComputeToHTML");
}
