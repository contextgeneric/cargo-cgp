//! Rendering a dependency path's predicates as human-readable tree labels.
//!
//! This is where every CGP wiring trait is replaced by the concept it stands for, so the reader
//! never meets a raw `IsProviderFor` or `Symbol`. The rendered labels fold into a
//! [`DependencyTree`] spine that the [wording](cargo_cgp_error_processing::diagnosis) renders as
//! `cargo tree`-style text.

use cargo_cgp_error_processing::ComponentTraitNames;
use cargo_cgp_error_processing::code::{
    DEP_CONSUMER_TRAIT_IMPL, DEP_FIELD_TRAIT_IMPL, DEP_PROVIDER_TRAIT_IMPL, DEP_REDIRECT_LOOKUP,
    DEP_TRAIT_IMPL,
};
use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::tree::DependencyTree;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, CONS_TYPE,
    DELEGATE_COMPONENT_TRAIT, EITHER_TYPE, FIELD_TYPE, HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
    NIL_TYPE, REDIRECT_LOOKUP_TYPE, VOID_TYPE,
};
use crate::resolve::cgp_item::{
    decode_symbol, is_cgp_item, is_namespace_lookup_trait, is_provider_trait,
};

/// The human-readable label for one predicate in a dependency path, replacing each CGP wiring
/// trait with the concept it stands for: `CanUseComponent` with the consumer-trait impl,
/// `IsProviderFor` with the provider-trait impl (its provider trait, context, and provider struct),
/// `HasField` with the field-trait impl (the field and the struct that must carry it). Any other
/// trait — a user's own consumer or getter capability — is shown as a trait impl for its self type.
///
/// A `RedirectLookup<Ctx, Path>` provider is not a real provider impl but a namespace/`open`
/// redirection, so its `IsProviderFor` node reads as `redirect lookup to \`Path\` in \`Ctx\``; a
/// chain of them reads as its successive hops. The missing-delegate leaf such a chain bottoms out
/// on is not rendered here — the caller ([`resolve_leaves`](super::walk::resolve_leaves)) re-states
/// the root cause as the tree's terminal leaf.
///
/// The steps that carry no information for a reader return `None` and are dropped so the chain
/// stays legible: an `IsProviderFor` for the *context itself* (the delegation routing, as opposed
/// to the real provider), the `DelegateComponent` table lookup, a namespace lookup, and a provider
/// trait applied directly (which every `IsProviderFor` node already stands for).
pub(crate) fn label_for<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
    names: &ComponentNameMap,
) -> Option<String> {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;

    if is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE) {
        let consumer = marker_role(tcx, trait_ref.args.type_at(1), names, |n| n.consumer);
        // `CanUseComponent<Marker, Params>` — the component's extra parameters, reattached so a
        // generic component's consumer trait reads as written (`CanCalculateArea<u32, u64, bool>`).
        let generics = render_params(tcx, trait_ref.args.type_at(2));
        Some(format!(
            "[{DEP_CONSUMER_TRAIT_IMPL}] consumer trait impl `{consumer}{generics}` for context `{}`",
            render_ty(tcx, trait_ref.self_ty())
        ))
    } else if is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE) {
        // Drop the routing `IsProviderFor` for the context itself; keep the real providers.
        if trait_ref.self_ty() == context {
            return None;
        }
        // A `RedirectLookup<Ctx, Path>` is not a real provider but a namespace/`open` redirect;
        // show it as the redirection it performs rather than as a provider-trait impl, so a
        // chain of redirects reads as its successive hops.
        if let Some(path) = redirect_path(tcx, trait_ref.self_ty()) {
            return Some(format!(
                "[{DEP_REDIRECT_LOOKUP}] redirect lookup to `{path}` in `{context}`"
            ));
        }
        let provider = render_ty(tcx, trait_ref.self_ty());
        let provider_trait = marker_role(tcx, trait_ref.args.type_at(1), names, |n| n.provider);
        // `IsProviderFor<Provider, Marker, Context, Params>` — the third argument is the context,
        // the fourth the component's extra parameters (reattached to the provider trait).
        let provider_context = render_ty(tcx, trait_ref.args.type_at(2));
        let generics = render_params(tcx, trait_ref.args.type_at(3));
        Some(format!(
            "[{DEP_PROVIDER_TRAIT_IMPL}] provider trait impl `{provider_trait}{generics}` with context `{provider_context}` for provider `{provider}`"
        ))
    } else if is_cgp_item(tcx, did, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
        let field = decode_symbol(tcx, trait_ref.args.type_at(1))?;
        Some(format!(
            "[{DEP_FIELD_TRAIT_IMPL}] field trait impl `HasField` with field `{field}` for `{}`",
            render_ty(tcx, trait_ref.self_ty())
        ))
    } else if is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_namespace_lookup_trait(tcx, did)
        || is_provider_trait(tcx, did)
    {
        // Plumbing that carries no information for a reader: the `DelegateComponent` table lookup,
        // a namespace lookup, and a provider trait an `IsProviderFor` node already stands for. The
        // missing-delegate leaf a wiring chain bottoms out on is re-stated as the tree's terminal
        // by the caller (`resolve_leaves`), so dropping these here keeps the chain legible.
        None
    } else {
        Some(format!(
            "[{DEP_TRAIT_IMPL}] trait impl `{}` for `{}`",
            tcx.item_name(did),
            render_ty(tcx, trait_ref.self_ty())
        ))
    }
}

