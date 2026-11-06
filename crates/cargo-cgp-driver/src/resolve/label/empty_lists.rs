//! Writing CGP's empty lists as the `Product![]` and `Sum![]` the programmer wrote.

use rustc_middle::ty::{self, Ty, TyCtxt, TypeFoldable, TypeFolder, TypeSuperFoldable};
use rustc_span::Symbol;

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_FIELD_CRATE, CHARS_TYPE, CONS_TYPE, EITHER_TYPE, FIELD_TYPE,
    NIL_TYPE, PATH_CONS_TYPE, SYMBOL_TYPE, VOID_TYPE,
};
use crate::resolve::query::is_cgp_item;

/// `value` with every CGP empty list that stands on its own (a `PipeHandlers<Nil>` argument, a
/// `Nil: ComposeProviders` bound's subject) replaced by a type that prints as `Product![]`, and
/// every empty sum by one that prints as `Sum![]`, so a label or a bound reads as the program
/// wrote it.
///
/// The `Nil` and `Void` are recognized by `DefId`, so a same-named type from another crate is left
/// alone. Only a standalone one is replaced: the `Nil` that ends a `Cons` or `PathCons` spine, or a
/// `Void` that ends an `Either` spine, is the spine's terminator, which the text resugaring folds
/// with the rest of the spine; and a `Symbol` or `Field` is left whole, since a `Symbol`'s `Nil`
/// ends a string and a `Field`'s `Nil` payload is a unit variant to the `Enum!` folding. The
/// replacement is a type parameter named for the macro, which no other code sees: it exists only to
/// be printed.
pub fn fold_empty_lists<'tcx, T: TypeFoldable<TyCtxt<'tcx>>>(tcx: TyCtxt<'tcx>, value: T) -> T {
    value.fold_with(&mut EmptyLists { tcx })
}

struct EmptyLists<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> EmptyLists<'tcx> {
    fn is(&self, ty: Ty<'tcx>, name: &str, krate: &str) -> bool {
        matches!(ty.kind(), ty::Adt(def, _) if is_cgp_item(self.tcx, def.did(), name, krate))
    }

    /// A spine's tail: its terminator is kept, a further cell folds its head in turn, and anything
    /// else folds as an ordinary type.
    fn fold_tail(&mut self, tail: Ty<'tcx>, terminator: &str, krate: &str) -> Ty<'tcx> {
        if self.is(tail, terminator, krate) {
            tail
        } else {
            tail.fold_with(self)
        }
    }

    /// A type that prints as `text`.
    fn marker(&self, text: &str) -> Ty<'tcx> {
        Ty::new_param(self.tcx, u32::MAX, Symbol::intern(text))
    }
}

impl<'tcx> TypeFolder<TyCtxt<'tcx>> for EmptyLists<'tcx> {
    fn cx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        let ty::Adt(def, args) = ty.kind() else {
            return ty.super_fold_with(self);
        };
        let did = def.did();
        let base = |name| is_cgp_item(self.tcx, did, name, CGP_BASE_TYPES_CRATE);
        let field = |name| is_cgp_item(self.tcx, did, name, CGP_FIELD_CRATE);
        if base(NIL_TYPE) {
            return self.marker("Product![]");
        }
        if field(VOID_TYPE) {
            return self.marker("Sum![]");
        }
        // A `Symbol` and its `Chars` spine (whose first argument is a `char` const) are a string,
        // and a `Field`'s payload is read by the `Enum!` folding, so all three are left whole.
        if base(SYMBOL_TYPE) || base(CHARS_TYPE) || field(FIELD_TYPE) {
            return ty;
        }
        let (terminator, krate) = if base(CONS_TYPE) || base(PATH_CONS_TYPE) {
            (NIL_TYPE, CGP_BASE_TYPES_CRATE)
        } else if field(EITHER_TYPE) {
            (VOID_TYPE, CGP_FIELD_CRATE)
        } else {
            return ty.super_fold_with(self);
        };
        // A spine cell `Cell<Head, Tail>`: the head is an element, the tail continues the spine.
        let head = args.type_at(0).fold_with(self);
        let tail = self.fold_tail(args.type_at(1), terminator, krate);
        let mut new_args: Vec<ty::GenericArg<'tcx>> = args.iter().collect();
        new_args[0] = head.into();
        new_args[1] = tail.into();
        Ty::new_adt(self.tcx, *def, self.tcx.mk_args(&new_args))
    }
}
