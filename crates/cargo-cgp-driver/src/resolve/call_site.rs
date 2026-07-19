//! Recovering a use-site failure's obligation from the call expression's own HIR.
//!
//! This is the anchor for the use-site failure whose spans touch nothing the span-matching
//! anchors can read: the wiring matches the called component unconditionally, so the method is
//! *found* and the failure is an `E0277` whose spans never leave the call. Everything is
//! recovered from the failing call expression, HIR-only (never `tcx.typeck`, which cannot be
//! forced from the emitter): the *receiver* carries the context, the component's parameters come
//! from unifying the call's *written* argument types against the method's own declared signature
//! — no calling convention assumed — and each parameter the call leaves to inference is seeded as
//! a rigid placeholder the walk resolves around but never reports on. The failure shape, the
//! rationale for each recovery step, the worked example, and the decline boundaries are
//! documented in `docs/implementation/typed-root-cause-resolution.md` under "Recovering from the
//! call expression itself"; this module holds the mechanics.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::def::{CtorKind, CtorOf, DefKind, Res};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir, Expr, ExprKind, QPath};
use rustc_infer::infer::{InferCtxt, TyCtxtInferExt as _};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode, Upcast as _};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Span, Symbol};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::cgp_item::is_consumer_trait;
use crate::resolve::walk::{holds, resolve_leaves, unknowns_to_placeholders};

/// Resolve a use-site failure by re-reading the failing *call expression*: recover the context
/// from the receiver's binding, the component's parameters by unifying the call's written
/// argument types against the method's declared signature, and seed the walk with a rigid
/// placeholder for each parameter the call leaves to inference. `None` when no method call sits
/// at the diagnostic's spans, the receiver's type is not syntactically recoverable, or no
/// placeholder-free root cause is found.
///
/// Tried last: a failure any span-matching anchor can recover keeps its more precise recovery.
pub fn resolve_call_site(tcx: TyCtxt<'_>, spans: &[Span]) -> Option<Resolved> {
    for call in method_calls_at(tcx, spans) {
        let Some(context) = receiver_context(tcx, call.receiver) else {
            continue;
        };
        // Only a local, monomorphic ADT can be re-checked as a context: a foreign receiver is not
        // a CGP context of this crate, and a generic one's arguments are exactly what the missing
        // typeck results would have supplied.
        if !is_local_adt(context) || context.has_param() {
            continue;
        }
        for (consumer_did, method_did) in consumer_traits_with_method(tcx, call.method) {
            let Some(top) = seed_from_call(tcx, context, consumer_did, method_did, call.args)
            else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(resolved) = resolve_leaves(tcx, top) {
                return Some(resolved);
            }
        }
    }
    None
}

/// One method call found at the diagnostic's spans: the method name, the receiver expression,
/// and the call's argument expressions.
struct MethodCall<'tcx> {
    method: Symbol,
    receiver: &'tcx Expr<'tcx>,
    args: &'tcx [Expr<'tcx>],
}

/// Every method-call expression in a local body at one of the diagnostic's spans. A use-site
/// failure's spans sit on the method name, an argument, the whole call — or, for the re-report
/// rustc raises where the result is awaited, on the `.await` alone, whose desugared wrapper
/// expressions contain the call without the call's own span overlapping. So a method call is
/// collected when its own span overlaps a diagnostic span *or* it sits inside any expression
/// whose span does; each match is a candidate the caller tries (and gates on actually failing).
fn method_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<MethodCall<'tcx>> {
    let mut finder = CallFinder {
        spans,
        within_match: false,
        calls: Vec::new(),
    };
    for owner in tcx.hir_body_owners() {
        finder.visit_expr(tcx.hir_body_owned_by(owner).value);
    }
    finder.calls
}

struct CallFinder<'a, 'tcx> {
    spans: &'a [Span],
    /// Whether the current expression sits inside one whose span overlapped a diagnostic span.
    within_match: bool,
    calls: Vec<MethodCall<'tcx>>,
}

