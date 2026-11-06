//! The one block a group of failures sharing a root cause is emitted as.

use cargo_cgp_error_processing::{
    AmbiguousNames, Causes, Origin, PendingNote, Resolved, coalesce_underived_fields,
    coalesced_headline, fix_help_messages,
};
use rustc_errors::{DiagInner, DiagMessage, Style, Sublevel, Suggestions};

use crate::emitter::edit::{merged_span, postprocess_diag, subdiag};
use crate::emitter::flush::ChainNodeSet;

/// Build the one merged block for a coalesced group of several failures sharing a root cause: a
/// headline stating the cause once ([`coalesced_headline`]), the first failing site in the source
/// as its caret and every other site as a labelled secondary span ([`merged_span`]), and
/// one root-cause note built by folding *every* member's causes into a single dependency graph,
/// so a chain that runs through another collapses into it, independent chains to the shared
/// cause render side by side, and no member's chain is dropped. The headline takes the class
/// code of the member written first in the source.
pub fn merged_diag(
    resolveds: &[&Resolved],
    diags: &[&DiagInner],
    seen: &mut ChainNodeSet,
    ambiguous: &AmbiguousNames,
) -> DiagInner {
    // Every cause across the coalesced consumers. The dependency graph then merges what they
    // share: a consumer whose chain runs through another collapses into it, while independent
    // chains to one cause render side by side, so no chain is dropped and every consumer appears.
    // These members were grouped precisely *because* they share a cause, so each carries its own
    // copy; `Causes::union` folds those copies back into one cause holding every path: the
    // one-cause-per-distinct-leaf invariant the wording below assumes, kept by construction.
    let causes = Causes::union(resolveds.iter().map(|resolved| resolved.causes.clone()));
    let causes = coalesce_underived_fields(&causes);
    let headline = coalesced_headline(resolveds, &causes);
    // The fixes follow the note that explains them, as they do in a block of its own.
    let helps = fix_help_messages(&causes);
    let mut children: Vec<_> = PendingNote {
        causes,
        header_leaf: None,
    }
    .render(seen)
    .into_iter()
    .map(|note| subdiag(Sublevel::Note, note))
    .collect();
    children.extend(helps.into_iter().map(|help| subdiag(Sublevel::Help, help)));

    let mut diag = diags[0].clone();
    diag.messages = vec![(DiagMessage::Str(headline.into()), Style::NoStyle)];
    diag.span = merged_span(diags);
    diag.children = children;
    diag.suggestions = Suggestions::Enabled(Vec::new());
    // The header, notes, and helps are freshly built, so post-process them (strip the module
    // qualifiers that tell no two items apart, resugar `Symbol!`) as the streaming path does for a
    // rewritten diagnostic.
    postprocess_diag(&mut diag, Origin::Rewritten, ambiguous);
    diag
}
