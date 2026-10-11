//! Tests for narrowing an expansion to one module or item
//! ([`cargo_cgp_expand::select`]).

use cargo_cgp_expand::{ExpandOptions, ItemPath, resugar_expanded_source};

/// Expand a snippet narrowed to `item`.
fn expand_item(source: &str, item: &str) -> String {
    let options = ExpandOptions {
        item: Some(ItemPath::parse(item).expect("a valid item path")),
        ..ExpandOptions::default()
    };
    resugar_expanded_source(source, &options)
}

/// A small two-module program in the shape a CGP expansion takes: a trait, impls of it, a struct,
/// and impls for that struct.
const PROGRAM: &str = "\
pub trait AreaCalculator<Context> {
    fn area(context: &Context) -> f64;
}
impl<Context> AreaCalculator<Context> for UseContext {
    fn area(context: &Context) -> f64 {
        0.0
    }
}
pub mod shapes {
    pub struct Rectangle {
        pub width: f64,
    }
    impl HasField<Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>>
    for Rectangle {
        type Value = f64;
    }
    impl<Context> AreaCalculator<Context> for RectangleArea {
        fn area(context: &Context) -> f64 {
            1.0
        }
    }
}
pub struct Circle;
";

#[test]
fn a_module_selects_its_contents() {
    let out = expand_item(PROGRAM, "shapes");

    // The `mod shapes { … }` wrapper is noise around what was asked for, so the contents come out
    // unwrapped — and resugared like any other expansion.
    assert!(!out.contains("mod shapes"), "{out}");
    assert!(out.contains("pub struct Rectangle"), "{out}");
    assert!(
        out.contains("impl HasField<Symbol!(\"width\")> for Rectangle"),
        "{out}"
    );
    assert!(out.contains("for RectangleArea"), "{out}");
    // Nothing from outside the module.
    assert!(!out.contains("struct Circle"), "{out}");
    assert!(!out.contains("for UseContext"), "{out}");
}

#[test]
fn a_type_selects_its_declaration_and_the_impls_for_it() {
    let out = expand_item(PROGRAM, "shapes::Rectangle");

    assert!(out.contains("pub struct Rectangle"), "{out}");
    assert!(
        out.contains("impl HasField<Symbol!(\"width\")> for Rectangle"),
        "{out}"
    );
    // An impl inside the same module but *for* something else is not about `Rectangle`.
    assert!(!out.contains("for RectangleArea"), "{out}");
}

#[test]
fn an_unqualified_path_reaches_into_a_module() {
    // A reader who says `Rectangle` means the one there is, wherever it sits.
    let out = expand_item(PROGRAM, "Rectangle");

    assert!(out.contains("pub struct Rectangle"), "{out}");
}

#[test]
fn a_trait_selects_the_impls_of_it() {
    // The CGP-shaped case: the generated items for a component are its provider trait and the impls
    // of it, which is what a reader has in mind when they name the trait.
    let out = expand_item(PROGRAM, "AreaCalculator");

    assert!(out.contains("pub trait AreaCalculator<Context>"), "{out}");
    assert!(out.contains("for UseContext"), "{out}");
    assert!(out.contains("for RectangleArea"), "{out}");
    assert!(!out.contains("struct Circle"), "{out}");
}

#[test]
fn nothing_matching_yields_nothing() {
    // Never the whole crate: that is not what was asked for, and the caller reports the miss.
    assert!(expand_item(PROGRAM, "shapes::Triangle").is_empty());
    assert!(expand_item(PROGRAM, "nowhere::at::all").is_empty());
}

#[test]
fn a_crate_root_prefix_is_accepted() {
    // `crate::contexts::app` is how the module is spelled in the source, so it is what a reader
    // reaches for; matching is against paths within the crate, which carry no such prefix.
    for spelling in [
        "shapes::Rectangle",
        "crate::shapes::Rectangle",
        "::shapes::Rectangle",
        "self::shapes::Rectangle",
    ] {
        let out = expand_item(PROGRAM, spelling);
        assert!(
            out.contains("pub struct Rectangle"),
            "`{spelling}` should reach the same module: {out}"
        );
    }
}

#[test]
fn an_item_path_must_be_identifiers() {
    assert!(ItemPath::parse("shapes::Rectangle").is_some());
    assert!(ItemPath::parse("Rectangle").is_some());
    assert!(ItemPath::parse("_private::thing").is_some());

    assert!(ItemPath::parse("crate::shapes").is_some());
    assert!(ItemPath::parse("::shapes").is_some());

    // Declining a malformed path is what keeps a typo from looking like an item that is not there.
    assert!(ItemPath::parse("shapes::").is_none());
    // A prefix with nothing after it names nothing.
    assert!(ItemPath::parse("crate").is_none());
    assert!(ItemPath::parse("crate::").is_none());
    assert!(ItemPath::parse("::").is_none());
    assert!(ItemPath::parse("shapes:Rectangle").is_none());
    assert!(ItemPath::parse("shapes::<T>").is_none());
    assert!(ItemPath::parse("not a path").is_none());
    assert!(ItemPath::parse("").is_none());
}

#[test]
fn the_whole_crate_is_expanded_without_a_filter() {
    let out = resugar_expanded_source(PROGRAM, &ExpandOptions::default());

    assert!(out.contains("mod shapes"), "{out}");
    assert!(out.contains("struct Circle"), "{out}");
}