impl<'tcx> Visitor<'tcx> for CallFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        let matched = self.within_match || self.spans.iter().any(|span| span.overlaps(expr.span));
        if matched && let ExprKind::MethodCall(segment, receiver, args, _) = expr.kind {
            self.calls.push(MethodCall {
                method: segment.ident.name,
                receiver,
                args,
            });
        }
        let saved = std::mem::replace(&mut self.within_match, matched);
        intravisit::walk_expr(self, expr);
        self.within_match = saved;
    }
}

/// The context type a call's receiver expression names — the *receiver* is what carries the
/// context in a consumer-method call, so this is the anchor's one source of it. The type is read
/// syntactically: a path to a binding follows the binding (a `let` typed by its annotation or its
/// initializer, a fn parameter typed by the enclosing signature); a struct literal, unit-struct
/// value, const, or static names its type directly; a plain constructor call (`MyApp::new()`)
/// takes the callee's declared return type; references are peeled on the way. `None` for a
/// receiver whose type only typeck could know (a method call's result, a field access).
fn receiver_context<'tcx>(tcx: TyCtxt<'tcx>, expr: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    match expr.kind {
        ExprKind::Path(QPath::Resolved(None, path)) => match path.res {
            Res::Local(binding) => local_binding_context(tcx, binding),
            // A unit-struct value used directly (`MyCliApp.handle(…)`) resolves to the struct's
            // const constructor.
            Res::Def(DefKind::Ctor(CtorOf::Struct, CtorKind::Const), ctor_did) => {
                item_ty(tcx, tcx.parent(ctor_did))
            }
            Res::Def(DefKind::Const { .. } | DefKind::Static { .. }, did) => item_ty(tcx, did),
            _ => None,
        },
        ExprKind::Struct(qpath, ..) => {
            let QPath::Resolved(None, path) = qpath else {
                return None;
            };
            let Res::Def(DefKind::Struct, did) = path.res else {
                return None;
            };
            item_ty(tcx, did)
        }
        // A plain constructor call: the callee's declared return type is the receiver's type,
        // read from its (collection-cached) signature — still no typeck results.
        ExprKind::Call(callee, _) => Some(call_output_ty(tcx, callee)?.peel_refs()),
        ExprKind::AddrOf(_, _, inner) | ExprKind::Unary(hir::UnOp::Deref, inner) => {
            receiver_context(tcx, inner)
        }
        _ => None,
    }
}

/// The declared return type of a call to a non-generic fn named by path — the one expression
/// shape whose type a (collection-cached) signature supplies without typeck. `None` for a generic
/// callee (its instantiation is exactly what inference would have decided), a type-relative path
/// (`Vec::new`, whose resolution lives only in typeck results), or an output that still carries a
/// late-bound region (relating it would leak an escaping bound var).
fn call_output_ty<'tcx>(tcx: TyCtxt<'tcx>, callee: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    let ExprKind::Path(QPath::Resolved(None, path)) = callee.kind else {
        return None;
    };
    let Res::Def(DefKind::Fn | DefKind::AssocFn, did) = path.res else {
        return None;
    };
    if !tcx.generics_of(did).is_empty() {
        return None;
    }
    let output = tcx
        .fn_sig(did)
        .instantiate_identity()
        .skip_norm_wip()
        .skip_binder()
        .output();
    (!output.has_escaping_bound_vars()).then_some(output)
}

