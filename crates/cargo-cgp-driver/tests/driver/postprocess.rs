//! Tests for post-processing a diagnostic's span labels in place.

use cargo_cgp_driver::emitter::postprocess_multispan;
use cargo_cgp_error_processing::{AmbiguousNames, Origin};
use rustc_errors::{DiagMessage, MultiSpan};

use crate::driver::source::with_source;

/// The source the spans point into: the label sits on `width`, bytes 29 to 34, and the unlabelled
/// context line rustc shows above it is the `fn` at bytes 0 to 2.
const SOURCE: &str = "fn area(shape: &Shape) {\n    width\n}\n";

#[test]
fn a_rewritten_label_keeps_the_span_context() {
    with_source(SOURCE, |_, span| {
        let mut multispan = MultiSpan::from_span(span((29, 34)));
        multispan.push_span_label(span((29, 34)), "expected `cgp::prelude::Chars`");
        multispan.push_span_context(span((0, 2)));

        postprocess_multispan(
            &mut multispan,
            false,
            Origin::Rewritten,
            &AmbiguousNames::default(),
        );

        let labels: Vec<_> = multispan
            .span_labels_raw()
            .iter()
            .map(|(_, message)| message.clone())
            .collect();
        assert_eq!(
            labels,
            vec![DiagMessage::from("expected `Chars`".to_owned())]
        );
        assert_eq!(multispan.primary_spans(), [span((29, 34))]);
        assert_eq!(multispan.span_context(), [span((0, 2))]);
    });
}
