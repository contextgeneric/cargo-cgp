//! Tests for the flush's coalescing and for pairing the two `E0428`s of a shared dispatch method.
//!
//! Each test builds its buffer by hand over one in-memory source file, so the carets fall on known
//! positions. The wording of a block is the error-processing crate's and is pinned there; these
//! check what the flush decides: which entries become one block, where it sits, and what it carries.

use cargo_cgp_driver::emitter::{
    BufEntry, ChainNodeSet, ResolvedEntry, Verdict, coalesced_groups, insert_note, merged_diag,
    render_blocks, shared_dispatch_method,
};
use cargo_cgp_error_processing::{
    AmbiguousNames, COALESCED_SITE_LABEL, Cause, Causes, ChainNode, DepNode, FieldFix, FieldIssue,
    Leaf, PendingNote, Resolved, ResolvedClass, dispatch_computer_name, dispatch_helper_method,
};
use rustc_errors::codes::{E0277, E0428};
use rustc_errors::{
    DiagArgValue, DiagInner, DiagMessage, Level, MultiSpan, Style, Subdiag, Sublevel, Suggestions,
};
use rustc_span::Span;

use crate::driver::source;

/// The source the carets point into: `first` is bytes 8 to 13, `second` 25 to 31, `third` 43 to 48.
const SOURCE: &str = "let a = first();\nlet b = second();\nlet c = third();\n";
const FIRST: (u32, u32) = (8, 13);
const SECOND: (u32, u32) = (25, 31);
const THIRD: (u32, u32) = (43, 48);

/// A failure of `consumer` on `App` bottoming out on a missing field per name in `fields`.
fn failure(consumer: &str, fields: &[&str], class: ResolvedClass) -> Resolved {
    let causes: Causes = fields
        .iter()
        .map(|name| {
            let leaf = Leaf::Field {
                name: (*name).to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
                fix: FieldFix {
                    confirmed: true,
                    ..FieldFix::default()
                },
            };
            Cause {
                paths: vec![vec![
                    ChainNode::Hop(DepNode::Consumer {
                        trait_ref: consumer.to_owned(),
                        context: "App".to_owned(),
                        is_context: true,
                    }),
                    ChainNode::Leaf(leaf.clone()),
                ]],
                leaf,
            }
        })
        .collect();
    Resolved {
        context: "App".to_owned(),
        consumers: vec![consumer.to_owned()],
        class,
        subject_is_context: true,
        causes,
    }
}

/// An error with `message` at the primary `caret`.
fn error(message: &str, caret: Span) -> DiagInner {
    let mut diag = DiagInner::new(Level::Error, message.to_owned());
    diag.code = Some(E0277);
    diag.span = MultiSpan::from_span(caret);
    diag
}

/// A plain buffered diagnostic.
fn plain(message: &str, caret: Span) -> BufEntry {
    BufEntry::Plain(Box::new(error(message, caret)))
}

/// A sub-diagnostic at `level` saying `text`.
fn child(level: Sublevel, text: &str) -> Subdiag {
    Subdiag {
        level,
        messages: vec![(DiagMessage::Str(text.to_owned().into()), Style::NoStyle)],
        span: MultiSpan::new(),
    }
}

/// A buffered resolution of `consumer`, headed by its name, with one `help` already attached.
fn resolution(consumer: &str, fields: &[&str], caret: Span, coalescible: bool) -> BufEntry {
    let resolved = failure(consumer, fields, ResolvedClass::Consumer);
    let note = PendingNote {
        causes: resolved.causes.iter().cloned().collect(),
        header_leaf: None,
    };
    let mut diag = error(consumer, caret);
    diag.children.push(child(Sublevel::Help, "a fix"));
    BufEntry::Resolved(Box::new(ResolvedEntry {
        coalescible,
        resolved,
        note,
        original: error("as rustc built it", caret),
        diag,
    }))
}

/// The text of a message list, joined.
fn text(messages: &[(DiagMessage, Style)]) -> String {
    messages
        .iter()
        .map(|(message, _)| match message {
            DiagMessage::Str(text) => text.to_string(),
            other => panic!("expected a plain message, got {other:?}"),
        })
        .collect()
}

/// The buffer rendered by the flush, with every group as `coalesced_groups` finds it.
fn blocks(buffer: &[BufEntry]) -> Vec<DiagInner> {
    render_blocks(
        buffer,
        &coalesced_groups(buffer),
        &AmbiguousNames::default(),
    )
}

// Property: for all buffers, the groups partition exactly the coalescible resolutions by shared
// cause, in arrival order, and `group_of` agrees with them. A plain entry and a resolution that
// never coalesces belong to no group, even when it shares a cause.
#[test]
fn only_coalescible_resolutions_are_grouped() {
    source::with_source(SOURCE, |_, span| {
        let caret = span(FIRST);
        let buffer = [
            plain("an unrelated error", caret),
            resolution("CanFoo", &["name"], caret, true),
            resolution("CanKept", &["name"], caret, false),
            resolution("CanBar", &["name"], caret, true),
            resolution("CanBaz", &["age"], caret, true),
        ];

        let groups = coalesced_groups(&buffer);

        assert_eq!(groups.groups, [vec![1, 3], vec![4]]);
        assert_eq!(groups.group_of.get(&1), Some(&0));
        assert_eq!(groups.group_of.get(&3), Some(&0));
        assert_eq!(groups.group_of.get(&4), Some(&1));
        assert_eq!(groups.group_of.len(), 3);
    });
}

