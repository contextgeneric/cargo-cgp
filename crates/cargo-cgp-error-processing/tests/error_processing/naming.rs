//! Tests for the rule CGP macros follow for the names they invent.

use cargo_cgp_error_processing::{is_reserved_name, reserved_names};

// Property: `is_reserved_name` holds exactly for two underscores at each end of a non-empty name
// that neither starts nor ends with an underscore of its own.
#[test]
fn a_reserved_name_has_two_underscores_at_each_end() {
    assert!(is_reserved_name("__Context__"));
    assert!(is_reserved_name("__a0__"));
    assert!(is_reserved_name("__compute_area__"));
    // A name the program wrote: a pair at one end only, a pair inside, a third underscore at an
    // end, or underscores alone.
    assert!(!is_reserved_name("__init"));
    assert!(!is_reserved_name("snake__case"));
    assert!(!is_reserved_name("___Context__"));
    assert!(!is_reserved_name("__Context___"));
    assert!(!is_reserved_name("____"));
    assert!(!is_reserved_name("Context"));
}

// Property: `reserved_names` gives exactly the words of `text` that `is_reserved_name` accepts,
// without a lifetime's quote.
#[test]
fn reserved_names_are_the_double_underscore_words() {
    // A generated lifetime, without its quote, and a generated type parameter.
    assert_eq!(reserved_names("Cow<'__a__, str>"), vec!["__a__"]);
    assert_eq!(
        reserved_names("'__a0__ and `__Context__`"),
        vec!["__a0__", "__Context__"]
    );
    assert_eq!(
        reserved_names("__compute_area__()"),
        vec!["__compute_area__"]
    );
    // A name with underscores only inside, a leading pair alone, a word that merely contains a
    // pair or ends in one, a triple underscore, uneven ends, and underscores alone are not
    // reserved names.
    assert!(reserved_names("snake_case_name").is_empty());
    assert!(reserved_names("__private").is_empty());
    assert!(reserved_names("my__a__b").is_empty());
    assert!(reserved_names("my__a__").is_empty());
    assert!(reserved_names("___x___").is_empty());
    assert!(reserved_names("___a_").is_empty());
    assert!(reserved_names("__a___").is_empty());
    assert!(reserved_names("___a__").is_empty());
    assert!(reserved_names("____").is_empty());
    assert!(reserved_names("______").is_empty());
    assert!(reserved_names("'_ and _").is_empty());
}
