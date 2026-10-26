//! The syntax-tree resugaring (this crate) and the diagnostic text resugaring
//! (`cargo-cgp-error-processing`) agree on every construct both can show.
//!
//! The two implementations share no code, because one reads a parsed type and the other a rendered
//! string, so nothing but this test keeps their spellings in step. Each case feeds the same type to
//! both: the text pass through the chain an untransformed diagnostic runs, which spells a path in
//! its `Path!(@…)` source form, and this crate's pass as the right-hand side of a type alias. The
//! typed pass inside the driver needs the compiler, so the UI fixtures cover it instead.

use cargo_cgp_error_processing::{AmbiguousNames, Origin, postprocess_message};
use cargo_cgp_expand::{ExpandOptions, resugar_expanded_source};

/// The type as the diagnostic text resugaring shows it.
fn text_pass(raw: &str) -> String {
    postprocess_message(
        raw,
        false,
        Origin::Untransformed,
        &AmbiguousNames::new(Vec::new(), Vec::new()),
    )
    .unwrap_or_else(|| raw.to_owned())
}

/// The type as this crate's resugaring prints it, read back out of `type T = …;`.
fn syntax_tree_pass(raw: &str) -> String {
    let printed = resugar_expanded_source(&format!("type T = {raw};\n"), &ExpandOptions::default());
    printed
        .strip_prefix("type T = ")
        .and_then(|rest| rest.strip_suffix(";\n"))
        .unwrap_or_else(|| panic!("unexpected expansion output: {printed:?}"))
        .to_owned()
}

/// A field tag `Symbol!(name)` written as its expansion.
fn symbol(name: &str) -> String {
    let mut spine = "Nil".to_owned();
    for c in name.chars().rev() {
        spine = format!("Chars<'{c}', {spine}>");
    }
    format!("Symbol<{}, {spine}>", name.len())
}

/// Every construct the passes resugar, as its expansion and the spelling both must produce.
fn cases() -> Vec<(String, &'static str)> {
    vec![
        (symbol("height"), "Symbol!(\"height\")"),
        ("Symbol<0, Nil>".to_owned(), "Symbol!(\"\")"),
        (
            "Cons<u64, Cons<String, Nil>>".to_owned(),
            "Product![u64, String]",
        ),
        (
            "Either<u64, Either<f64, Void>>".to_owned(),
            "Sum![u64, f64]",
        ),
        (
            "Cons<Either<u64, Void>, Nil>".to_owned(),
            "Product![Sum![u64]]",
        ),
        (
            format!("Cons<Multiply<{}>, Nil>", symbol("foo")),
            "Product![Multiply<Symbol!(\"foo\")>]",
        ),
        (
            format!(
                "PathCons<{}, PathCons<GreeterComponent, Nil>>",
                symbol("app")
            ),
            "Path!(@app.GreeterComponent)",
        ),
        (
            "PathCons<ItemEncoderComponent, PathCons<u64, Nil>>".to_owned(),
            "Path!(@ItemEncoderComponent.u64)",
        ),
        (
            "PathCons<ItemEncoderComponent, PathCons<Vec<u8>, Nil>>".to_owned(),
            "Path!(@ItemEncoderComponent.Vec<u8>)",
        ),
        (
            format!(
                "Cons<Field<{}, f64>, Cons<Field<{}, Vec<u8>>, Nil>>",
                symbol("width"),
                symbol("name")
            ),
            "Struct! { width: f64, name: Vec<u8> }",
        ),
        (
            "Cons<Field<Index<0>, u8>, Cons<Field<Index<1>, u16>, Nil>>".to_owned(),
            "Struct!(u8, u16)",
        ),
        (
            "Cons<Field<Index<0>, u8>, Nil>".to_owned(),
            "Product![Field<Index<0>, u8>]",
        ),
        (
            format!("Cons<Field<{}, u8>, Nil>", symbol("type")),
            "Struct! { r#type: u8 }",
        ),
        (
            format!("Cons<Field<{}, u8>, Nil>", symbol("a b")),
            "Product![Field<Symbol!(\"a b\"), u8>]",
        ),
        (
            format!(
                "Either<Field<{}, Nil>, Either<Field<{}, Cons<Field<{}, f64>, Nil>>, \
                 Either<Field<{}, Cons<Field<Index<0>, u8>, Cons<Field<Index<1>, u16>, Nil>>>, \
                 Either<Field<{}, f64>, Void>>>>",
                symbol("Empty"),
                symbol("Rect"),
                symbol("w"),
                symbol("Pair"),
                symbol("Circle")
            ),
            "Enum! { Empty, Rect { w: f64 }, Pair(u8, u16), Circle(f64) }",
        ),
        (
            "Either<Field<Index<0>, u8>, Void>".to_owned(),
            "Sum![Field<Index<0>, u8>]",
        ),
    ]
}

#[test]
fn both_passes_spell_every_construct_the_same() {
    for (raw, expected) in cases() {
        assert_eq!(text_pass(&raw), expected, "text pass on {raw}");
        assert_eq!(
            syntax_tree_pass(&raw),
            expected,
            "syntax-tree pass on {raw}"
        );
    }
}

#[test]
fn both_passes_decline_the_same_non_constructs() {
    // A wrong length, a foreign spine, an open list, and a bare terminator stay as written.
    for raw in [
        "Symbol<9, Chars<'a', Nil>>",
        "Symbol<1, Other<'a', Nil>>",
        "Cons<u8, Rest>",
        "Nil",
    ] {
        assert_eq!(text_pass(raw), raw, "text pass on {raw}");
        assert_eq!(syntax_tree_pass(raw), raw, "syntax-tree pass on {raw}");
    }
}

#[test]
fn neither_pass_folds_an_open_ended_path() {
    // A path whose tail is unknown has no `Path!` spelling, so both passes keep the chain and
    // resugar only its segments.
    let raw = format!("PathCons<{}, _>", symbol("app"));
    assert_eq!(text_pass(&raw), "PathCons<Symbol!(\"app\"), _>");
    assert_eq!(syntax_tree_pass(&raw), "PathCons<Symbol!(\"app\"), _>");
}
