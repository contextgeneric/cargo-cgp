//! Tests for the resugaring passes, over hand-written expanded source.
//!
//! Each case is the source a CGP macro's expansion prints, run through the whole pipeline the way
//! the driver runs it, so what is asserted is what a reader of `cargo cgp expand` sees.

use cargo_cgp_expand::{ExpandOptions, resugar_expanded_source};

/// Resugar a snippet the way the driver does, the prelude qualifier stripped first.
fn expand(source: &str) -> String {
    resugar_expanded_source(source, &ExpandOptions::default())
}

#[test]
fn resugars_a_symbol_spine_to_its_literal() {
    let source = "\
impl HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<'h', Chars<'t', Nil>>>>>>>>
for Rectangle {}
";

    assert!(
        expand(source).contains(r#"impl HasField<Symbol!("height")> for Rectangle {}"#),
        "{}",
        expand(source)
    );
}

#[test]
fn resugars_the_empty_symbol() {
    let source = "type Empty = Symbol<0, Nil>;\n";

    assert_eq!(expand(source), "type Empty = Symbol!(\"\");\n");
}

#[test]
fn declines_a_symbol_whose_length_disagrees() {
    // The length `Symbol!` bakes in is the string's byte length, so a mismatch means this is not
    // a `Symbol!` expansion and must be left exactly as it stands.
    let source = "type Wrong = Symbol<9, Chars<'a', Nil>>;\n";

    assert_eq!(expand(source), source);
}

#[test]
fn declines_a_symbol_whose_spine_is_not_chars() {
    let source = "type Foreign = Symbol<1, Other<'a', Nil>>;\n";

    assert_eq!(expand(source), source);
}

#[test]
fn resugars_a_product_of_bare_types() {
    let source = "type Stages = PipeHandlers<Cons<StepOne, Cons<StepTwo, Nil>>>;\n";

    assert_eq!(
        expand(source),
        "type Stages = PipeHandlers<Product![StepOne, StepTwo]>;\n"
    );
}

#[test]
fn resugars_a_sum_of_bare_types() {
    let source = "type Values = Either<u64, Either<f64, Void>>;\n";

    assert_eq!(expand(source), "type Values = Sum![u64, f64];\n");
}

#[test]
fn resugars_a_nested_list() {
    let source = "type Nested = Cons<Either<u64, Void>, Nil>;\n";

    assert_eq!(expand(source), "type Nested = Product![Sum![u64]];\n");
}

#[test]
fn folds_a_named_field_list_to_a_struct_shape() {
    // The printer breaks a brace body onto lines of its own; the spacing pass joins it back.
    let source = "\
type Fields = Cons<
    Field<Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>, f64>,
    Cons<Field<Symbol<4, Chars<'n', Chars<'a', Chars<'m', Chars<'e', Nil>>>>>, Vec<u8>>, Nil>,
>;
";

    assert_eq!(
        expand(source),
        "type Fields = Struct! { width: f64, name: Vec<u8> };\n"
    );
}

#[test]
fn folds_an_index_field_list_to_a_tuple_struct_shape() {
    let source = "type Fields = Cons<Field<Index<0>, u8>, Cons<Field<Index<1>, Vec<u8>>, Nil>>;\n";

    assert_eq!(expand(source), "type Fields = Struct!(u8, Vec<u8>);\n");
}

#[test]
fn a_single_index_field_stays_a_product() {
    // `Struct!(u8)` is the bare `u8`, so this one-element list has no `Struct!` spelling.
    let source = "type Fields = Cons<Field<Index<0>, u8>, Nil>;\n";

    assert_eq!(
        expand(source),
        "type Fields = Product![Field<Index<0>, u8>];\n"
    );
}

#[test]
fn a_keyword_field_name_is_written_raw() {
    let source = "type Fields = Cons<Field<Symbol<4, Chars<'t', Chars<'y', Chars<'p', Chars<'e', Nil>>>>>, u8>, Nil>;\n";

    assert_eq!(expand(source), "type Fields = Struct! { r#type: u8 };\n");
}

#[test]
fn a_field_name_that_is_not_an_identifier_stays_a_product() {
    let source =
        "type Fields = Cons<Field<Symbol<3, Chars<'a', Chars<' ', Chars<'b', Nil>>>>, u8>, Nil>;\n";

    assert_eq!(
        expand(source),
        "type Fields = Product![Field<Symbol!(\"a b\"), u8>];\n"
    );
}

#[test]
fn folds_a_variant_list_to_an_enum_shape_in_each_variant_spelling() {
    // A `Nil` payload is a unit variant, a named-field payload lends the variant its braces, an
    // `Index` payload its parentheses, and any other payload is the single positional field.
    let source = "\
type Fields = Either<
    Field<Symbol<5, Chars<'E', Chars<'m', Chars<'p', Chars<'t', Chars<'y', Nil>>>>>>, Nil>,
    Either<
        Field<Symbol<4, Chars<'R', Chars<'e', Chars<'c', Chars<'t', Nil>>>>>, Cons<Field<Symbol<1, Chars<'w', Nil>>, f64>, Nil>>,
        Either<
            Field<Symbol<4, Chars<'P', Chars<'a', Chars<'i', Chars<'r', Nil>>>>>, Cons<Field<Index<0>, u8>, Cons<Field<Index<1>, u16>, Nil>>>,
            Either<Field<Symbol<6, Chars<'C', Chars<'i', Chars<'r', Chars<'c', Chars<'l', Chars<'e', Nil>>>>>>>, f64>, Void>,
        >,
    >,
>;
";

    assert_eq!(
        expand(source),
        "type Fields = Enum! { Empty, Rect { w: f64 }, Pair(u8, u16), Circle(f64) };\n"
    );
}

#[test]
fn a_variant_list_tagged_by_position_stays_a_sum() {
    let source = "type Fields = Either<Field<Index<0>, u8>, Void>;\n";

    assert_eq!(expand(source), "type Fields = Sum![Field<Index<0>, u8>];\n");
}

#[test]
fn resugars_a_generic_element_tightly() {
    // The printer spaces a macro body's tokens apart, so this is what the spacing pass exists for:
    // without it the elements read `Multiply < Symbol!("foo") >`.
    let source = "\
type Stages = PipeHandlers<
    Cons<Multiply<Symbol<3, Chars<'f', Chars<'o', Chars<'o', Nil>>>>>, Nil>,
>;
";

    assert_eq!(
        expand(source),
        "type Stages = PipeHandlers<Product![Multiply<Symbol!(\"foo\")>]>;\n"
    );
}

#[test]
fn declines_a_spine_that_does_not_terminate() {
    let source = "type Open = Cons<u8, Rest>;\n";

    assert_eq!(expand(source), source);
}

#[test]
fn leaves_a_bare_terminator_as_its_type() {
    // An empty list reads as the terminator it is; resugaring it would claim an empty list where
    // a plain type was written.
    let source = "type Nothing = Nil;\n";

    assert_eq!(expand(source), source);
}

#[test]
fn resugars_a_namespace_path() {
    let source = "\
type Route = RedirectLookup<
    App,
    PathCons<Symbol<3, Chars<'a', Chars<'p', Chars<'p', Nil>>>>, PathCons<GreeterComponent, Nil>>,
>;
";

    assert_eq!(
        expand(source),
        "type Route = RedirectLookup<App, Path!(@app.GreeterComponent)>;\n"
    );
}

#[test]
fn resugars_an_open_dispatch_path_with_a_compound_value_segment() {
    let source = "type Key = PathCons<ItemEncoderComponent, PathCons<Vec<u8>, Nil>>;\n";

    assert_eq!(
        expand(source),
        "type Key = Path!(@ItemEncoderComponent.Vec<u8>);\n"
    );
}

#[test]
fn resugars_a_primitive_path_segment_as_a_type() {
    // Every primitive segment is kept as the named type by `Path!`, so each round-trips bare.
    for primitive in PRIMITIVE_NAMES {
        let source =
            format!("type Key = PathCons<ItemEncoderComponent, PathCons<{primitive}, Nil>>;\n");

        assert_eq!(
            expand(&source),
            format!("type Key = Path!(@ItemEncoderComponent.{primitive});\n"),
            "{primitive}",
        );
    }
}

/// The names `Path!` keeps as types, as `cgp`'s path segment rule lists them.
const PRIMITIVE_NAMES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "char", "bool", "str",
];