/// The type of the binding a receiver path resolves to. A `let` with a type annotation supplies
/// it directly; a `let` without one is typed by its initializer expression (a struct literal,
/// usually); a fn parameter is typed by the matching input of the enclosing signature. All three
/// are syntactic — no typeck results are consulted.
fn local_binding_context<'tcx>(tcx: TyCtxt<'tcx>, binding: hir::HirId) -> Option<Ty<'tcx>> {
    match tcx.parent_hir_node(binding) {
        hir::Node::LetStmt(let_stmt) => {
            if let Some(ty) = let_stmt.ty {
                return lower_hir_ty(tcx, peel_hir_refs(ty));
            }
            receiver_context(tcx, let_stmt.init?)
        }
        hir::Node::Param(param) => {
            let owner = tcx.hir_enclosing_body_owner(binding);
            let body = tcx.hir_body_owned_by(owner);
            let index = body
                .params
                .iter()
                .position(|candidate| candidate.pat.hir_id == param.pat.hir_id)?;
            let decl = tcx.hir_node_by_def_id(owner).fn_decl()?;
            lower_hir_ty(tcx, peel_hir_refs(decl.inputs.get(index)?))
        }
        _ => None,
    }
}

/// Strip the reference layers off a written type (`&App` → `App`), since the receiver's context
/// is the ADT beneath them.
fn peel_hir_refs<'tcx>(mut ty: &'tcx hir::Ty<'tcx>) -> &'tcx hir::Ty<'tcx> {
    while let hir::TyKind::Ref(_, mut_ty) = ty.kind {
        ty = mut_ty.ty;
    }
    ty
}

/// Build the seed obligation `Ctx: Consumer<…>` by unifying the call's arguments against the
/// method's own declared signature — no calling convention assumed; the signature's own use of
/// the trait's generics decides what each written argument type pins down (the rationale is in
/// the implementation document's call-site section). Every parameter of the method's item starts
/// as a fresh inference variable; `Self` is pinned to the recovered context; each argument whose
/// type the call *writes* syntactically ([`expr_written_ty`]) is unified with its declared input;
/// and each parameter left unconstrained is folded into a rigid
/// [placeholder](Ty::new_placeholder) the walk treats as unknown
/// ([`unknowns_to_placeholders`]). `None` when the consumer carries a const parameter, which this
/// recovery cannot supply.
fn seed_from_call<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: Ty<'tcx>,
    consumer_did: DefId,
    method_did: DefId,
    args: &[Expr<'tcx>],
) -> Option<ty::PolyTraitPredicate<'tcx>> {
    let param_env = ty::ParamEnv::empty();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);

    // Fresh variables for the method's whole item — for an associated fn the parent trait's
    // parameters come first, `Self` at index 0 — then pin `Self` to the recovered context.
    let method_args = infcx.fresh_args_for_item(DUMMY_SP, method_did);
    ocx.eq(
        &ObligationCause::dummy(),
        param_env,
        method_args.type_at(0),
        context,
    )
    .ok()?;

    // The declared inputs, their late-bound lifetimes instantiated as placeholders first —
    // relating a type with an escaping bound var panics the generalizer, as in the walk.
    let sig = tcx
        .fn_sig(method_did)
        .instantiate(tcx, method_args)
        .skip_norm_wip();
    let sig = infcx.enter_forall_and_leak_universe(sig);
    // `inputs()[0]` is the `self` receiver ([`consumer_traits_with_method`] requires one), already
    // pinned above, so the call's arguments line up with the declared inputs from position 1.
    for (arg, declared) in std::iter::zip(args, sig.inputs().iter().skip(1)) {
        if let Some(written) = expr_written_ty(&infcx, arg) {
            // Best effort: a written type that does not unify (a mis-guessed consumer candidate,
            // a coerced argument) just leaves its parameter unknown — the seed is gated on
            // failing, and the walk reports nothing it cannot prove.
            let _ = ocx.eq(&ObligationCause::dummy(), param_env, *declared, written);
        }
    }
    // Propagate what the unifications imply before the variables are read back.
    let _ = ocx.try_evaluate_obligations();

    // Read the trait's parameters back out of the method args, each unresolved remainder folded
    // into a rigid placeholder so the seed can cross into the walk's fresh inference contexts.
    let mut seed_args: Vec<ty::GenericArg<'tcx>> = vec![context.into()];
    for param in &tcx.generics_of(consumer_did).own_params[1..] {
        match param.kind {
            ty::GenericParamDefKind::Type { .. } => {
                let var = method_args.type_at(param.index as usize);
                let resolved = infcx.resolve_vars_if_possible(var);
                seed_args.push(unknowns_to_placeholders(tcx, resolved).into());
            }
            // A lifetime parameter is erased everywhere in the walk; supply it erased here too.
            ty::GenericParamDefKind::Lifetime => seed_args.push(tcx.lifetimes.re_erased.into()),
            ty::GenericParamDefKind::Const { .. } => return None,
        }
    }
    let trait_ref = ty::TraitRef::new(tcx, consumer_did, seed_args);
    let seed: ty::PolyTraitPredicate<'tcx> = ty::Binder::dummy(trait_ref).upcast(tcx);
    // Erase the region variables the fresh args and written references minted, so nothing of this
    // inference context leaks into the walk's.
    Some(tcx.erase_and_anonymize_regions(seed))
}

