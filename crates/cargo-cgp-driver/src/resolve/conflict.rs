//! Classifying a duplicate-key wiring conflict by querying the trait solver.
//!
//! A duplicate key in `delegate_components!` makes the expansion emit two overlapping
//! `DelegateComponent` impls, so the compiler reports the coherence error (`E0119`) *twice* —
//! once for the `DelegateComponent` table impl and once for the `IsProviderFor` forwarding impl
//! the same entry generates. The two are one logical mistake, so this module recognizes the
//! pair: it drops the redundant `IsProviderFor` half and rewrites the `DelegateComponent` half
//! into a message that names the colliding key(s).
//!
//! Everything is recovered from the compiler, not from the error text. The failing diagnostic's
//! primary span equals `tcx.def_span` of the *conflicting* `DelegateComponent` impl (the macro
//! re-spans each entry onto its key token, and rustc aims the `E0119` at that impl's def-span),
//! so the classifier matches the caret to that impl and reads the entry off it — its self type,
//! its key, and its `Delegate` — then does the same for the "first implementation here" impl
//! found at the diagnostic's other labelled span. Each `DelegateComponent` DefId is anchored to
//! [`CGP_COMPONENT_CRATE`], exactly as the rest of [`resolve`](crate::resolve) is, so a same-named
//! trait from another crate can never drive the rewrite.

use cargo_cgp_error_processing::{WiringConflict, WiringKey};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode, Unnormalized};
use rustc_span::Span;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, REDIRECT_LOOKUP_TYPE};
use crate::resolve::cgp_item::{decode_symbol, find_cgp_trait, is_cgp_item};

/// Which trait an `E0119` conflict is about, read from the diagnostic's own message — the only
/// thing that tells the `DelegateComponent` half of the pair from its `IsProviderFor` half, since
/// both carry the same caret. The classifier still verifies a genuine CGP `DelegateComponent`
/// conflict sits at that caret before acting, so the message is used only to route within a
/// confirmed pair.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConflictTrait {
    Delegate,
    IsProviderFor,
}

/// What to do with an `E0119` recognized as a duplicate-key wiring conflict.
pub enum ConflictAction {
    /// Drop this diagnostic entirely — it is the redundant `IsProviderFor` half of the pair.
    Suppress,
    /// Rewrite this diagnostic's header from the recovered conflict.
    Rewrite(WiringConflict),
}

/// Whether two spans cover the same source range, ignoring `SyntaxContext`. The two halves of the
/// `E0119` pair are re-spanned onto the same entry token but carry different contexts, so matching
/// a caret to an impl must compare ranges, not whole spans.
fn same_range(a: Span, b: Span) -> bool {
    a.lo() == b.lo() && a.hi() == b.hi()
}

/// One local `DelegateComponent` impl, read off the compiler: where it was written, the context
/// it wires, the key it maps, and its `Delegate` (the provider or redirect it maps to).
struct DelegateImpl<'tcx> {
    def_span: Span,
    self_ty: Ty<'tcx>,
    key: Ty<'tcx>,
    delegate: Option<Ty<'tcx>>,
    impl_did: DefId,
}

/// Classify an `E0119` at `primary_span` as a duplicate-key wiring conflict, or return `None` if
/// no CGP `DelegateComponent` conflict sits at that caret (so the caller leaves the diagnostic to
/// the ordinary fallback). `variant` routes within a confirmed pair — the `IsProviderFor` half is
/// dropped, the `DelegateComponent` half rewritten. `label_spans` are the diagnostic's labelled
/// spans, searched (minus the primary) for the "first implementation here" impl.
pub fn classify_wiring_conflict(
    tcx: TyCtxt<'_>,
    variant: ConflictTrait,
    primary_span: Span,
    label_spans: &[Span],
) -> Option<ConflictAction> {
    let delegate_did = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;
    let impls = local_delegate_impls(tcx, delegate_did);

    // The conflicting entry is the impl whose def-span the caret sits on. Its presence is what
    // confirms this `E0119` is a real `DelegateComponent` duplicate — a same-named `IsProviderFor`
    // conflict elsewhere (e.g. a duplicate provider *name*) has no such impl at its caret. Spans
    // are matched by source range, not `==`: the `IsProviderFor` and `DelegateComponent` halves of
    // the pair carry the same caret range but under different `SyntaxContext`s, so an exact
    // comparison would miss the match for one half.
    let conflicting = impls
        .iter()
        .find(|i| same_range(i.def_span, primary_span))?;

    if variant == ConflictTrait::IsProviderFor {
        return Some(ConflictAction::Suppress);
    }

    // The "first implementation here" impl is at whichever other labelled span matches an impl.
    let first = label_spans
        .iter()
        .filter(|&&sp| !same_range(sp, primary_span))
        .find_map(|&sp| impls.iter().find(|i| same_range(i.def_span, sp)));

    Some(ConflictAction::Rewrite(build_conflict(
        tcx,
        conflicting,
        first,
    )?))
}