// Property: for all buffers, a group of several is one block at its first member's arrival
// position, and every other entry keeps its arrival order.
#[test]
fn a_group_is_one_block_at_its_first_members_position() {
    source::with_source(SOURCE, |_, span| {
        let buffer = [
            plain("before", span(FIRST)),
            resolution("CanFoo", &["name"], span(SECOND), true),
            plain("between", span(FIRST)),
            resolution("CanBar", &["name"], span(THIRD), true),
            plain("after", span(FIRST)),
        ];

        let emitted = blocks(&buffer);

        assert_eq!(emitted.len(), 4, "the two members become one block");
        assert_eq!(text(&emitted[0].messages), "before");
        assert_eq!(emitted[1].span.primary_span(), Some(span(SECOND)));
        assert_eq!(text(&emitted[2].messages), "between");
        assert_eq!(text(&emitted[3].messages), "after");
    });
}

// Property: a group of one, and a resolution that never coalesces, each emit their own header with
// their note rendered ahead of their `help`s.
#[test]
fn a_block_of_its_own_keeps_its_header_with_the_note_before_the_help() {
    source::with_source(SOURCE, |_, span| {
        let buffer = [
            resolution("CanFoo", &["name"], span(FIRST), true),
            resolution("CanKept", &["name"], span(SECOND), false),
        ];

        let emitted = blocks(&buffer);

        assert_eq!(
            emitted.len(),
            2,
            "a resolution that never coalesces stays apart"
        );
        for (block, header) in emitted.iter().zip(["CanFoo", "CanKept"]) {
            assert_eq!(text(&block.messages), header);
            let levels: Vec<Sublevel> = block.children.iter().map(|child| child.level).collect();
            assert_eq!(levels, [Sublevel::Note, Sublevel::Help], "for {header}");
        }
    });
}

// Property: a merged block's caret is its member written first in the source, whatever the arrival
// order, and every other member's site is labelled `COALESCED_SITE_LABEL`.
#[test]
fn a_merged_block_points_at_the_site_written_first() {
    source::with_source(SOURCE, |_, span| {
        let buffer = [
            resolution("CanBar", &["name"], span(THIRD), true),
            resolution("CanFoo", &["name"], span(FIRST), true),
        ];

        let emitted = blocks(&buffer);

        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].span.primary_span(), Some(span(FIRST)));
        let also_required: Vec<Span> = emitted[0]
            .span
            .span_labels()
            .into_iter()
            .filter(|label| {
                matches!(&label.label, Some(DiagMessage::Str(text)) if text == COALESCED_SITE_LABEL)
            })
            .map(|label| label.span)
            .collect();
        assert_eq!(also_required, [span(THIRD)]);
    });
}

// Property (the part on suggestions assumed from `driver/program.md`'s list of what a merged block holds): a
// merged block states the shared cause once, in a headline and one note naming every member's
// consumer, followed by the fix; rustc's own children and suggestions go.
#[test]
fn a_merged_block_states_the_cause_once_and_names_every_consumer() {
    source::with_source(SOURCE, |_, span| {
        let first = failure("CanFoo", &["name"], ResolvedClass::Consumer);
        let second = failure("CanBar", &["name"], ResolvedClass::Consumer);
        let mut with_children = error("CanFoo", span(FIRST));
        with_children
            .children
            .push(child(Sublevel::Note, "rustc's own note"));
        with_children.suggestions = Suggestions::Disabled;
        let other = error("CanBar", span(SECOND));

        let merged = merged_diag(
            &[&first, &second],
            &[&with_children, &other],
            &mut ChainNodeSet::new(),
            &AmbiguousNames::default(),
        );

        let headline = text(&merged.messages);
        assert!(
            headline.contains("`name`"),
            "the headline states the cause: {headline}"
        );
        let children: Vec<(Sublevel, String)> = merged
            .children
            .iter()
            .map(|child| (child.level, text(&child.messages)))
            .collect();
        let notes: Vec<&String> = children
            .iter()
            .filter(|(level, _)| *level == Sublevel::Note)
            .map(|(_, text)| text)
            .collect();
        assert_eq!(notes.len(), 1, "one note folds every chain: {children:?}");
        assert!(
            notes[0].contains("CanFoo") && notes[0].contains("CanBar"),
            "{}",
            notes[0]
        );
        assert!(!notes[0].contains("rustc's own note"));
        assert!(matches!(&merged.suggestions, Suggestions::Enabled(found) if found.is_empty()));
        assert_eq!(
            children.last().map(|(level, _)| *level),
            Some(Sublevel::Help)
        );
    });
}

