//! Rendering a dependency path's predicates as human-readable tree labels.
//!
//! This is where every CGP wiring trait is replaced by the concept it stands for, so the reader
//! never meets a raw `IsProviderFor` or `Symbol`. The rendered labels fold into a
//! [`DependencyTree`] spine that the [wording](cargo_cgp_error_processing::diagnosis) renders as
//! `cargo tree`-style text.

use cargo_cgp_error_processing::code::{
    DEP_CONSUMER_TRAIT_IMPL, DEP_FIELD_TRAIT_IMPL, DEP_PROVIDER_TRAIT_IMPL, DEP_REDIRECT_LOOKUP,
    DEP_TRAIT_IMPL,
};
use cargo_cgp_error_processing::tree::DependencyTree;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, CONS_TYPE,
    DELEGATE_COMPONENT_TRAIT, EITHER_TYPE, FIELD_TYPE, HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
    NIL_TYPE, REDIRECT_LOOKUP_TYPE, VOID_TYPE,
};
use crate::resolve::cgp_item::{
    decode_symbol, is_cgp_item, is_consumer_trait, is_namespace_lookup_trait, is_provider_trait,
};

/// The human-readable label for one predicate in a dependency path, replacing each CGP wiring
/// trait with the concept it stands for. Crucially it reads the **real** consumer and provider
/// trait obligations that the walk descends — `Ctx: ConsumerTrait<…>` and `Provider:
/// ProviderTrait<Ctx, …>` — and takes every name straight off the trait `DefId` and the
/// obligation's own type arguments. It does *not* read `CanUseComponent`/`IsProviderFor`: those
/// are the check-trait scaffolding cargo-cgp treats as removable, and the resolver never depends on
/// them for a name.
///
/// - A consumer-trait obligation on the context becomes the consumer-trait impl.
/// - A provider-trait obligation whose `Self` is a real provider becomes the provider-trait impl
///   (its trait, context, provider struct, and the component's extra parameters). A
///   `RedirectLookup<Ctx, Path>` provider is a namespace/`open` redirection hop instead, so a chain
///   of them reads as its successive hops.
/// - `HasField` becomes the field-trait impl (the field and the struct that must carry it).
/// - Any other trait — a user's own capability or getter — is shown as a trait impl for its self.
///
/// The steps that carry no information for a reader return `None` and are dropped: the
/// `CanUseComponent`/`IsProviderFor` scaffolding, a provider-trait obligation *for the context
/// itself* (delegation routing), the `DelegateComponent` table lookup, and a namespace lookup. The
/// missing-delegate leaf a wiring chain bottoms out on is not rendered here — the caller
/// ([`resolve_leaves`](super::walk::resolve_leaves)) re-states the root cause as the terminal leaf.
pub(crate) fn label_for<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
) -> Option<String> {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;
    let self_ty = trait_ref.self_ty();

    // The check-trait scaffolding and table plumbing carry nothing for the reader; the real
    // consumer/provider obligations beside them do. Dropping these is also what keeps the resolver
    // off `IsProviderFor`: it is never read for a name, only recognized here to be discarded.
    if is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_namespace_lookup_trait(tcx, did)
    {
        return None;
    }

    if is_cgp_item(tcx, did, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
        let field = decode_symbol(tcx, trait_ref.args.type_at(1))?;
        return Some(format!(
            "[{DEP_FIELD_TRAIT_IMPL}] field trait impl `HasField` with field `{field}` for `{}`",
            render_ty(tcx, self_ty)
        ));
    }

    if is_provider_trait(tcx, did) {
        // A provider-trait obligation for the context itself is delegation routing, dropped.
        if self_ty == context {
            return None;
        }
        // A `RedirectLookup<Ctx, Path>` provider is a namespace/`open` redirection hop, not a real
        // provider impl, so a chain of them reads as its successive hops.
        if let Some(path) = redirect_path(tcx, self_ty) {
            return Some(format!(
                "[{DEP_REDIRECT_LOOKUP}] redirect lookup to `{path}` in `{context}`"
            ));
        }
        // `Provider: ProviderTrait<Ctx, Params…>` — trait name, context, provider, and the
        // component's extra parameters all read straight off the obligation, no marker or map. The
        // context is the first type argument after `Self`, indexed by *type* position because a
        // component's lifetime parameters sort ahead of the context in the argument list
        // (`ReferenceGetter<'a, Ctx, T>`); indexing by argument position would land on the region
        // and abort the compiler. A trait with no such argument is not a shape `#[cgp_component]`
        // emits, so it falls through to the plain label.
        if let Some(provider_context) = trait_ref.args.types().nth(1) {
            let generics = trait_generics(tcx, trait_ref, 2);
            return Some(format!(
                "[{DEP_PROVIDER_TRAIT_IMPL}] provider trait impl `{}{generics}` with context `{}` for provider `{}`",
                tcx.item_name(did),
                render_ty(tcx, provider_context),
                render_ty(tcx, self_ty)
            ));
        }
    }

    if is_consumer_trait(tcx, did) && self_ty == context {
        // `Ctx: ConsumerTrait<Params…>` — the consumer name and its parameters read directly.
        let generics = trait_generics(tcx, trait_ref, 1);
        return Some(format!(
            "[{DEP_CONSUMER_TRAIT_IMPL}] consumer trait impl `{}{generics}` for context `{}`",
            tcx.item_name(did),
            render_ty(tcx, self_ty)
        ));
    }

    // A user's own capability/getter trait, or a terminal ordinary bound.
    Some(format!(
        "[{DEP_TRAIT_IMPL}] trait impl `{}` for `{}`",
        tcx.item_name(did),
        render_ty(tcx, self_ty)
    ))
}

/// Render a trait obligation's type arguments after `skip` leading ones as a generic list —
/// `<u32, u64>` — or the empty string when there are none. For a consumer obligation `Ctx:
/// C<A, B>` the arguments are spread (skip the `Self`=`Ctx`), and for a provider obligation `P:
/// T<Ctx, A, B>` likewise (skip `Self` and the leading context), so the component's extra
/// parameters reattach to the trait exactly as written — read straight off the obligation rather
/// than from a `CanUseComponent`/`IsProviderFor` params tuple.
pub(crate) fn trait_generics<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_ref: ty::TraitRef<'tcx>,
    skip: usize,
) -> String {
    let params: Vec<String> = trait_ref
        .args
        .types()
        .skip(skip)
        .map(|ty| render_ty(tcx, ty))
        .collect();
    if params.is_empty() {
        String::new()
    } else {
        format!("<{}>", params.join(", "))
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
