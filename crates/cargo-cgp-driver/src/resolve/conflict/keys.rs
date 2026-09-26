//! Rendering a conflicting entry's key, provider, and redirect path to their surface forms.

use cargo_cgp_error_processing::WiringKey;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, NIL_TYPE, PATH_CONS_TYPE, REDIRECT_LOOKUP_TYPE,
};
use crate::resolve::cgp_item::{decode_symbol, is_cgp_item, is_component_marker};

/// The surface form of a wiring key — a `DelegateComponent` key, or the `Self` of a namespace
/// lookup-trait entry: a bare component marker, an ordinary type, an `@`-path, or a blanket
/// forwarding tagged by the namespace/table trait that keys it. A component marker is told from an
/// ordinary type structurally, by whether a provider trait keys on it
/// ([`is_component_marker`]), so a per-type default's `String` reads as a type rather than as a
/// component nobody defined. `None` for a key the classifier cannot render — one still carrying a
/// generic parameter, an inference variable, or a placeholder — so the rewrite is declined rather
/// than guessed.
pub(crate) fn describe_key<'tcx>(
    tcx: TyCtxt<'tcx>,
    key: Ty<'tcx>,
    impl_did: DefId,
) -> Option<WiringKey> {
    match key.kind() {
        ty::Param(_) => Some(WiringKey::Blanket(bounding_trait(tcx, impl_did, key)?)),
        ty::Adt(def, _) if is_cgp_item(tcx, def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE) => {
            Some(WiringKey::Path(render_path(tcx, key)?))
        }
        ty::Adt(def, _) if is_component_marker(tcx, key) => {
            Some(WiringKey::Component(tcx.item_name(def.did()).to_string()))
        }
        _ if key.has_param() || key.has_non_region_infer() || key.has_placeholders() => None,
        _ => Some(WiringKey::Type(
            tcx.erase_and_anonymize_regions(key).to_string(),
        )),
    }
}

/// The namespace lookup trait an entry registers into, rendered the way it is written in a
/// `#[default_impl(… in …)]` or a `cgp_namespace!` header: the trait name with its leading
/// arguments but without the components-table parameter the macro appends, so a plain namespace
/// reads `AppNamespace` and a per-type table reads `DefaultImpls1<ShowImplComponent>`. It is the
/// subject a namespace conflict is reported on, standing where a context does for a table
/// collision.
pub(crate) fn render_lookup_trait(tcx: TyCtxt<'_>, impl_did: DefId) -> String {
    let trait_ref = tcx
        .impl_trait_ref(impl_did)
        .instantiate_identity()
        .skip_norm_wip();
    let name = tcx.item_name(trait_ref.def_id).to_string();
    // A lookup trait's shape is `Trait<…, Components>` and the macros append that table parameter
    // themselves, so the arguments are `[Self, leading…, Table]` and the written ones sit between.
    let args = trait_ref.args.as_slice();
    let leading: Vec<String> = args
        .get(1..args.len().saturating_sub(1))
        .unwrap_or_default()
        .iter()
        .map(|arg| tcx.erase_and_anonymize_regions(*arg).to_string())
        .collect();
    if leading.is_empty() {
        name
    } else {
        format!("{name}<{}>", leading.join(", "))
    }
}

/// Render a `PathCons<..>` key back to its bare `@a.b.*` surface form — the typed counterpart of
/// the text `resugar_path`, done straight off the types so a generic tail or loop parameter is
/// read as a `.*` wildcard rather than a printed parameter name. A lowercase `Symbol` segment
/// decodes to its string; a named segment keeps its type name, and any other type prints as itself;
/// a generic segment reads `*` in its own position, so the concrete segments after it still show
/// (`@ComputerComponent.*.u64.*`); and a generic tail ends the path in `.*`, folded into a generic
/// last segment so a `for`-loop key reads `@a.b.*` rather than `@a.b.*.*`. The bare `@…` form (no
/// `Path!(…)` wrapper) is what the rewritten conflict message uses. `None` on a spine that is not
/// `PathCons`/`Nil`.
pub(crate) fn render_path<'tcx>(tcx: TyCtxt<'tcx>, path: Ty<'tcx>) -> Option<String> {
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
            // A generic tail — the rest of the path is open, so it reads as `.*`, unless the last
            // segment already reads `*`.
            ty::Param(_) => {
                wildcard = segments.last().is_none_or(|last| last != "*");
                break;
            }
            ty::Adt(def, args) => {
                if is_cgp_item(tcx, def.did(), NIL_TYPE, CGP_BASE_TYPES_CRATE) {
                    break;
                }
                if !is_cgp_item(tcx, def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE) {
                    return None;
                }
                let head = args.type_at(0);
                match head.kind() {
                    // A generic segment (a `for`-loop key parameter, or an `open` key's per-entry
                    // generic) matches any value in its position.
                    ty::Param(_) => segments.push("*".to_owned()),
                    ty::Adt(head_def, _) => {
                        if let Some(symbol) = decode_symbol(tcx, head) {
                            segments.push(symbol);
                        } else {
                            segments.push(tcx.item_name(head_def.did()).to_string());
                        }
                    }
                    // A primitive, reference, or tuple segment, such as the `u64` an input-keyed
                    // `open` entry names, reads as the type itself.
                    _ => segments.push(tcx.erase_and_anonymize_regions(head).to_string()),
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
pub(crate) fn redirect_path<'tcx>(tcx: TyCtxt<'tcx>, delegate: Ty<'tcx>) -> Option<String> {
    let ty::Adt(def, args) = delegate.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE) {
        return None;
    }
    render_path(tcx, args.type_at(1))
}

/// Render a provider (a `DelegateComponent` entry's `Delegate`, when it is a plain provider rather
/// than a `RedirectLookup`) to the surface name the fix message uses, e.g. `GreetHello` or
/// `UseType<String>`. CGP path prefixes are stripped by the post-processing pass that runs after.
pub(crate) fn render_provider<'tcx>(tcx: TyCtxt<'tcx>, delegate: Ty<'tcx>) -> String {
    tcx.erase_and_anonymize_regions(delegate).to_string()
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
pub(crate) fn bounding_trait_ref<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_did: DefId,
    key: Ty<'tcx>,
) -> Option<ty::TraitRef<'tcx>> {
    let sized = tcx.lang_items().sized_trait();
    for &(clause, _) in tcx.clauses_of(impl_did).clauses {
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