/// The type an argument expression *writes*, syntactically — the call-side information the
/// signature unification consumes. Covered shapes: a unit-struct or unit-variant value path with
/// its written arguments (`PhantomData::<Program>`, `GetMethod`), a non-generic const, a struct
/// literal, a reference, a tuple (its *structure* recovered even when some elements are not
/// written), a literal whose type is definite (`"…"`, suffixed numerics, `true`, `'c'`), and a call
/// to a non-generic fn (its declared return type). `None` for anything whose type only inference
/// could know — an unsuffixed literal, a variable, a generic constructor like `Vec::new()` —
/// leaving the corresponding parameter unknown rather than guessed.
fn expr_written_ty<'tcx>(infcx: &InferCtxt<'tcx>, expr: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    let tcx = infcx.tcx;
    match expr.kind {
        ExprKind::Path(QPath::Resolved(None, path)) => match path.res {
            // A unit-struct/unit-variant value: its type is the ADT, with whatever arguments the
            // path writes (defaults filled in; a generic ADT with none written stays unknown).
            Res::Def(DefKind::Ctor(ctor_of, CtorKind::Const), ctor_did) => {
                let adt_did = match ctor_of {
                    CtorOf::Struct => tcx.parent(ctor_did),
                    CtorOf::Variant => tcx.parent(tcx.parent(ctor_did)),
                };
                instantiate_written(tcx, adt_did, written_type_args(tcx, path)?)
            }
            Res::Def(DefKind::Const { .. }, did) if tcx.generics_of(did).is_empty() => {
                item_ty(tcx, did)
            }
            _ => None,
        },
        ExprKind::Struct(qpath, ..) => {
            let QPath::Resolved(None, path) = qpath else {
                return None;
            };
            let Res::Def(DefKind::Struct, did) = path.res else {
                return None;
            };
            instantiate_written(tcx, did, written_type_args(tcx, path)?)
        }
        ExprKind::AddrOf(_, mutbl, inner) => Some(Ty::new_ref(
            tcx,
            tcx.lifetimes.re_erased,
            expr_written_ty(infcx, inner)?,
            mutbl,
        )),
        // A tuple literal writes its *shape*, whether or not every element's type is written. An
        // element the call does not type becomes a fresh inference variable (folded into a
        // placeholder with the rest of the seed by [`unknowns_to_placeholders`]), so the tuple
        // arity and its written elements are recovered even beside an unknown one. This matters
        // because providers destructure their input on the tuple shape — `HandleIf`'s
        // `(InputCond, InputBranch)`, `HandleCompare`'s `(InputA, InputB)` — so collapsing the whole
        // tuple to one flat unknown (as returning `None` would) leaves such a provider's impl
        // unmatched and hides a cause sitting inside a *known* branch (a field read by the condition,
        // say). The recovered structure is real call-side information, not a guess: the leaves it
        // cannot type stay unknown and are never reported.
        ExprKind::Tup(elems) => {
            let tys: Vec<Ty<'tcx>> = elems
                .iter()
                .map(|elem| {
                    expr_written_ty(infcx, elem).unwrap_or_else(|| infcx.next_ty_var(DUMMY_SP))
                })
                .collect();
            Some(Ty::new_tup(tcx, &tys))
        }
        ExprKind::Lit(lit) => lit_ty(tcx, &lit),
        ExprKind::Call(callee, _) => call_output_ty(tcx, callee),
        _ => None,
    }
}