/// Word the recovered impls into a [`WiringConflict`], picking the shape from how the two entries
/// relate: a redirect (one or both `Delegate`s are `RedirectLookup`), or a plain
/// duplicate/overlap otherwise. `None` when a key cannot be rendered to a surface form.
fn build_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    conflicting: &DelegateImpl<'tcx>,
    first: Option<&DelegateImpl<'tcx>>,
) -> Option<WiringConflict> {
    let context = tcx
        .erase_and_anonymize_regions(conflicting.self_ty)
        .to_string();
    let key = describe_key(tcx, conflicting.key, conflicting.impl_did)?;

    let conflicting_redirect = conflicting.delegate.and_then(|d| redirect_path(tcx, d));
    let first_redirect = first
        .and_then(|f| f.delegate)
        .and_then(|d| redirect_path(tcx, d));

    match (first_redirect, conflicting_redirect) {
        // Both entries redirect the same key — a duplicate redirect (the two targets may differ).
        (Some(first_path), Some(second_path)) => {
            return Some(WiringConflict::DuplicateRedirect {
                context,
                key,
                first_path,
                second_path,
            });
        }
        // One entry redirects the key while the other sets it directly — a redirect collision. The
        // provider named in the fix comes from the *direct* entry (the non-redirect one).
        (Some(path), None) => {
            let provider = render_provider(tcx, conflicting.delegate?);
            return Some(WiringConflict::Redirect {
                context,
                key,
                path,
                provider,
            });
        }
        (None, Some(path)) => {
            let provider = render_provider(tcx, first?.delegate?);
            return Some(WiringConflict::Redirect {
                context,
                key,
                path,
                provider,
            });
        }
        (None, None) => {}
    }

    let Some(first) = first else {
        return Some(WiringConflict::Duplicate { context, key });
    };
    let first_key = describe_key(tcx, first.key, first.impl_did)?;

    // A direct wiring can also collide with a *namespace* that redirects the same key — the blanket
    // forwarding's `Delegate` for that concrete key normalizes to a `RedirectLookup`. Recover that
    // here, so it reads as a redirect collision rather than a bare overlap.
    if let Some(redirect) =
        namespace_redirect_conflict(tcx, &context, conflicting, first, &key, &first_key)
    {
        return Some(redirect);
    }

    Some(match (&first_key, &key) {
        // Two blanket forwardings, each over every key — a context joining more than one
        // namespace (`namespace` desugars to a bare-key `for` loop, so the two are the same shape).
        (WiringKey::Blanket(first_ns), WiringKey::Blanket(second_ns)) => {
            WiringConflict::MultipleNamespaces {
                context,
                first: first_ns.clone(),
                second: second_ns.clone(),
            }
        }
        _ if first_key == key => WiringConflict::Duplicate { context, key },
        _ => WiringConflict::Overlap {
            context,
            conflicting: key,
            first: first_key,
        },
    })
}

/// Recognize a direct wiring colliding with a *namespace* that redirects the same key: one of the
/// two entries is a blanket namespace forwarding, the other a concrete key the namespace maps to a
/// `RedirectLookup`. The message then reads as a redirect collision — wire the direct entry's
/// provider under the redirected path — rather than a bare overlap. `None` when neither entry is a
/// blanket, or the namespace does not redirect the concrete key.
fn namespace_redirect_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: &str,
    conflicting: &DelegateImpl<'tcx>,
    first: &DelegateImpl<'tcx>,
    key: &WiringKey,
    first_key: &WiringKey,
) -> Option<WiringConflict> {
    let is_blanket = |k: &WiringKey| matches!(k, WiringKey::Blanket(_));
    let (blanket, concrete) = match (is_blanket(key), is_blanket(first_key)) {
        (true, false) => (conflicting, first),
        (false, true) => (first, conflicting),
        _ => return None,
    };
    let path = namespace_redirect(tcx, blanket, concrete.key)?;
    Some(WiringConflict::Redirect {
        context: context.to_owned(),
        key: describe_key(tcx, concrete.key, concrete.impl_did)?,
        path,
        provider: render_provider(tcx, concrete.delegate?),
    })
}