#[test]
fn declines_a_path_segment_named_like_a_primitive() {
    // `f`, `u2`, and `i7` look like primitives but are none, so `Path!` would read each as a
    // symbol; a type of that name has no `Path!` spelling, and the chain is left as it stands.
    for near_miss in ["f", "u2", "i7"] {
        let source =
            format!("type Key = PathCons<ItemEncoderComponent, PathCons<{near_miss}, Nil>>;\n");

        assert_eq!(expand(&source), source, "{near_miss}");
    }
}

#[test]
fn declines_an_open_tailed_path() {
    // An open-ended path has no `Path!` spelling, so the chain is left as it stands, as the
    // diagnostic text pass leaves it.
    let source =
        "type Route = PathCons<Symbol<3, Chars<'a', Chars<'p', Chars<'p', Nil>>>>, Rest>;\n";

    // The chain is left whole; its symbol segment still resugars, since that pass is independent
    // of the path fold — which is what a declined path shows in a diagnostic too.
    assert_eq!(
        expand(source),
        "type Route = PathCons<Symbol!(\"app\"), Rest>;\n"
    );
}

#[test]
fn declines_a_path_with_a_bare_lowercase_segment() {
    // `Path!` would have encoded a lowercase identifier as a `Symbol`, so meeting one as a plain
    // type is ambiguous — the spine is left raw rather than guessed at.
    let source = "type Route = PathCons<app, PathCons<GreeterComponent, Nil>>;\n";

    assert_eq!(expand(source), source);
}

