//! Turning a terminal predicate into the rustc-free [`Leaf`] the emitter words.

use cargo_cgp_error_processing::{FieldFix, FieldIssue, Leaf, NearMiss};
use rustc_middle::ty::print::PrintTraitRefExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, DELEGATE_COMPONENT_TRAIT, HAS_FIELD_TRAIT,
};
use crate::resolve::cgp_item::{
    abstract_type_component_marker, decode_symbol, is_cgp_item, is_namespace_lookup_trait,
    is_path_cons,
};
use crate::resolve::classify::{
    classify_data_leaf, field_issue, field_type, is_blanket_dead_end, is_dispatch_lookup,
    owner_has_impl_of, projected_type, provider_for_other_arguments, required_field_type,
    similar_field, similar_wired_key, tuple_index_tag,
};
use crate::resolve::conflict::render_path;
use crate::resolve::label::fold_empty_lists;
use crate::resolve::walk::ProjectionMismatch;

/// Classify the terminal predicate a dependency chain bottoms out on. A `HasField` whose branch
/// carried an unmet projection (`mismatch` is `Some`) becomes a [`Leaf::FieldTypeMismatch`], its
/// actual field type queried from the struct; a plain `HasField` becomes a [`Leaf::Field`]
/// (inspecting the struct so the emitter can tell missing from underived); a branch whose unmet
/// projection is on any *other* associated type becomes a [`Leaf::AssocTypeMismatch`]; an unmet
/// `DelegateComponent<Marker>` — a component the context does not wire — becomes a
/// [`Leaf::MissingWiring`] naming that component marker (or, for a redirect path, a
/// [`Leaf::MissingRedirectWiring`] on the context and a [`Leaf::MissingDispatchEntry`] on an
/// aggregate provider); an unmet namespace lookup
/// (`Path: DefaultNamespace<Table>` or a user `cgp_namespace!` trait) — a `RedirectLookup` whose
/// path the table does not terminate — becomes a [`Leaf::MissingRedirectWiring`] naming the path
/// when the table is the context, and a [`Leaf::MissingDispatchEntry`] when it is an aggregate
/// provider; any other bound becomes a [`Leaf::Bound`] restating it as `self: Trait`.
pub(crate) fn classify_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    mismatch: Option<ProjectionMismatch<'tcx>>,
) -> Leaf {
    // An extensible-data operation's bound names the field or variant whose state its partial value
    // records, read off the presence markers before the generic shapes below claim it.
    if let Some(leaf) = classify_data_leaf(tcx, leaf_ref, parent, mismatch) {
        return leaf;
    }
    // A projection mismatch on a trait that is *not* `HasField` — most often a CGP abstract type the
    // context binds one way and a provider pins another. Classified before the trait-keyed branches
    // below, since the leaf is about the projected type rather than the trait bound (which holds).
    if let Some(mismatch) = mismatch
        && !is_cgp_item(tcx, leaf_ref.def_id, HAS_FIELD_TRAIT, CGP_FIELD_CRATE)
    {
        return assoc_type_mismatch(tcx, mismatch);
    }
    if is_cgp_item(
        tcx,
        leaf_ref.def_id,
        DELEGATE_COMPONENT_TRAIT,
        CGP_COMPONENT_CRATE,
    ) {
        let key = leaf_ref.args.type_at(1);
        let self_ty = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        let owner = self_ty.to_string();
        // A `DelegateComponent<PathCons<…>>` key is a redirect *path* an `open` statement or a
        // namespace routed the lookup along, not a bare component marker, and the table it was
        // looked up in has no entry terminating it. Rendering only its ADT item name would flatten
        // the whole path to a useless `PathCons`, so the leaf names the full path (its `PathCons`
        // spine resugars to `@…` when the note is post-processed).
        if is_path_cons(tcx, key) {
            // On a *non-context* table the path is an aggregate provider's own `open` redirect, so
            // the leaf names that provider table as any other entry missing from it does. On the
            // context it is a [`Leaf::MissingRedirectWiring`], parallel to the namespace-lookup
            // leaf below.
            if self_ty != context {
                return Leaf::MissingDispatchEntry {
                    key: tcx.erase_and_anonymize_regions(key).to_string(),
                    table: owner,
                };
            }
            return Leaf::MissingRedirectWiring {
                path: tcx.erase_and_anonymize_regions(key).to_string(),
                similar: wired_near_miss(tcx, self_ty, render_path(tcx, key)),
                context: owner,
            };
        }
        if self_ty != context {
            // A provider reached through its own delegation blanket, although it implements the
            // parent's trait, implements that trait for other arguments than the wiring gives it.
            if let Some(parent) = parent
                && is_blanket_dead_end(tcx, self_ty, parent)
                && let Some(leaf) = provider_for_other_arguments(tcx, parent)
            {
                return leaf;
            }
            // A `DelegateComponent<Key>` on a *non-context* type splits two ways (both let through
            // by `is_reportable_leaf`). If the owner is a delegation table — a separate-table
            // dispatch lookup ([`is_dispatch_lookup`]) or an owner that wires some other key
            // ([`owner_has_impl_of`] for `DelegateComponent`) — it is an aggregate provider or a
            // `UseDelegate`/`UseInputDelegate` table missing this entry: a [`Leaf::MissingDispatchEntry`]
            // naming the table and the key (named in full, since it may be a dispatched-on type).
            let is_dispatch = parent.is_some_and(|p| is_dispatch_lookup(tcx, self_ty, p.self_ty()))
                || owner_has_impl_of(tcx, leaf_ref.def_id, self_ty);
            if is_dispatch {
                return Leaf::MissingDispatchEntry {
                    key: tcx.erase_and_anonymize_regions(key).to_string(),
                    table: owner,
                };
            }
            // Otherwise the owner is not a table at all: a type wired where a provider was expected
            // that does not implement the provider trait. Name the provider trait from the parent
            // obligation `owner: ProviderTrait<Ctx>` whose blanket produced this leaf.
            if let Some(parent) = parent {
                return Leaf::NotAProvider {
                    provider: owner,
                    provider_trait: tcx.item_name(parent.def_id).to_string(),
                };
            }
            // No parent trait to name (a root-level `DelegateComponent`); fall back to the
            // dispatch-entry wording rather than invent a trait name.
            return Leaf::MissingDispatchEntry {
                key: tcx.erase_and_anonymize_regions(key).to_string(),
                table: owner,
            };
        }
        // A bare `DelegateComponent<Marker>` on the context with no satisfying impl: the context does
        // not wire the component at all. The marker's own item name (`BarProviderComponent`) is what
        // the programmer writes to fix it, so it names the leaf.
        let component = component_marker_name(tcx, key);
        return Leaf::MissingWiring {
            similar: wired_near_miss(tcx, self_ty, Some(component.clone())),
            component,
            owner,
        };
    }
    if is_namespace_lookup_trait(tcx, leaf_ref.def_id) {
        // A namespace lookup trait (`DefaultNamespace`, a user `cgp_namespace!` trait, …) unmet at
        // the terminal: a `RedirectLookup` forwarded the lookup to this path inside a table's
        // wiring, but nothing terminates it. The `Self` type is the redirect path (its `PathCons`
        // spine resugars to `Path!(@…)` when the note is post-processed) and the trait's last type
        // argument is the table that carries no entry for it.
        let path = tcx
            .erase_and_anonymize_regions(leaf_ref.self_ty())
            .to_string();
        let table = leaf_ref
            .args
            .types()
            .last()
            .map(|table| tcx.erase_and_anonymize_regions(table));
        // A table other than the context is an aggregate provider that joins the namespace itself,
        // so the leaf names that provider table, as for a path missing from its own `open` table.
        if let Some(table) = table
            && table != context
        {
            return Leaf::MissingDispatchEntry {
                key: path,
                table: table.to_string(),
            };
        }
        let similar = wired_near_miss(
            tcx,
            table.unwrap_or(context),
            render_path(tcx, leaf_ref.self_ty()),
        );
        let context = table
            .map(|table| table.to_string())
            .unwrap_or_else(|| path.clone());
        return Leaf::MissingRedirectWiring {
            path,
            context,
            similar,
        };
    }
    if is_cgp_item(tcx, leaf_ref.def_id, HAS_FIELD_TRAIT, CGP_FIELD_CRATE)
        && let Some(name) = decode_symbol(tcx, leaf_ref.args.type_at(1))
    {
        let owner = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        if let Some(mismatch) = mismatch {
            let expected = mismatch.expected.to_string();

            // A required type read off the projection may itself project through the context's own
            // wiring — `Pool<<App as HasDbType>::Db>` when a provider reads a field whose type is
            // expressed through an abstract type it imports. Keep that form, since it names where
            // the requirement comes from, and carry what it reduces to alongside; a requirement
            // that is already concrete normalizes to itself and gets nothing extra.
            let expected_normalized = projected_type(tcx, mismatch.expected)
                .map(|ty| ty.to_string())
                .filter(|normalized| *normalized != expected);

            return Leaf::FieldTypeMismatch {
                actual: field_type(tcx, owner, &name).unwrap_or_else(|| "_".to_owned()),
                name,
                owner: owner.to_string(),
                expected,
                expected_normalized,
            };
        }
        let issue = field_issue(tcx, owner, &name);
        // Adding a missing field fixes the cause only when the struct has named fields, which a
        // tuple struct cannot gain one of, and derives `HasField`, which it shows by implementing
        // it for another field; the fix then names the type the requirement gives the field, and
        // a similar field the struct carries, a likely typo.
        let fix = match issue {
            FieldIssue::Missing => FieldFix {
                confirmed: has_named_fields(owner)
                    && owner_has_impl_of(tcx, leaf_ref.def_id, owner),
                ty: parent.and_then(|parent| required_field_type(tcx, parent, leaf_ref)),
                similar: similar_field(owner, &name),
                index_tag: tuple_index_tag(owner, &name),
            },
            _ => FieldFix::default(),
        };
        return Leaf::Field {
            name,
            owner: owner.to_string(),
            issue,
            fix,
        };
    }
    // An empty list in the bound reads as the `Product![]` the program wrote.
    let leaf_ref = fold_empty_lists(tcx, leaf_ref);
    Leaf::Bound {
        summary: format!(
            "{}: {}",
            leaf_ref.self_ty(),
            leaf_ref.print_only_trait_path()
        ),
    }
}