/// The redirected path a blanket namespace forwarding maps `concrete_key` to, if it maps it to a
/// `RedirectLookup` at all. Recovered by normalizing the namespace trait's `Delegate` projection for
/// that key — `<concrete_key as DefaultNamespace<Ctx>>::Delegate` — through the trait solver, the
/// same re-entrant normalization the typed resolver uses. `None` unless the key is fully concrete
/// and the projection resolves to a `RedirectLookup`.
fn namespace_redirect<'tcx>(
    tcx: TyCtxt<'tcx>,
    blanket: &DelegateImpl<'tcx>,
    concrete_key: Ty<'tcx>,
) -> Option<String> {
    // Only a fully concrete key resolves through the namespace to a single value.
    if concrete_key.has_param() || concrete_key.has_non_region_infer() {
        return None;
    }
    // The blanket's bounding trait `<blanket key>: NsTrait<Ctx>`, rebuilt with the concrete key as
    // `Self` so the projection names the mapping for *this* key.
    let ns_ref = bounding_trait_ref(tcx, blanket.impl_did, blanket.key)?;
    let delegate_did = tcx
        .associated_items(ns_ref.def_id)
        .in_definition_order()
        .find(|item| item.name().as_str() == "Delegate")?
        .def_id;
    let mut args: Vec<ty::GenericArg<'tcx>> = ns_ref.args.iter().collect();
    *args.first_mut()? = concrete_key.into();

    let projection = Ty::new_projection(tcx, ty::IsRigid::No, delegate_did, args);
    let delegate = normalize(tcx, projection)?;
    let ty::Adt(def, redirect_args) = delegate.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE) {
        return None;
    }
    render_path(tcx, redirect_args.type_at(1))
}

/// Normalize `ty` through a fresh inference context, returning the resolved type — or `None` if it
/// does not resolve to a concrete type (an ambiguous or unresolved projection leaves inference vars
/// or an alias behind). Re-entering the solver mid-emission is the same technique the typed resolver
/// relies on.
fn normalize<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let normalized = ocx.normalize(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        Unnormalized::new_wip(ty),
    );
    let normalized = infcx.resolve_vars_if_possible(normalized);
    if normalized.has_non_region_infer() {
        return None;
    }
    Some(tcx.erase_and_anonymize_regions(normalized))
}

/// Render a provider (a `DelegateComponent` entry's `Delegate`, when it is a plain provider rather
/// than a `RedirectLookup`) to the surface name the fix message uses, e.g. `GreetHello` or
/// `UseType<String>`. CGP path prefixes are stripped by the post-processing pass that runs after.
fn render_provider<'tcx>(tcx: TyCtxt<'tcx>, delegate: Ty<'tcx>) -> String {
    tcx.erase_and_anonymize_regions(delegate).to_string()
}

/// The surface form of a `DelegateComponent` key: a bare component marker, an `@`-path, or a
/// blanket forwarding tagged by the namespace/table trait that keys it. `None` for a key the
/// classifier cannot render (so the rewrite is declined rather than guessed).
fn describe_key<'tcx>(tcx: TyCtxt<'tcx>, key: Ty<'tcx>, impl_did: DefId) -> Option<WiringKey> {
    match key.kind() {
        ty::Adt(def, _) => {
            if is_cgp_item(
                tcx,
                def.did(),
                "PathCons",
                crate::config::CGP_BASE_TYPES_CRATE,
            ) {
                Some(WiringKey::Path(render_path(tcx, key)?))
            } else {
                Some(WiringKey::Component(tcx.item_name(def.did()).to_string()))
            }
        }
        ty::Param(_) => Some(WiringKey::Blanket(bounding_trait(tcx, impl_did, key)?)),
        _ => None,
    }
}