/// Render a type to its dependency-tree form, resugaring CGP's type-level list and sum spines back
/// to their surface macros: a `Cons<A, Cons<B, Nil>>` product spine to `Product![A, B]` and an
/// `Either<A, Either<B, Void>>` sum spine to `Sum![A, B]`, so a reader meets the field/variant list
/// as written rather than its raw right-nested spine. Every cell is anchored by `DefId` to the CGP
/// crate that defines it (`Cons`/`Nil` in `cgp-base-types`, `Either`/`Void` in `cgp-field`), so a
/// same-named type from another crate is never resugared. Each element is rendered recursively, so a
/// nested list (a `Sum!` inside a `Product!`, say) is resugared too; a non-spine type falls back to
/// its ordinary printed form (whose inner `Symbol!`/`Path!` the post-processing then resugars).
///
/// A list whose elements are *all* named fields — `Field<Symbol!("name"), Type>` — resugars one step
/// further to the record/variant surface form the shape describes: a product to `Struct! { name:
/// Type, … }` and a sum to `Enum! { Name(Type), … }`, so a `HasFields` field list reads as the struct
/// or enum it represents. `Struct!`/`Enum!` are not (yet) real CGP macros — like `Path!`'s `.*`
/// wildcard, they are a presentation form chosen for readability, not something that parses back.
pub(crate) fn render_ty<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> String {
    if let Some(elems) = cgp_spine(
        tcx,
        ty,
        CONS_TYPE,
        CGP_BASE_TYPES_CRATE,
        NIL_TYPE,
        CGP_BASE_TYPES_CRATE,
    ) {
        if let Some(fields) = named_fields(tcx, &elems) {
            let body = fields
                .iter()
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join(", ");
            return format!("Struct! {{ {body} }}");
        }
        return format!("Product![{}]", render_ty_list(tcx, &elems));
    }
    if let Some(elems) = cgp_spine(
        tcx,
        ty,
        EITHER_TYPE,
        CGP_FIELD_CRATE,
        VOID_TYPE,
        CGP_FIELD_CRATE,
    ) {
        if let Some(fields) = named_fields(tcx, &elems) {
            let body = fields
                .iter()
                .map(|(name, value)| format!("{name}({value})"))
                .collect::<Vec<_>>()
                .join(", ");
            return format!("Enum! {{ {body} }}");
        }
        return format!("Sum![{}]", render_ty_list(tcx, &elems));
    }
    ty.to_string()
}

/// Interpret every element of a resugared list as a named field `Field<Symbol!("name"), Value>`,
/// returning each `(name, rendered value)` pair — or `None` if *any* element is not such a field, so
/// the caller keeps the plain `Product!`/`Sum!` form. The `Field` cell is anchored by `DefId` to
/// `cgp-field`, its name decoded from the `Symbol!` tag, and its value rendered recursively so a
/// nested record/variant resugars in turn.
fn named_fields<'tcx>(tcx: TyCtxt<'tcx>, elems: &[Ty<'tcx>]) -> Option<Vec<(String, String)>> {
    elems
        .iter()
        .map(|elem| {
            let ty::Adt(def, args) = elem.kind() else {
                return None;
            };
            if !is_cgp_item(tcx, def.did(), FIELD_TYPE, CGP_FIELD_CRATE) {
                return None;
            }
            // `Field<Tag, Value>` — the tag is a `Symbol!` name, the value its type.
            let name = decode_symbol(tcx, args.type_at(0))?;
            Some((name, render_ty(tcx, args.type_at(1))))
        })
        .collect()
}