/// Build the [`Leaf::AssocTypeMismatch`] for a projection mismatch on a trait other than `HasField`:
/// the associated type and its trait named off the projection, the *expected* type read from the
/// failing projection's right-hand side, and the *actual* one read by normalizing the projection —
/// the same query for a `UseType<T>` wiring and a hand-written impl alike. When the trait is a CGP
/// abstract-type component, its wiring marker rides along so the emitter can offer the `UseType<…>`
/// fix and call the type an *abstract* rather than a plain associated type. An actual type that does
/// not reduce is rendered `_`, as the field query's does.
fn assoc_type_mismatch<'tcx>(tcx: TyCtxt<'tcx>, mismatch: ProjectionMismatch<'tcx>) -> Leaf {
    let trait_did = mismatch.trait_ref.def_id;
    let owner = tcx.erase_and_anonymize_regions(mismatch.trait_ref.self_ty());
    let actual = projected_type(tcx, mismatch.alias)
        .map(|ty| ty.to_string())
        .unwrap_or_else(|| "_".to_owned());
    let expected = mismatch.expected.to_string();

    // A required type read off the projection may itself project through another abstract type —
    // the unification pin form produces exactly that — so carry what it reduces to alongside the
    // form the provider wrote, on the same rule as the field leaf.
    let expected_normalized = projected_type(tcx, mismatch.expected)
        .map(|ty| ty.to_string())
        .filter(|normalized| *normalized != expected);

    Leaf::AssocTypeMismatch {
        assoc: tcx.item_name(mismatch.assoc_did).to_string(),
        trait_name: tcx.item_name(trait_did).to_string(),
        owner: owner.to_string(),
        expected,
        expected_normalized,
        actual,
        component: abstract_type_component_marker(tcx, trait_did)
            .map(|marker| component_marker_name(tcx, marker)),
    }
}

/// The plain item name of a component marker type — `BarProviderComponent` for the
/// `DelegateComponent<BarProviderComponent>` key — which is the identifier a programmer writes on
/// the left of a `delegate_components!` entry. Falls back to the marker's printed form when it is
/// not an ADT (which a real component marker always is).
fn component_marker_name<'tcx>(tcx: TyCtxt<'tcx>, marker: Ty<'tcx>) -> String {
    match marker.kind() {
        ty::Adt(def, _) => tcx.item_name(def.did()).to_string(),
        _ => marker.to_string(),
    }
}

/// The near miss for a key missing from `context`'s wiring: a key the context wires that is a likely
/// slip of `target`, the missing key rendered as an entry writes it (`None` when it has no such
/// rendering).
fn wired_near_miss<'tcx>(tcx: TyCtxt<'tcx>, context: Ty<'tcx>, target: Option<String>) -> NearMiss {
    NearMiss(target.and_then(|target| similar_wired_key(tcx, context, &target)))
}

/// Whether `ty` is a struct whose fields are named, the only shape a named field can be added to.
fn has_named_fields(ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(adt, _) if adt.is_struct() && adt.non_enum_variant().ctor_kind().is_none())
}
