//! Tests for bounding the implementor list the driver's `--verbose` flag un-elides.

use cargo_cgp_error_processing::bound_candidate_list;

/// rustc's candidate-list help with `count` implementors, as a verbose session prints it.
fn candidate_list(heading: &str, count: usize) -> String {
    let mut text = heading.to_owned();
    for index in 0..count {
        text.push_str(&format!(
            "\n  `Provider{index}` implements `IsProviderFor<C{index}, Context>`"
        ));
    }
    text
}

// Property: for all candidate-list helps longer than rustc prints whole, the result keeps the
//   heading, the first `RUSTC_CANDIDATE_LIMIT` candidates, and "and N others" for the rest
//   (`bound_candidate_list`'s doc).
#[test]
fn a_long_list_is_cut_to_eight_and_a_count() {
    let heading =
        "the following other types implement trait `IsProviderFor<Component, Context, Params>`:";
    let bounded = bound_candidate_list(&candidate_list(heading, 12)).unwrap();
    assert_eq!(bounded.lines().count(), 10);
    assert!(bounded.starts_with(heading));
    assert!(bounded.contains("`Provider7`"));
    assert!(!bounded.contains("`Provider8`"));
    assert!(bounded.ends_with("\nand 4 others"));
}

// Property: a list of nine candidates or fewer, which rustc prints whole, is left alone
//   (`bound_candidate_list`'s doc).
#[test]
fn a_list_rustc_prints_whole_is_left_alone() {
    let heading = "the following types implement trait `Greeter<Context>`:";
    assert_eq!(bound_candidate_list(&candidate_list(heading, 9)), None);
}

// Property: the heading rustc uses when every candidate shares one type opens a candidate list too,
//   so its list is cut the same way (`is_candidate_heading`'s doc).
#[test]
fn the_shared_type_heading_is_bounded_too() {
    let heading = "`Rectangle` implements trait `HasField<Tag>`:";
    let bounded = bound_candidate_list(&candidate_list(heading, 10)).unwrap();
    assert!(bounded.ends_with("\nand 2 others"));
}

// Property: a help that is no candidate list is left alone (`bound_candidate_list`'s doc).
#[test]
fn any_other_help_is_left_alone() {
    assert_eq!(
        bound_candidate_list("consider restricting type parameter `Value`:\n  `Value: Describe`"),
        None
    );
}