/// The definite type of a literal — `None` for the suffixless numerics whose type only inference
/// decides.
fn lit_ty<'tcx>(tcx: TyCtxt<'tcx>, lit: &hir::Lit) -> Option<Ty<'tcx>> {
    use rustc_ast::{LitFloatType, LitIntType, LitKind};
    match lit.node {
        LitKind::Str(..) => Some(Ty::new_imm_ref(
            tcx,
            tcx.lifetimes.re_erased,
            tcx.types.str_,
        )),
        LitKind::Bool(_) => Some(tcx.types.bool),
        LitKind::Char(_) => Some(tcx.types.char),
        LitKind::Byte(_) => Some(tcx.types.u8),
        LitKind::Int(_, LitIntType::Signed(int)) => Some(Ty::new_int(tcx, int)),
        LitKind::Int(_, LitIntType::Unsigned(uint)) => Some(Ty::new_uint(tcx, uint)),
        LitKind::Float(_, LitFloatType::Suffixed(float)) => Some(Ty::new_float(tcx, float)),
        _ => None,
    }
}

/// Lower a *written* type to its `ty::Ty`, syntactically — a path to an ADT or alias (with
/// written arguments, defaulted parameters filled in, lifetimes erased), a primitive, a tuple, a
/// reference, or a slice. This is deliberately not the compiler's HIR lowering: it runs inside
/// the emitter, where only already-cached queries may be forced, so it reads `type_of` for the
/// named item (cached — typeck resolved this very type to produce the diagnostic) and composes
/// the rest by hand. `None` for any shape beyond it, declining the anchor rather than guessing.
fn lower_hir_ty<'tcx>(tcx: TyCtxt<'tcx>, hir_ty: &hir::Ty<'tcx>) -> Option<Ty<'tcx>> {
    match hir_ty.kind {
        hir::TyKind::Path(QPath::Resolved(None, path)) => {
            let written = written_type_args(tcx, path)?;
            match path.res {
                Res::PrimTy(prim) if written.is_empty() => Some(prim_ty(tcx, prim)),
                Res::Def(
                    DefKind::Struct | DefKind::Enum | DefKind::Union | DefKind::TyAlias,
                    did,
                ) => instantiate_written(tcx, did, written),
                _ => None,
            }
        }
        hir::TyKind::Tup(tys) => {
            let elems: Vec<Ty<'tcx>> = tys
                .iter()
                .map(|ty| lower_hir_ty(tcx, ty))
                .collect::<Option<_>>()?;
            Some(Ty::new_tup(tcx, &elems))
        }
        hir::TyKind::Ref(_, mut_ty) => Some(Ty::new_ref(
            tcx,
            tcx.lifetimes.re_erased,
            lower_hir_ty(tcx, mut_ty.ty)?,
            mut_ty.mutbl,
        )),
        hir::TyKind::Slice(ty) => Some(Ty::new_slice(tcx, lower_hir_ty(tcx, ty)?)),
        _ => None,
    }
}

