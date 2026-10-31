//! The one-sentence root-cause statement of a leaf, and the codes it carries.

use crate::code::{
    DEP_ASSOC_TYPE_MISMATCH, DEP_DATA_TRAIT_UNIMPLEMENTED, DEP_FIELD_ALREADY_SET,
    DEP_FIELD_NOT_OPTIONAL, DEP_FIELD_TYPE_MISMATCH, DEP_FIELDS_NOT_SET,
    DEP_MISSING_DELEGATE_ENTRY, DEP_MISSING_DISPATCH_ENTRY, DEP_MISSING_FIELD, DEP_MISSING_VARIANT,
    DEP_NOT_A_PROVIDER, DEP_ORDINARY_BOUND, DEP_PARTIAL_LACKS_DATA_TRAIT,
    DEP_PROVIDER_FOR_OTHER_ARGUMENTS, DEP_UNBOUNDED_PARAMETER, DEP_UNIMPLEMENTED_ACCESSOR,
    DEP_VARIANT_ALREADY_EXTRACTED, DEP_VARIANT_TYPE_MISMATCH, DEP_VARIANTS_NOT_EXTRACTED,
    DEP_WIRING_WITHOUT_PROVIDER_FOR,
};
use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::wording::{quoted_list, required_type};

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
/// type required, as a provider for other arguments names the arguments its impls take and the one
/// the wiring gives; an extensible-data cause names the field or variant whose state the operation
/// does not allow, or the data trait a type lacks; an ordinary bound keeps the compiler's wording.
pub fn root_cause_lead(leaf: &Leaf) -> String {
    match leaf {
        Leaf::Field {
            name,
            owner,
            issue: FieldIssue::Missing,
            ..
        } => format!("missing field `{name}` on `{owner}`"),
        Leaf::MissingWiring {
            component, owner, ..
        } => missing_wiring(owner, component),
        Leaf::MissingDispatchEntry { key, table } => {
            format!("missing entry for `{key}` in provider `{table}`")
        }
        Leaf::NotAProvider {
            provider,
            provider_trait,
            ..
        } => format!("`{provider}` is not a provider of `{provider_trait}`"),
        Leaf::ProviderForOtherArguments {
            provider,
            provider_trait,
            param,
            provided,
            required,
        } => {
            // The context reads as the `for` of the `#[cgp_impl]` header that names it; a component
            // argument reads as the parameter it binds.
            let argument = |ty: &str| match param {
                Some(param) => format!("`{param} = {ty}`"),
                None => format!("`{ty}`"),
            };
            let provided: Vec<String> = provided.iter().map(|ty| argument(ty)).collect();
            let preposition = if param.is_some() { "with" } else { "for" };
            format!(
                "`{provider}` implements `{provider_trait}` only {preposition} {}, not {}",
                provided.join(" or "),
                argument(required)
            )
        }
        Leaf::MissingRedirectWiring { path, context, .. } => missing_wiring(context, path),
        Leaf::WiringWithoutProviderFor {
            component, owner, ..
        } => format!("the wiring of `{component}` on `{owner}` has no `IsProviderFor` impl"),
        Leaf::UnboundedParameter {
            param,
            trait_name,
            item,
        } => format!("the type parameter `{param}` of `{item}` is not bounded by `{trait_name}`"),
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
        Leaf::FieldAlreadySet { name, owner } => {
            format!("field `{name}` of `{owner}` is already set")
        }
        Leaf::FieldsNotSet { names, owner } => match names.as_slice() {
            [one] => format!("field `{one}` of `{owner}` is not set"),
            _ => format!("fields {} of `{owner}` are not set", quoted_list(names)),
        },
        Leaf::MissingVariant { name, owner, .. } => {
            format!("missing variant `{name}` on `{owner}`")
        }
        Leaf::VariantsNotExtracted { names, owner } => match names.as_slice() {
            [one] => format!("variant `{one}` of `{owner}` is not extracted"),
            _ => format!(
                "variants {} of `{owner}` are not extracted",
                quoted_list(names)
            ),
        },
        Leaf::VariantTypeMismatch {
            name,
            owner,
            expected,
            actual,
        } => format!(
            "variant `{name}` on `{owner}` has type `{actual}`, but `{expected}` is required"
        ),
        Leaf::DataTraitUnimplemented {
            owner, trait_name, ..
        } => format!("`{owner}` does not implement `{trait_name}`"),
        Leaf::VariantAlreadyExtracted { name, owner } => {
            format!("variant `{name}` of `{owner}` is already extracted")
        }
        Leaf::FieldNotOptional { name, owner, .. } => {
            format!("field `{name}` of `{owner}` is not optional")
        }
        Leaf::PartialLacksDataTrait {
            kind,
            target,
            trait_name,
            ..
        } => format!(
            "{} of `{target}` does not implement `{trait_name}`",
            kind.with_article()
        ),
        Leaf::Bound { summary } => format!("the trait bound `{summary}` is not satisfied"),
    }
}

/// The `CGP-E1xx` code of a root-cause leaf, carried both by the tree's last entry and by the
/// `root cause:` line that names it. Each kind of leaf has its own code: a missing field, a
/// present-but-underived field, a missing delegate entry (plain or redirect), a missing dispatch
/// entry, a non-provider, a provider for other arguments, the two mismatch leaves (a `HasField`
/// value type and any other associated type), each extensible-data cause, a wiring without
/// `IsProviderFor`, an unbounded type parameter, and an ordinary trait bound.
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
        Leaf::ProviderForOtherArguments { .. } => DEP_PROVIDER_FOR_OTHER_ARGUMENTS,
        Leaf::WiringWithoutProviderFor { .. } => DEP_WIRING_WITHOUT_PROVIDER_FOR,
        Leaf::UnboundedParameter { .. } => DEP_UNBOUNDED_PARAMETER,
        Leaf::FieldTypeMismatch { .. } => DEP_FIELD_TYPE_MISMATCH,
        Leaf::AssocTypeMismatch { .. } => DEP_ASSOC_TYPE_MISMATCH,
        Leaf::FieldAlreadySet { .. } => DEP_FIELD_ALREADY_SET,
        Leaf::FieldsNotSet { .. } => DEP_FIELDS_NOT_SET,
        Leaf::MissingVariant { .. } => DEP_MISSING_VARIANT,
        Leaf::VariantsNotExtracted { .. } => DEP_VARIANTS_NOT_EXTRACTED,
        Leaf::VariantTypeMismatch { .. } => DEP_VARIANT_TYPE_MISMATCH,
        Leaf::DataTraitUnimplemented { .. } => DEP_DATA_TRAIT_UNIMPLEMENTED,
        Leaf::VariantAlreadyExtracted { .. } => DEP_VARIANT_ALREADY_EXTRACTED,
        Leaf::FieldNotOptional { .. } => DEP_FIELD_NOT_OPTIONAL,
        Leaf::PartialLacksDataTrait { .. } => DEP_PARTIAL_LACKS_DATA_TRAIT,
        Leaf::Bound { .. } => DEP_ORDINARY_BOUND,
    }
}

/// The terminal root-cause leaf as it appears in the dependency tree: [`root_cause_lead`] with its
/// [`dependency_leaf_code`] prefixed. The `root cause:` note lead repeats the same text and code.
pub fn dependency_tree_leaf(leaf: &Leaf) -> String {
    format!("[{}] {}", dependency_leaf_code(leaf), root_cause_lead(leaf))
}