/// Render a spine's collected element types as a comma-separated list, each recursively through
/// [`render_ty`] so a nested spine resugars in turn.
fn render_ty_list<'tcx>(tcx: TyCtxt<'tcx>, elems: &[Ty<'tcx>]) -> String {
    elems
        .iter()
        .map(|elem| render_ty(tcx, *elem))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The head types of a CGP type-level spine `Cell<Head, Tail>` ended by `Terminator` — the element
/// list a `Product!`/`Sum!` macro was written with — or `None` when `ty` is not such a spine. The
/// first cell must be a `Cell` (a bare terminator is not resugared, so an empty list is left as its
/// terminator type), each `Cell` and the final `Terminator` are checked by `DefId` against the given
/// CGP crate, and an open-ended spine (a tail that is neither another `Cell` nor the terminator, such
/// as a generic "rest" parameter) declines so only a fully-terminated list is resugared.
fn cgp_spine<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: Ty<'tcx>,
    cell: &str,
    cell_crate: &str,
    terminator: &str,
    terminator_crate: &str,
) -> Option<Vec<Ty<'tcx>>> {
    // Require the first node to be a spine cell, so a bare terminator (an empty list) is not
    // resugared into `Product![]`/`Sum![]` where it more likely reads as its plain type.
    let ty::Adt(def, _) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), cell, cell_crate) {
        return None;
    }

    let mut elems = Vec::new();
    let mut current = ty;
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 4096 {
            return None;
        }
        let ty::Adt(def, args) = current.kind() else {
            return None;
        };
        let did = def.did();
        if is_cgp_item(tcx, did, cell, cell_crate) {
            // `Cell<Head, Tail>` — collect the head and continue down the tail.
            elems.push(args.type_at(0));
            current = args.type_at(1);
        } else if is_cgp_item(tcx, did, terminator, terminator_crate) {
            return Some(elems);
        } else {
            // A tail that is neither a further cell nor the terminator: not a closed CGP list.
            return None;
        }
    }
}

/// The redirect path of a `RedirectLookup<Ctx, Path>` provider — its second type argument — or
/// `None` when `provider` is not a `RedirectLookup`. Anchored to the CGP crate that defines the
/// type, so a same-named type elsewhere is never mistaken for it.
fn redirect_path<'tcx>(tcx: TyCtxt<'tcx>, provider: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let ty::Adt(def, args) = provider.kind() else {
        return None;
    };
    is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE).then(|| args.type_at(1))
}

/// Fold a path's rendered labels into a single-spine dependency tree, root first.
pub(crate) fn spine(labels: Vec<String>) -> Option<DependencyTree> {
    let mut rev = labels.into_iter().rev();
    let mut node = DependencyTree::leaf(rev.next()?);
    for label in rev {
        node = DependencyTree::node(label, vec![node]);
    }
    Some(node)
}

/// Render a component's extra type parameters as a trait generic list — `<u32, u64, bool>`, or the
/// empty string when there are none. CGP groups the parameters into the `Params` slot of
/// `CanUseComponent`/`IsProviderFor`: none as the unit `()`, a single one bare, several as a tuple,
/// so a tuple is unwrapped and a bare parameter reattached directly.
pub(crate) fn render_params<'tcx>(tcx: TyCtxt<'tcx>, params: Ty<'tcx>) -> String {
    match params.kind() {
        ty::Tuple(elems) if elems.is_empty() => String::new(),
        ty::Tuple(elems) => format!(
            "<{}>",
            elems
                .iter()
                .map(|elem| render_ty(tcx, elem))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => format!("<{}>", render_ty(tcx, params)),
    }
}

/// Resolve a component marker type to its consumer or provider trait name through the name map,
/// keyed by the marker's *full path* so two same-named markers in different modules never collide.
/// Falls back to the marker's bare item name when the component is not in the map, and to the
/// marker's printed form when it is not an ADT at all.
pub(crate) fn marker_role(
    tcx: TyCtxt<'_>,
    marker: Ty<'_>,
    names: &ComponentNameMap,
    role: impl Fn(ComponentTraitNames) -> String,
) -> String {
    let ty::Adt(def, _) = marker.kind() else {
        return marker.to_string();
    };
    match names.get_by_path(&tcx.def_path_str(def.did())) {
        Some(entry) => role(entry),
        None => tcx.item_name(def.did()).to_string(),
    }
}
