//! Rendering a type to its dependency-tree form, resugaring CGP's type-level spines.

use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_FIELD_CRATE, CONS_TYPE, EITHER_TYPE, FIELD_TYPE, NIL_TYPE, VOID_TYPE,
};
use crate::resolve::cgp_item::{decode_symbol, is_cgp_item};

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
    // The call-site anchor's stand-in for a parameter the call leaves to inference: render it as
    // the `_` the programmer would write, never rustc's internal placeholder form.
    if let ty::Placeholder(_) = ty.kind() {
        return "_".to_owned();
    }
    // A tuple can carry such placeholders in its elements — the call-site anchor recovers a written
    // tuple's *shape* while leaving an unwritten element a placeholder — so render it recursively
    // rather than through `to_string`, which would print the raw `!N` placeholder form. This is the
    // one non-spine structural type the anchor puts placeholders inside (a reference or ADT argument
    // stays all-or-nothing, so it never contains one).
    if let ty::Tuple(elems) = ty.kind() {
        let parts: Vec<String> = elems.iter().map(|elem| render_ty(tcx, elem)).collect();
        return match parts.as_slice() {
            [] => "()".to_owned(),
            [one] => format!("({one},)"),
            _ => format!("({})", parts.join(", ")),
        };
    }
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