#[test]
fn folds_a_qualified_path_segment_to_its_tail() {
    let source = "type Route = PathCons<finance::QuantityComponent, Nil>;\n";

    assert_eq!(expand(source), "type Route = Path!(@QuantityComponent);\n");
}

#[test]
fn a_path_terminator_is_not_read_as_an_empty_product() {
    // `Nil` terminates a path as well as a list, so the path pass must consume its own
    // terminator before the list pass runs. If it does not, this renders `Path!` over a
    // `Product![]` — or fails to resugar at all.
    let source = "type Route = PathCons<GreeterComponent, Nil>;\n";

    assert_eq!(expand(source), "type Route = Path!(@GreeterComponent);\n");
}

#[test]
fn a_symbol_terminator_is_not_read_as_an_empty_product() {
    // The same overlap one level in: a combined pass would rewrite the `Nil` closing this
    // symbol's character spine and leave the field name raw.
    let source = "type Tag = Symbol<1, Chars<'a', Nil>>;\n";

    assert_eq!(expand(source), "type Tag = Symbol!(\"a\");\n");
}

#[test]
fn strips_the_macro_prelude_qualifier() {
    let source = "\
impl ::cgp::macro_prelude::DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
";

    assert_eq!(
        expand(source),
        "\
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
"
    );
}

#[test]
fn keeps_ordinary_module_qualifiers() {
    // Unlike in a diagnostic, a module qualifier in source carries information a reader may want.
    let source = "type Marker = ::core::marker::PhantomData<App>;\n";

    assert_eq!(expand(source), source);
}

#[test]
fn resugars_a_qualified_spine_without_stripping() {
    // The passes match on a construct's final path segment, so a spine resugars under a qualifier
    // the prelude strip leaves in place.
    let source =
        "type Tag = ::cgp::prelude::Symbol<1, ::cgp::prelude::Chars<'a', ::cgp::prelude::Nil>>;\n";

    assert_eq!(expand(source), "type Tag = Symbol!(\"a\");\n");
}

#[test]
fn unparsable_source_is_returned_unchanged() {
    let source = "this is not rust";

    assert_eq!(expand(source), source);
}

#[test]
fn leaves_a_non_cgp_program_alone() {
    // Everything outside the CGP constructs is the compiler's output, printed back as it was.
    let source = "\
fn main() {
    println!(\"{}\", 1 + 1);
}
";

    assert_eq!(expand(source), source);
}

#[test]
fn strips_the_qualifier_from_a_qualified_path() {
    // A qualified path indexes into its own segments to say where the qualifier ends, so dropping
    // the prelude prefix has to move that index too. Getting it wrong panics the printer rather
    // than printing something wrong, which is how this surfaced.
    let source = "\
type Delegate = <__Provider__ as ::cgp::macro_prelude::DelegateComponent<
    AreaCalculatorComponent,
>>::Delegate;
";

    assert_eq!(
        expand(source),
        "type Delegate = <__Provider__ as DelegateComponent<AreaCalculatorComponent>>::Delegate;\n"
    );
}

#[test]
fn strips_the_qualifier_from_a_qualified_call() {
    // The same shape in expression position, which generated CGP code is full of.
    let source = "\
fn area(context: &Context) -> f64 {
    <__Provider__ as ::cgp::macro_prelude::DelegateComponent<C>>::Delegate::area(context)
}
";

    assert!(
        expand(source).contains("<__Provider__ as DelegateComponent<C>>::Delegate::area(context)"),
        "{}",
        expand(source)
    );
}

#[test]
fn a_string_literal_naming_a_sugar_macro_is_left_alone() {
    // The spacing pass edits only the bodies of the calls this crate emits; text inside a literal
    // is the program's data.
    let source = "const HINT: &str = \"Sum![a , b]\";\n";

    assert_eq!(expand(source), source);
}

#[test]
fn a_raw_string_naming_a_sugar_macro_is_left_alone() {
    // The quote inside it ends a plain string early, so a scan that missed the raw form would
    // take the `Sum![a , b]` after that quote for code and tighten it.
    let source = "const HINT: &str = r#\"say \"Sum![a , b]\" now\"#;\n";

    assert_eq!(expand(source), source);
}

#[test]
fn a_doc_comment_naming_a_sugar_macro_is_left_alone() {
    let source = "\
/// Spell it `Product![A , B]`.
pub struct Marker;
";

    assert_eq!(expand(source), source);
}

#[test]
fn a_sugar_call_after_a_literal_is_still_tightened() {
    // Skipping the literal must not skip the code after it.
    let source = "\
const HINT: &str = \"Sum![a , b]\";
type Stages = PipeHandlers<Cons<Multiply<Symbol<1, Chars<'f', Nil>>>, Nil>>;
";

    assert_eq!(
        expand(source),
        "\
const HINT: &str = \"Sum![a , b]\";
type Stages = PipeHandlers<Product![Multiply<Symbol!(\"f\")>]>;
"
    );
}