/// A buffered resolution that never coalesces, failing `consumer` through `shared` on a missing field.
fn through_shared(consumer: &str, shared: &str, caret: Span) -> BufEntry {
    let mut resolved = failure(consumer, &["name"], ResolvedClass::Consumer);
    let causes: Causes = resolved
        .causes
        .iter()
        .cloned()
        .map(|mut cause| {
            cause.paths[0].insert(
                1,
                ChainNode::Hop(DepNode::Consumer {
                    trait_ref: shared.to_owned(),
                    context: "App".to_owned(),
                    is_context: true,
                }),
            );
            cause
        })
        .collect();
    resolved.causes = causes;
    let note = PendingNote {
        causes: resolved.causes.iter().cloned().collect(),
        header_leaf: None,
    };
    BufEntry::Resolved(Box::new(ResolvedEntry {
        coalescible: false,
        resolved,
        note,
        original: error("as rustc built it", caret),
        diag: error(consumer, caret),
    }))
}

// Property: for all buffers, the subtrees drawn so far are shared across blocks in emission order, so
// a later block `(*)`-elides a subtree an earlier one showed.
#[test]
fn a_later_block_elides_what_an_earlier_one_drew() {
    source::with_source(SOURCE, |_, span| {
        let buffer = [
            through_shared("CanFoo", "CanShared", span(FIRST)),
            through_shared("CanBar", "CanShared", span(SECOND)),
        ];

        let emitted = blocks(&buffer);

        let notes: Vec<String> = emitted
            .iter()
            .map(|block| text(&block.children[0].messages))
            .collect();
        assert!(!notes[0].contains("(*)"), "{}", notes[0]);
        assert!(notes[1].contains("CanShared"), "{}", notes[1]);
        assert!(notes[1].contains("(*)"), "{}", notes[1]);
    });
}

// Property: for all diagnostics, `insert_note` puts the note ahead of the first `help`, and last
// when there is none, and leaves every other child in order.
#[test]
fn a_note_goes_ahead_of_the_first_help() {
    source::with_source(SOURCE, |_, span| {
        let resolved = failure("CanFoo", &["name"], ResolvedClass::Consumer);
        let note = PendingNote {
            causes: resolved.causes.iter().cloned().collect(),
            header_leaf: None,
        };

        let mut with_help = error("header", span(FIRST));
        with_help.children = vec![
            child(Sublevel::Note, "kept"),
            child(Sublevel::Help, "a fix"),
            child(Sublevel::Help, "another"),
        ];
        insert_note(
            &mut with_help,
            &note,
            &mut ChainNodeSet::new(),
            &AmbiguousNames::default(),
        );
        let order: Vec<Sublevel> = with_help.children.iter().map(|child| child.level).collect();
        assert_eq!(
            order,
            [
                Sublevel::Note,
                Sublevel::Note,
                Sublevel::Help,
                Sublevel::Help
            ]
        );
        assert_eq!(text(&with_help.children[0].messages), "kept");
        assert_eq!(text(&with_help.children[2].messages), "a fix");

        let mut no_help = error("header", span(FIRST));
        insert_note(
            &mut no_help,
            &note,
            &mut ChainNodeSet::new(),
            &AmbiguousNames::default(),
        );
        assert_eq!(no_help.children.len(), 1);
        assert_eq!(no_help.children[0].level, Sublevel::Note);
    });
}

/// An `E0428` reporting `name`, the way a derived diagnostic carries it.
fn defined_twice(name: &str, caret: Span) -> DiagInner {
    let mut diag = error("the name is defined multiple times", caret);
    diag.code = Some(E0428);
    diag.args
        .insert("name".into(), DiagArgValue::Str(name.to_owned().into()));
    diag
}

// Concern: the shared dispatch method's stage could pair an `E0428` with any buffered one that
// names the method's helper.
// Handled: the helper's caret must come from a `#[cgp_auto_dispatch]` expansion, so a helper-shaped
// name the program wrote itself, at a caret with no expansion, pairs with nothing.
// Property: for all diagnostics that are not a computer `E0428` with a buffered helper from that
// expansion, `shared_dispatch_method` leaves the diagnostic to the next stage.
// The positive pairing needs an expansion span, so
// `tests/ui/wiring/conflicting-wiring/auto_dispatch_shared_method_name.rs` covers it end to end.
#[test]
fn a_diagnostic_without_its_helper_goes_to_the_next_stage() {
    source::with_source(SOURCE, |_, span| {
        // The names really are a pair, so only the caret keeps the last case from pairing.
        assert_eq!(
            dispatch_helper_method("__compute_area__").map(dispatch_computer_name),
            Some("ComputeArea".to_owned())
        );
        let computer = defined_twice("ComputeArea", span(SECOND));
        let written_helper = defined_twice("__compute_area__", span(FIRST));
        for (diag, buffer) in [
            (error("another error", span(SECOND)), vec![]),
            (defined_twice("ComputeArea", span(SECOND)), vec![]),
            (computer, vec![BufEntry::Plain(Box::new(written_helper))]),
        ] {
            assert_eq!(shared_dispatch_method(&diag, &buffer), Verdict::Continue);
        }
    });
}