/// Render a `PathCons<..>` key back to its bare `@a.b.*` surface form — the typed counterpart of
/// the text `resugar_path`, done straight off the types so a generic tail or loop parameter is
/// read as a `.*` wildcard rather than a printed parameter name. A lowercase `Symbol` segment
/// decodes to its string; a named segment keeps its type name; a `Param` anywhere ends the path in
/// `.*`. The bare `@…` form (no `Path!(…)` wrapper) is what the rewritten conflict message uses.
/// `None` on a spine that is not `PathCons`/`Nil`.
fn render_path<'tcx>(tcx: TyCtxt<'tcx>, path: Ty<'tcx>) -> Option<String> {
    let mut segments: Vec<String> = Vec::new();
    let mut wildcard = false;
    let mut current = path;
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 256 {
            return None;
        }
        match current.kind() {
            // A generic tail — the rest of the path is open, so it reads as `.*`.
            ty::Param(_) => {
                wildcard = true;
                break;
            }
            ty::Adt(def, args) => {
                if is_cgp_item(tcx, def.did(), "Nil", crate::config::CGP_BASE_TYPES_CRATE) {
                    break;
                }
                if !is_cgp_item(
                    tcx,
                    def.did(),
                    "PathCons",
                    crate::config::CGP_BASE_TYPES_CRATE,
                ) {
                    return None;
                }
                let head = args.type_at(0);
                match head.kind() {
                    // A generic segment (a `for`-loop key parameter) opens the path here.
                    ty::Param(_) => {
                        wildcard = true;
                        break;
                    }
                    ty::Adt(head_def, _) => {
                        if let Some(symbol) = decode_symbol(tcx, head) {
                            segments.push(symbol);
                        } else {
                            segments.push(tcx.item_name(head_def.did()).to_string());
                        }
                    }
                    _ => return None,
                }
                current = args.type_at(1);
            }
            _ => return None,
        }
    }

    let mut rendered = String::from("@");
    rendered.push_str(&segments.join("."));
    if wildcard {
        if !segments.is_empty() {
            rendered.push('.');
        }
        rendered.push('*');
    }
    Some(rendered)
}

/// If `delegate` is a `RedirectLookup<Table, Path>`, render its `Path` (the second argument) to
/// its bare `@..` surface form — the redirected key the user should set. `None` for any other
/// `Delegate`, which is an ordinary provider, not a redirect.
fn redirect_path<'tcx>(tcx: TyCtxt<'tcx>, delegate: Ty<'tcx>) -> Option<String> {
    let ty::Adt(def, args) = delegate.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE) {
        return None;
    }
    render_path(tcx, args.type_at(1))
}

/// The name of the namespace/table trait that keys a blanket `DelegateComponent<Key>` impl (e.g.
/// `DefaultNamespace`), read off its [`bounding_trait_ref`].
fn bounding_trait<'tcx>(tcx: TyCtxt<'tcx>, impl_did: DefId, key: Ty<'tcx>) -> Option<String> {
    Some(
        tcx.item_name(bounding_trait_ref(tcx, impl_did, key)?.def_id)
            .to_string(),
    )
}

/// The bounding trait ref of a blanket `DelegateComponent<Key>` impl — the single non-`Sized` bound
/// on its generic key parameter (`Key: DefaultNamespace<Ctx>`), the namespace or `for`-loop table
/// the forwarding routes through. `None` if no such bound is found.
fn bounding_trait_ref<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_did: DefId,
    key: Ty<'tcx>,
) -> Option<ty::TraitRef<'tcx>> {
    let sized = tcx.lang_items().sized_trait();
    for &(clause, _) in tcx.predicates_of(impl_did).predicates {
        let Some(predicate) = clause.as_trait_clause() else {
            continue;
        };
        let trait_ref = predicate.skip_binder().trait_ref;
        if trait_ref.self_ty() == key && Some(trait_ref.def_id) != sized {
            return Some(trait_ref);
        }
    }
    None
}

/// Every local impl of the `DelegateComponent` trait, read into [`DelegateImpl`]s. Only local
/// impls can be an entry the user wrote; library blanket impls sit in another crate and never
/// match a caret in the user's source.
fn local_delegate_impls<'tcx>(tcx: TyCtxt<'tcx>, delegate_did: DefId) -> Vec<DelegateImpl<'tcx>> {
    let mut impls = Vec::new();
    for impl_did in tcx.all_impls(delegate_did) {
        if !impl_did.is_local() {
            continue;
        }
        let self_ty = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
        // `DelegateComponent<Key>` — args are `[Self, Key]`.
        let key = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip()
            .args
            .type_at(1);
        let delegate = tcx
            .associated_items(impl_did)
            .in_definition_order()
            .find(|item| item.name().as_str() == "Delegate")
            .map(|item| {
                tcx.type_of(item.def_id)
                    .instantiate_identity()
                    .skip_norm_wip()
            });
        impls.push(DelegateImpl {
            def_span: tcx.def_span(impl_did),
            self_ty,
            key,
            delegate,
            impl_did,
        });
    }
    impls
}
