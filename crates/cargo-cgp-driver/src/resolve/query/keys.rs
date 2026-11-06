//! Rendering a conflicting entry's key, provider, and redirect path to their surface forms.

use cargo_cgp_error_processing::{WiringKey, is_ident, is_primitive_type};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, NIL_TYPE, PATH_CONS_TYPE, REDIRECT_LOOKUP_TYPE,
    WILDCARD_PARAM,
};
use crate::resolve::query::{decode_symbol, is_cgp_item, is_component_marker};

/// The surface form of a wiring key: a `DelegateComponent` key, or the `Self` of a namespace
/// lookup-trait entry: a bare component marker, an ordinary type, an `@`-path, or a blanket
/// forwarding tagged by the namespace/table trait that keys it. A component marker is told from an
/// ordinary type structurally, by whether a provider trait keys on it
/// ([`is_component_marker`]), so a per-type default's `String` reads as a type rather than as a
/// component nobody defined. `None` for a key the classifier cannot render (one still carrying a
/// generic parameter, an inference variable, or a placeholder), so the rewrite is declined rather
/// than guessed.
pub fn describe_key<'tcx>(tcx: TyCtxt<'tcx>, key: Ty<'tcx>, impl_did: DefId) -> Option<WiringKey> {
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
pub fn render_lookup_trait(tcx: TyCtxt<'_>, impl_did: DefId) -> String {
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

/// Render a `PathCons<..>` key back to its bare `@a.b` surface form. It is the typed counterpart of
/// the text `resugar_path`, done straight off the types so each generic parameter is read by name.
/// A lowercase `Symbol` segment decodes to its string; a named segment keeps its type name, and any
/// other type prints as itself, arguments included (`Wrap<u64>`); a generic segment, such as an
/// `open` key's per-entry generic or a `for`-loop key parameter, prints its parameter name
/// (`@ComputerComponent.Code.u64`). The `__Wildcard__` tail `cgp`'s macros end a key with is left
/// out, since the programmer never wrote it, so a key reads exactly as its entry wrote it. The bare
/// `@…` form (no `Path!(…)` wrapper) is what the rewritten conflict message uses. `None` on a spine
/// that is not `PathCons`/`Nil`.
pub fn render_path<'tcx>(tcx: TyCtxt<'tcx>, path: Ty<'tcx>) -> Option<String> {
    let mut segments: Vec<String> = Vec::new();
    let mut current = path;
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 256 {
            return None;
        }
        match current.kind() {
            ty::Param(param) => {
                if param.name.as_str() != WILDCARD_PARAM {
                    segments.push(param.name.to_string());
                }
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
                    ty::Adt(_, _) if let Some(symbol) = decode_symbol(tcx, head) => {
                        // Only a lowercase, non-primitive name is a symbol segment going forward,
                        // so any other reads as a type and has no path spelling.
                        if !is_symbol_segment(&symbol) {
                            return None;
                        }
                        segments.push(symbol);
                    }
                    // Any other segment prints as the type itself: a generic segment as its
                    // parameter name, a primitive such as the `u64` an input-keyed `open` entry
                    // names, a type with a qualifier only when it tells two types apart
                    // (`std::io::Error`), and one with arguments with all of them (`Wrap<u64>`),
                    // which is what tells two keys over one type apart. A plain lowercase name would
                    // read as a symbol, so it declines.
                    _ => {
                        let segment = tcx.erase_and_anonymize_regions(head).to_string();
                        if is_symbol_segment(&segment) {
                            return None;
                        }
                        segments.push(segment);
                    }
                }
                current = args.type_at(1);
            }
            _ => return None,
        }
    }
    Some(format!("@{}", segments.join(".")))
}

/// Whether `name` is a path segment `Path!` encodes as a symbol: a lowercase, non-primitive
/// identifier.
fn is_symbol_segment(name: &str) -> bool {
    is_ident(name) && name.starts_with(|c: char| c.is_ascii_lowercase()) && !is_primitive_type(name)
}

/// If `delegate` is a `RedirectLookup<Table, Path>`, render its `Path` (the second argument) to
/// its bare `@..` surface form: the redirected key the user should set. `None` for any other
/// `Delegate`, which is an ordinary provider, not a redirect.
pub fn redirect_path<'tcx>(tcx: TyCtxt<'tcx>, delegate: Ty<'tcx>) -> Option<String> {
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
/// `UseType<String>`. The post-processing chain that runs after strips its module qualifiers,
/// except where they tell two items apart.
pub fn render_provider<'tcx>(tcx: TyCtxt<'tcx>, delegate: Ty<'tcx>) -> String {
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

/// The bounding trait ref of a blanket `DelegateComponent<Key>` impl: the single non-`Sized` bound
/// on its generic key parameter (`Key: DefaultNamespace<Ctx>`), the namespace or `for`-loop table
/// the forwarding routes through. `None` if no such bound is found.
pub fn bounding_trait_ref<'tcx>(
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
