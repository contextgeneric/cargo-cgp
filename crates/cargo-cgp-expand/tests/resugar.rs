//! Tests for the resugaring passes, over hand-written expanded source.
//!
//! Each case is the source a CGP macro's expansion prints, run through the whole pipeline the way
//! the driver runs it, so what is asserted is what a reader of `cargo cgp expand` sees.

use cargo_cgp_expand::{ExpandOptions, resugar_expanded_source};

/// Resugar a snippet with the default options (the prelude qualifier stripped).
fn expand(source: &str) -> String {
    resugar_expanded_source(source, &ExpandOptions::default())
}

/// Resugar a snippet with nothing stripped, so only the resugaring itself shows.
fn expand_verbatim(source: &str) -> String {
    let options = ExpandOptions {
        strip_cgp_prefixes: false,
    };
    resugar_expanded_source(source, &options)
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
fn a_field_list_stays_a_product_rather_than_folding_to_a_record() {
    // A diagnostic folds an all-field list on to `Struct! { width: f64, … }`, but that form is not
    // a real CGP macro. This pass writes source, so it stops at the list macro that is.
    let source = "\
type Fields = Cons<
    Field<Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>, f64>,
    Cons<Field<Symbol<4, Chars<'n', Chars<'a', Chars<'m', Chars<'e', Nil>>>>>, String>, Nil>,
>;
";

    // The printer breaks a body too long for one line, which the tightening leaves alone.
    assert_eq!(
        expand(source),
        "\
type Fields = Product![
    Field<Symbol!(\"width\"), f64>, Field<Symbol!(\"name\"), String>
];
"
    );
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
    let source = "type Key = PathCons<ItemEncoderComponent, PathCons<u64, Nil>>;\n";

    assert_eq!(
        expand(source),
        "type Key = Path!(@ItemEncoderComponent.u64);\n"
    );
}

#[test]
fn declines_an_open_tailed_path() {
    // A diagnostic renders an open-ended path with a trailing `.*` wildcard, which is not `Path!`
    // syntax. Source output emits only real syntax, so the chain is left as it stands.
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
    // The passes match on a construct's final path segment, so a fully-qualified spine resugars
    // whether or not the qualifier was stripped first.
    let source = "type Tag = ::cgp::macro_prelude::Symbol<1, ::cgp::macro_prelude::Chars<'a', ::cgp::macro_prelude::Nil>>;\n";

    assert_eq!(expand_verbatim(source), "type Tag = Symbol!(\"a\");\n");
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
