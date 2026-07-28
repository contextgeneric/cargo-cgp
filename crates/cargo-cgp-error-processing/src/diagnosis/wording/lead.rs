//! The one-sentence root-cause statement of a leaf, and the codes it carries.

use crate::code::{
    DEP_ASSOC_TYPE_MISMATCH, DEP_FIELD_TYPE_MISMATCH, DEP_MISSING_DELEGATE_ENTRY,
    DEP_MISSING_DISPATCH_ENTRY, DEP_MISSING_FIELD, DEP_NOT_A_PROVIDER, DEP_UNIMPLEMENTED_ACCESSOR,
    ROOT_CAUSE_ORDINARY_BOUND,
};
use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::wording::header::{quoted_list, required_type};

/// What an associated type is called in a message: `abstract type` when it belongs to a CGP
/// abstract-type component (a context *chooses* its concrete type by wiring), and `associated type`
/// for any other trait, where no such choice exists. Shared by the root-cause lead and the
/// `[CGP-E017]` header so the two never drift.
pub fn assoc_type_noun(component: Option<&str>) -> &'static str {
    match component {
        Some(_) => "abstract type",
        None => "associated type",
    }
}

/// The statement that a context carries no delegate entry for a key — a component marker for a
/// plain missing wiring, or a redirect path for an unterminated namespace lookup. Shared by the
/// root-cause lead here and the terminal dependency-tree node the driver renders, so the two never
/// drift: a missing delegate entry reads the same whether it heads the note or ends the chain.
pub fn missing_delegate_entry(context: &str, key: &str) -> String {
    format!("context `{context}` does not contain any delegate entry for `{key}`")
}

/// The one root-cause statement for a leaf — what the note names before the dependency chain,
/// and (repeated) the tree's terminal leaf the driver appends so the chain visibly bottoms out at
/// the root cause. A genuinely missing field is said plainly (without a `context` qualifier, since
/// `HasField` can land on any struct); a present-but-underived field is worded as the unimplemented
/// accessor, with the fix (the derive) carried by a separate `help`; a missing wiring (plain or a
/// redirect) names the missing delegate entry; a projection mismatch names both the type required
/// and the one supplied, with an abstract type's wiring fix likewise carried by a separate `help`;
/// any other leaf restates its unmet bound.
pub fn root_cause_lead(leaf: &Leaf) -> String {
    match leaf {
        Leaf::Field {
            name,
            owner,
            issue: FieldIssue::Missing,
        } => format!("missing field `{name}` on `{owner}`"),
        Leaf::MissingWiring { component, owner } => missing_delegate_entry(owner, component),
        Leaf::MissingDispatchEntry { key, table } => {
            format!("provider `{table}` does not contain any delegate entry for `{key}`")
        }
        Leaf::NotAProvider {
            provider,
            provider_trait,
        } => format!("the provider trait `{provider_trait}` is not implemented for `{provider}`"),
        Leaf::MissingRedirectWiring { path, context } => missing_delegate_entry(context, path),
        Leaf::Field { name, owner, .. } => {
            format!(
                "accessor trait `HasField` with field `{name}` is not implemented for `{owner}`"
            )
        }
        Leaf::UnderivedFields { names, owner } => {
            format!(
                "accessor trait `HasField` is not implemented for the fields {} of `{owner}`",
                quoted_list(names)
            )
        }
        Leaf::FieldTypeMismatch {
            name,
            owner,
            expected,
            expected_normalized,
            actual,
        } => {
            let required = required_type(expected, expected_normalized.as_deref());
            format!("field `{name}` on `{owner}` has type `{actual}`, but {required} is required")
        }
        Leaf::AssocTypeMismatch {
            assoc,
            trait_name,
            owner,
            expected,
            expected_normalized,
            actual,
            component,
        } => {
            let noun = assoc_type_noun(component.as_deref());
            let required = required_type(expected, expected_normalized.as_deref());
            format!(
                "{noun} `{assoc}` of `{trait_name}` on `{owner}` is `{actual}`, but {required} is required"
            )
        }
        Leaf::Bound { summary } => format!("the trait bound `{summary}` is not satisfied"),
    }
}

/// The `CGP-E1xx` code for the terminal root-cause leaf as a dependency-tree entry, or `None` when
/// the leaf is a pass-through of a non-CGP bound (`the trait bound … is not satisfied`), which
/// carries no code. Keyed by leaf kind — a missing field, a present-but-underived field, a missing
/// delegate entry (plain or redirect), a missing dispatch entry, a non-provider, and the two
/// mismatch leaves (a `HasField` value type and any other associated type) each get their own code.
pub fn dependency_leaf_code(leaf: &Leaf) -> Option<&'static str> {
    match leaf {
        Leaf::Field {
            issue: FieldIssue::Missing,
            ..
        } => Some(DEP_MISSING_FIELD),
        Leaf::Field { .. } | Leaf::UnderivedFields { .. } => Some(DEP_UNIMPLEMENTED_ACCESSOR),
        Leaf::MissingWiring { .. } | Leaf::MissingRedirectWiring { .. } => {
            Some(DEP_MISSING_DELEGATE_ENTRY)
        }
        Leaf::MissingDispatchEntry { .. } => Some(DEP_MISSING_DISPATCH_ENTRY),
        Leaf::NotAProvider { .. } => Some(DEP_NOT_A_PROVIDER),
        Leaf::FieldTypeMismatch { .. } => Some(DEP_FIELD_TYPE_MISMATCH),
        Leaf::AssocTypeMismatch { .. } => Some(DEP_ASSOC_TYPE_MISMATCH),
        Leaf::Bound { .. } => None,
    }
}

/// The terminal root-cause leaf as it appears *in* the dependency tree — [`root_cause_lead`] with
/// its `CGP-E1xx` code prefixed, or bare when the leaf is a pass-through non-CGP bound. This is the
/// coded counterpart the driver appends as the tree's last node; the `root cause:` note lead
/// ([`cause_note`](super::cause_note)) repeats the same text with the [`root_cause_code`] tag.
pub fn dependency_tree_leaf(leaf: &Leaf) -> String {
    match dependency_leaf_code(leaf) {
        Some(code) => format!("[{code}] {}", root_cause_lead(leaf)),
        None => root_cause_lead(leaf),
    }
}

/// The `CGP-Exxx` code the `root cause:` note lead carries. It reuses the terminal leaf's
/// [`dependency_leaf_code`] where the leaf has one, so the lead and the tree's terminal show the
/// same code; when the leaf is an uncoded pass-through bound it takes the `CGP-E2xx` root-cause code
/// [`ROOT_CAUSE_ORDINARY_BOUND`] instead, since the lead still names a classified root cause.
pub fn root_cause_code(leaf: &Leaf) -> &'static str {
    dependency_leaf_code(leaf).unwrap_or(ROOT_CAUSE_ORDINARY_BOUND)
}
