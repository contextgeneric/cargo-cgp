//! Reading CGP's type-level spines: paths, the `Nil` terminator, symbols, and indices.

use rustc_middle::ty::consts::ConstExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_FIELD_CRATE, CHARS_TYPE, INDEX_TYPE, NIL_TYPE, PATH_CONS_TYPE,
    SYMBOL_TYPE,
};
use crate::resolve::query::is_cgp_item;

/// Whether `ty` is CGP's type-level path spine `PathCons<…>`: an `open`/namespace redirect key,
/// as opposed to a bare component marker. Anchored by `DefId` to [`CGP_BASE_TYPES_CRATE`].
pub fn is_path_cons(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if is_cgp_item(tcx, def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE))
}

/// The `(head, tail)` of a `PathCons<Head, Tail>` type, or `None` when `ty` is not a `PathCons`.
pub fn path_cons_parts<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<(Ty<'tcx>, Ty<'tcx>)> {
    match ty.kind() {
        ty::Adt(def, args) if is_cgp_item(tcx, def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE) => {
            Some((args.type_at(0), args.type_at(1)))
        }
        _ => None,
    }
}

/// A redirect path with its trailing **unknown** segments removed, or `None` when it has none.
///
/// A `RedirectLookup` keys its table on the redirect path *plus the component's own parameters*, so
/// a `Handler` lookup for the fragment `Missing` reads `@…HandlerComponent.Missing.<Input>`. When
/// the failure was recovered from a call the code does not fully type (a
/// [call site](crate::resolve::resolve_call_site) whose input is inferred), that trailing parameter
/// is a placeholder, and reporting it would name an entry the programmer could not write, besides
/// being dropped as unknowable. Trimming the trailing run leaves the path they *can* wire.
///
/// Only a trailing run is trimmed: a placeholder further up is part of what the lookup keys on, so
/// such a leaf stays unknowable and is dropped as before.
pub fn trim_unknown_path_tail<'tcx>(tcx: TyCtxt<'tcx>, path: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let ty::Adt(cons_def, _) = path.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, cons_def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE) {
        return None;
    }

    let mut segments = Vec::new();
    let mut rest = path;
    while let Some((head, tail)) = path_cons_parts(tcx, rest) {
        segments.push(head);
        rest = tail;
    }
    if !is_nil(tcx, rest) {
        return None;
    }

    let kept = segments
        .iter()
        .rposition(|segment| !segment.has_placeholders())
        .map_or(0, |last| last + 1);
    if kept == segments.len() {
        return None;
    }

    Some(segments[..kept].iter().rev().fold(rest, |tail, &head| {
        Ty::new_adt(tcx, *cons_def, tcx.mk_args(&[head.into(), tail.into()]))
    }))
}

/// Whether `ty` is CGP's type-level path/list terminator `Nil`.
pub fn is_nil(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if is_cgp_item(tcx, def.did(), NIL_TYPE, CGP_BASE_TYPES_CRATE))
}

/// Decode a CGP `Symbol!` type into its string, by walking the `Chars<'c', Tail>` spine and
/// reading each `char` const argument until `Nil`. Anchored to `cgp_base_types`, and returns
/// `None` for any type that is not a well-formed `Symbol`, including one whose `LEN` is not the
/// decoded string's byte length, since `Symbol!` always writes `str::len()` there.
pub fn decode_symbol(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<String> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), SYMBOL_TYPE, CGP_BASE_TYPES_CRATE) {
        return None;
    }

    // `Symbol<const LEN, Chars>`: the first argument is the byte length, checked once the spine is
    // decoded, and the second is the head of the `Chars` spine.
    let len = args
        .const_at(0)
        .try_to_value()?
        .valtree
        .try_to_leaf()?
        .to_target_usize(tcx);
    let mut current = args.type_at(1);
    let mut name = String::new();
    loop {
        let ty::Adt(def, args) = current.kind() else {
            return None;
        };
        if is_cgp_item(tcx, def.did(), NIL_TYPE, CGP_BASE_TYPES_CRATE) {
            break;
        }
        if !is_cgp_item(tcx, def.did(), CHARS_TYPE, CGP_BASE_TYPES_CRATE) {
            return None;
        }

        // `Chars<const CHAR: char, Tail>`: read the char, then follow the tail.
        let scalar = args.const_at(0).try_to_value()?.valtree.try_to_leaf()?;
        name.push(char::from_u32(scalar.to_u32())?);
        current = args.type_at(1);
    }
    (u64::try_from(name.len()).ok()? == len).then_some(name)
}

/// Decode a CGP `Index<N>` type, the position tag of a tuple field, into its `N`. Anchored to
/// `cgp_field`, and returns `None` for any other type.
pub fn decode_index(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<usize> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), INDEX_TYPE, CGP_FIELD_CRATE) {
        return None;
    }

    // `Index<const I: usize>`: the only argument is the position.
    let index = args
        .const_at(0)
        .try_to_value()?
        .valtree
        .try_to_leaf()?
        .to_target_usize(tcx);
    usize::try_from(index).ok()
}
