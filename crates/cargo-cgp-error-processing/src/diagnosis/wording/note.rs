//! The `root cause:` note bodies, one per group of causes sharing a dependency root.

use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::resolved::Cause;
use crate::diagnosis::wording::lead::{root_cause_code, root_cause_lead};
use crate::tree::{merge_dependency_forest, render_dependency_tree};

/// The one `note` per root-cause group the emitter emits, grouping causes that share a dependency
/// root so their trees merge into one. Causes whose chains descend from the **same** root obligation
/// — the usual shape of a multi-root-cause failure, where every chain restates the whole shared
/// prefix — collapse into a single [`merged_cause_note`] whose common ancestors appear once and whose
/// branches end at the distinct leaves; a lone cause keeps its own [`cause_note`]. Grouping is by the
/// tree's root label, so causes that genuinely share no ancestor stay separate notes.
pub fn cause_notes(causes: &[Cause], header_bound: Option<&str>) -> Vec<String> {
    let mut groups: Vec<Vec<&Cause>> = Vec::new();
    for cause in causes {
        match groups
            .iter_mut()
            .find(|group| group[0].tree.label == cause.tree.label)
        {
            Some(group) => group.push(cause),
            None => groups.push(vec![cause]),
        }
    }
    groups
        .into_iter()
        .map(|group| match group.as_slice() {
            [only] => cause_note(only, header_bound),
            many => merged_cause_note(many),
        })
        .collect()
}

/// Indent every line of a rendered dependency chain by two spaces, so it nests under the note's
/// `this is required through the dependency chain:` heading.
fn indent_chain(chain: &str) -> String {
    chain
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The note body for one root cause: the `root cause:` lead naming the leaf, then the rendered
/// dependency chain nested beneath its heading. When the diagnostic's kept main message already
/// states the leaf bound (`header_bound`), the lead would only repeat it, so the note carries
/// the chain alone — as it also does for a field-type mismatch, whose `[CGP-E003]` header
/// states the leaf in full.
pub fn cause_note(cause: &Cause, header_bound: Option<&str>) -> String {
    let chain = render_dependency_tree(&cause.tree);
    if let Leaf::FieldTypeMismatch { .. } = &cause.leaf {
        return format!(
            "this is required through the dependency chain:\n{}",
            indent_chain(&chain)
        );
    }
    if let (Some(bound), Leaf::Bound { summary }) = (header_bound, &cause.leaf)
        && summary == bound
    {
        return format!(
            "this is required through the dependency chain:\n{}",
            indent_chain(&chain)
        );
    }
    format!(
        "root cause: [{code}] {lead}\nthis is required through the dependency chain:\n{chain}",
        code = root_cause_code(&cause.leaf),
        lead = root_cause_lead(&cause.leaf),
        chain = indent_chain(&chain),
    )
}

/// The note body for several root causes that share a dependency root: a `root causes:` heading
/// listing each leaf, then the *merged* dependency tree (built by [`merge_dependency_forest`]) whose
/// shared ancestors appear once and whose branches bottom out at the listed leaves. Because the tree
/// itself names every leaf at its branch end, the leading list is a summary a reader sees first; it
/// carries the same `[CGP-Exxx]` code and wording as each leaf's tree terminal.
fn merged_cause_note(causes: &[&Cause]) -> String {
    let trees: Vec<_> = causes.iter().map(|cause| cause.tree.clone()).collect();
    let merged = merge_dependency_forest(&trees);
    // Grouping in [`cause_notes`] guarantees a shared root, so the merge yields exactly one tree.
    let chain = match merged.as_slice() {
        [tree] => render_dependency_tree(tree),
        // Defensive: unshared roots would leave a forest — render each and stack them.
        many => many
            .iter()
            .map(render_dependency_tree)
            .collect::<Vec<_>>()
            .join("\n"),
    };
    let list: String = causes
        .iter()
        .map(|cause| {
            format!(
                "  - [{code}] {lead}",
                code = root_cause_code(&cause.leaf),
                lead = root_cause_lead(&cause.leaf),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "root causes:\n{list}\nthis is required through the dependency chain:\n{}",
        indent_chain(&chain),
    )
}