/// The written type arguments of a path's last segment, each lowered; lifetimes are skipped
/// (they are re-supplied erased) and a const or inferred argument declines. An argument-less
/// path yields the empty list.
fn written_type_args<'tcx>(tcx: TyCtxt<'tcx>, path: &hir::Path<'tcx>) -> Option<Vec<Ty<'tcx>>> {
    let Some(args) = path.segments.last()?.args else {
        return Some(Vec::new());
    };
    args.args
        .iter()
        .filter(|arg| !matches!(arg, hir::GenericArg::Lifetime(_)))
        .map(|arg| match arg {
            hir::GenericArg::Type(ty) => lower_hir_ty(tcx, ty.as_unambig_ty()),
            _ => None,
        })
        .collect()
}

/// Instantiate an ADT or type alias with its written type arguments: lifetimes are erased,
/// missing trailing parameters take their declared defaults, and a const parameter or an arity
/// mismatch declines.
fn instantiate_written<'tcx>(
    tcx: TyCtxt<'tcx>,
    did: DefId,
    written: Vec<Ty<'tcx>>,
) -> Option<Ty<'tcx>> {
    if tcx.generics_of(did).parent.is_some() {
        return None;
    }
    let mut written = written.into_iter();
    let mut lowered = true;
    let args = ty::GenericArgs::for_item(tcx, did, |param, args_so_far| match param.kind {
        ty::GenericParamDefKind::Lifetime => tcx.lifetimes.re_erased.into(),
        ty::GenericParamDefKind::Type { .. } => {
            if let Some(ty) = written.next() {
                ty.into()
            } else if let Some(default) = param.default_value(tcx) {
                default.instantiate(tcx, args_so_far).skip_norm_wip()
            } else {
                lowered = false;
                tcx.types.unit.into()
            }
        }
        ty::GenericParamDefKind::Const { .. } => {
            lowered = false;
            tcx.types.unit.into()
        }
    });
    if !lowered || written.next().is_some() {
        return None;
    }
    if tcx.def_kind(did) == DefKind::TyAlias {
        return Some(tcx.type_of(did).instantiate(tcx, args).skip_norm_wip());
    }
    Some(Ty::new_adt(tcx, tcx.adt_def(did), args))
}

/// The written primitive as its `ty::Ty`.
fn prim_ty<'tcx>(tcx: TyCtxt<'tcx>, prim: hir::PrimTy) -> Ty<'tcx> {
    match prim {
        hir::PrimTy::Bool => tcx.types.bool,
        hir::PrimTy::Char => tcx.types.char,
        hir::PrimTy::Str => tcx.types.str_,
        hir::PrimTy::Int(int) => Ty::new_int(tcx, int),
        hir::PrimTy::Uint(uint) => Ty::new_uint(tcx, uint),
        hir::PrimTy::Float(float) => Ty::new_float(tcx, float),
    }
}

/// The identity type of an item (`type_of`), for the receivers whose type is an item's own — a
/// unit struct's, a const's, or a static's.
fn item_ty<'tcx>(tcx: TyCtxt<'tcx>, did: DefId) -> Option<Ty<'tcx>> {
    Some(tcx.type_of(did).instantiate_identity().skip_norm_wip())
}

/// Every CGP consumer trait (recognized structurally, cross-crate) declaring a `self`-receiver
/// associated fn named `method` — the candidates a method call by that name can resolve through —
/// paired with that method's `DefId`, whose signature [`tag_bindings`] reads.
fn consumer_traits_with_method(tcx: TyCtxt<'_>, method: Symbol) -> Vec<(DefId, DefId)> {
    tcx.all_traits_including_private()
        .filter_map(|trait_did| {
            let method_did = tcx
                .associated_items(trait_did)
                .filter_by_name_unhygienic(method)
                .find(|item| matches!(item.kind, ty::AssocKind::Fn { has_self: true, .. }))?
                .def_id;
            Some((trait_did, method_did))
        })
        .filter(|&(trait_did, _)| is_consumer_trait(tcx, trait_did))
        .collect()
}

/// Whether `ty` is a struct or enum defined in the crate being compiled — the only kind of type
/// the resolver re-checks as a context.
fn is_local_adt(ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if def.did().is_local())
}
