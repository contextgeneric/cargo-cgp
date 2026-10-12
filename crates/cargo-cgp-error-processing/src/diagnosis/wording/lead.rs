//! The one-sentence root-cause statement of a leaf, and the codes it carries.

use crate::code::{
    DEP_ASSOC_TYPE_MISMATCH, DEP_FIELD_TYPE_MISMATCH, DEP_MISSING_DELEGATE_ENTRY,
    DEP_MISSING_DISPATCH_ENTRY, DEP_MISSING_FIELD, DEP_NOT_A_PROVIDER, DEP_ORDINARY_BOUND,
    DEP_UNIMPLEMENTED_ACCESSOR,
};
use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::wording::header::{quoted_list, required_type};

/// What an associated type is called in a message: `abstract type` when it belongs to a CGP
/// abstract-type component (a context *chooses* its concrete type by wiring), and `associated type`
/// for any other trait, where no such choice exists.
pub fn assoc_type_noun(component: Option<&str>) -> &'static str {
    match component {
        Some(_) => "abstract type",
        None => "associated type",
    }
}

/// The statement that a context wires nothing for a key: a component marker for a plain missing
/// wiring, or a redirect path for an unterminated namespace lookup.
pub fn missing_wiring(context: &str, key: &str) -> String {
    format!("missing wiring for `{key}` on `{context}`")
}

/// The one sentence that states a root cause. It is the cause wherever the cause appears: the
/// headline of a block with this one cause, the `root cause:` line, and the tree's last entry
/// (see `cgp-knowledge-base/cargo-cgp/error-code.md`). A missing field, wiring, or table
/// entry names what is missing and where; a present field without `HasField` names the derive that
/// is missing, whose fix rides in a `help`; a projection mismatch names the type supplied and the
/// type required; an ordinary bound keeps the compiler's wording.
pub fn root_cause_lead(leaf: &Leaf) -> String {
    match leaf {
        Leaf::Field {
            name,
            owner,
            issue: FieldIssue::Missing,
            ..
        } => format!("missing field `{name}` on `{owner}`"),
        Leaf::MissingWiring { component, owner } => missing_wiring(owner, component),
        Leaf::MissingDispatchEntry { key, table } => {
            format!("missing entry for `{key}` in provider `{table}`")
        }
        Leaf::NotAProvider {
            provider,
            provider_trait,
        } => format!("`{provider}` is not a provider of `{provider_trait}`"),
        Leaf::MissingRedirectWiring { path, context } => missing_wiring(context, path),
        Leaf::Field { name, owner, .. } => {
            format!("`HasField` not derived for field `{name}` of `{owner}`")
        }
        Leaf::UnderivedFields { names, owner } => {
            format!(
                "`HasField` not derived for fields {} of `{owner}`",
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

/// The `CGP-E1xx` code of a root-cause leaf, carried both by the tree's last entry and by the
/// `root cause:` line that names it. Keyed by leaf kind: a missing field, a present-but-underived
/// field, a missing delegate entry (plain or redirect), a missing dispatch entry, a non-provider,
/// the two mismatch leaves (a `HasField` value type and any other associated type), and an ordinary
/// trait bound each get their own code.
pub fn dependency_leaf_code(leaf: &Leaf) -> &'static str {
    match leaf {
        Leaf::Field {
            issue: FieldIssue::Missing,
            ..
        } => DEP_MISSING_FIELD,
        Leaf::Field { .. } | Leaf::UnderivedFields { .. } => DEP_UNIMPLEMENTED_ACCESSOR,
        Leaf::MissingWiring { .. } | Leaf::MissingRedirectWiring { .. } => {
            DEP_MISSING_DELEGATE_ENTRY
        }
        Leaf::MissingDispatchEntry { .. } => DEP_MISSING_DISPATCH_ENTRY,
        Leaf::NotAProvider { .. } => DEP_NOT_A_PROVIDER,
        Leaf::FieldTypeMismatch { .. } => DEP_FIELD_TYPE_MISMATCH,
        Leaf::AssocTypeMismatch { .. } => DEP_ASSOC_TYPE_MISMATCH,
        Leaf::Bound { .. } => DEP_ORDINARY_BOUND,
    }
}

/// The terminal root-cause leaf as it appears in the dependency tree: [`root_cause_lead`] with its
/// [`dependency_leaf_code`] prefixed. The `root cause:` note lead repeats the same text and code.
pub fn dependency_tree_leaf(leaf: &Leaf) -> String {
    format!("[{}] {}", dependency_leaf_code(leaf), root_cause_lead(leaf))
}
