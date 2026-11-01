//! Tests for the post-processing chain over a message and over its styled fragments.
//!
//! Each transform reads plain text and gives `Some` when it changed it, so it is driven directly
//! over the case under test, with no diagnostic wrapper and no compiler.

use cargo_cgp_error_processing::{
    AmbiguousNames, Origin, postprocess_fragments, postprocess_message,
};

// Property: `postprocess_message` resugars `Symbol!` before the lists, so a raw `Field` tag still
// names its field.
#[test]
fn full_chain_resugars_a_raw_field_spine_to_a_struct() {
    // End to end: the raw `Symbol` spine is resugared first, then the field list folds to `Struct!`.
    assert_eq!(
        postprocess_message(
            "Cons<Field<Symbol<1, Chars<'x', Nil>>, u64>, Nil>",
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("Struct! { x: u64 }"),
    );
}

// Property: `postprocess_message` resugars `Symbol!` before `Path!`, which reads the resugared
// segments.
#[test]
fn postprocess_message_resugars_an_expanded_path() {
    // End to end through the chain: an expanded `Symbol` spine inside a `PathCons` is
    // resugared to a symbol and then folded into a `Path!`.
    let text = "PathCons<Symbol<3, Chars<'a', Chars<'p', Chars<'p', Nil>>>>, PathCons<GreeterComponent, Nil>>";
    assert_eq!(
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("Path!(@app.GreeterComponent)"),
    );
}

// Property: for all open-tailed paths, `Origin` chooses the form: bare `@...` for a rewritten
// message, and the spine with its symbols resugared for an untransformed one.
#[test]
fn postprocess_message_resugars_an_open_tailed_path() {
    // The conflicting-wiring E0119 shape end to end: two prefixed, expanded `Symbol` spines
    // inside a `PathCons` whose open `_` tail is the impl's generic "rest of path" parameter.
    // A rewritten message folds it into the key its entry wrote; an untransformed one has no
    // `Path!` spelling for it, so it keeps the spine with its symbols resugared.
    let text = "PathCons<cgp::prelude::Symbol<3, cgp::prelude::Chars<'f', cgp::prelude::Chars<'o', cgp::prelude::Chars<'o', Nil>>>>, PathCons<Symbol<3, Chars<'b', Chars<'a', Chars<'r', Nil>>>>, _>>";
    assert_eq!(
        postprocess_message(text, false, Origin::Rewritten, &AmbiguousNames::default()).as_deref(),
        Some("@foo.bar"),
    );
    assert_eq!(
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("PathCons<Symbol!(\"foo\"), PathCons<Symbol!(\"bar\"), _>>"),
    );
}

// Property: a clause with no landmark gets its strip and resugaring but keeps rustc's wording.
#[test]
fn postprocess_message_chains_the_transforms() {
    // A prefixed, expanded `HasField<Symbol<…>>` clause with no similar-impl landmark: stripped
    // and resugared in one pass, and left in rustc's wording, since nothing shows the fix.
    let text = "the trait `HasField<cgp::prelude::Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>>` is not implemented for `Rectangle`";
    assert_eq!(
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("the trait `HasField<Symbol!(\"width\")>` is not implemented for `Rectangle`"),
    );
}

// Property: for all messages holding no CGP encoding, the chain gives `None`.
#[test]
fn postprocess_message_leaves_plain_rust_alone() {
    assert_eq!(
        postprocess_message(
            "the trait bound `Foo: Bar` is not satisfied",
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        ),
        None,
    );
}

// --- Fragmented messages -----------------------------------------------------------------
//
// rustc splits a message into styled fragments to highlight part of it, and splits at every
// difference when highlighting the difference between two types, shredding a CGP spine so no
// single fragment matches. `postprocess_fragments` reads them as the one line they render as.

// Property: for all fragments whose concatenation holds a CGP construct no fragment holds,
// `postprocess_fragments` gives the rewritten whole.
#[test]
fn a_spine_split_across_fragments_is_resugared_as_a_whole() {
    // The shape rustc's "similar impl" hint produces, split at each differing character.
    let fragments = [
        "the trait `FromVariant<Symbol<3, Chars<'",
        "B",
        "', Chars<'",
        "a",
        "', Chars<'",
        "r",
        "', Nil>>>>>` is not implemented for `Big`",
    ];

    assert_eq!(
        postprocess_fragments(
            &fragments,
            false,
            Origin::Rewritten,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("the trait `FromVariant<Symbol!(\"Bar\")>` is not implemented for `Big`")
    );
}

// Property: for all fragments with no construct across a boundary, `postprocess_fragments` gives
// `None`.
#[test]
fn fragments_needing_no_joining_keep_their_styling() {
    // Each fragment is post-processed on its own by the caller before this runs, so a message
    // with nothing spanning a boundary must decline: flattening would drop rustc's
    // highlighting for no gain.
    let fragments = ["the trait `", "Foo", "` is not implemented for `Bar`"];

    assert_eq!(
        postprocess_fragments(
            &fragments,
            false,
            Origin::Rewritten,
            &AmbiguousNames::default()
        ),
        None
    );
}

// Property: for a single fragment, nothing spans a boundary, so `postprocess_fragments` gives
// `None`.
#[test]
fn a_single_fragment_is_left_to_the_per_message_pass() {
    let fragments = ["Symbol<2, Chars<'x', Chars<'y', Nil>>>"];

    assert_eq!(
        postprocess_fragments(
            &fragments,
            false,
            Origin::Rewritten,
            &AmbiguousNames::default()
        ),
        None
    );
}

// Property: for all untransformed messages where no later stage recovers an encoding, the strips
// alone do not count as a change.
#[test]
fn an_untransformed_message_keeps_the_paths_rustc_printed() {
    // Two types sharing a name are told apart only by their paths, so a message rustc built and the
    // tool left alone keeps them; so does a glob-imported trait's path in a note.
    let same_name = "expected `a::Foo`, found `b::Foo`";
    assert_eq!(
        postprocess_message(
            same_name,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        ),
        None
    );
    let glob = "candidate #1 is defined in an impl of the trait `cgp::prelude::CanRaiseError`";
    assert_eq!(
        postprocess_message(
            glob,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        ),
        None
    );
}

// Property: for all rewritten messages, the chain strips every qualifier except on a name in
// `ambiguous`.
#[test]
fn a_rewritten_message_keeps_only_qualifiers_that_disambiguate() {
    // A message the tool built drops every qualifier except on a name that names several items,
    // where the qualifier tells them apart.
    let ambiguous = AmbiguousNames::new([("App".to_owned(), vec!["app::App".to_owned()])], []);
    assert_eq!(
        postprocess_message(
            "the consumer trait `cgp::prelude::CanRaiseError<std::io::Error>` for `app::App`",
            false,
            Origin::Rewritten,
            &ambiguous,
        )
        .as_deref(),
        Some("the consumer trait `CanRaiseError<std::io::Error>` for `App`"),
    );
}

// Property: for all untransformed messages where a later stage recovers an encoding, the strips
// apply to the whole message.
#[test]
fn an_untransformed_message_with_a_cgp_encoding_is_cleaned_whole() {
    // Once a CGP encoding is recovered, the strip applies to the rest of that message too.
    let text = "`app::App: cgp::prelude::HasField<cgp::prelude::Symbol<1, Chars<'x', Nil>>>` is not satisfied";
    assert_eq!(
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("`App: HasField<Symbol!(\"x\")>` is not satisfied"),
    );
}
