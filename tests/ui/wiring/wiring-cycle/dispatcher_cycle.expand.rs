#![feature(prelude_import)]
//! A dispatcher wired as the entry for the type it dispatches, so its variant handlers loop back.
//!
//! `MatchWithValueHandlers` evaluates an `Expr` by handing each variant's payload back to the
//! context through its inner `UseContext`, and `EvalAdd` evaluates a `Plus<Expr>` by asking the
//! context to evaluate each operand, an `Expr` again. Wired as `Interp`'s entry for `Expr`, the
//! dispatcher makes that recursion part of the trait resolution, so checking either key overflows
//! (`E0275`). The fix is a wrapper provider for `Expr` whose impl has no `where` clause and calls
//! the dispatcher in its body, as the expression interpreter example does.
//!
//! Both checks are `[CGP-E020]`. The `Expr` check's help names `EvalAdd`, the entry whose
//! requirement closes its loop. The `Plus<Expr>` check's loop closes at the dispatcher's inner
//! `UseContext`, which no entry names, so its help names no provider. The output stays `usability`:
//! neither help points at the `Expr` entry, the one to change.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::dispatch::MatchWithValueHandlers;
use cgp::extra::handler::{CanCompute, ComputerComponent};
use cgp::prelude::*;
pub struct Eval;
pub struct Plus<Expr> {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}
pub struct Literal(pub u64);
pub enum Expr {
    Plus(Plus<Expr>),
    Literal(Literal),
}
impl HasFields for Expr {
    type Fields = Enum! { Plus(Plus<Expr>), Literal(Literal) };
}
impl HasFieldsRef for Expr {
    type FieldsRef<'__a> = Enum! { Plus(&'__a Plus<Expr>), Literal(&'__a Literal) }
    where
        Self: '__a;
}
impl FromFields for Expr {
    fn from_fields(rest: <Self as HasFields>::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Plus(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Literal(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Expr {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Plus(field) => Either::Left(field.into()),
            Self::Literal(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for Expr {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Plus(field) => Either::Left(field.into()),
            Self::Literal(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Plus")> for Expr {
    type Value = Plus<Expr>;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Plus")>,
        value: <Self as FromVariant<Symbol!("Plus")>>::Value,
    ) -> Self {
        Self::Plus(value)
    }
}
impl FromVariant<Symbol!("Literal")> for Expr {
    type Value = Literal;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Literal")>,
        value: <Self as FromVariant<Symbol!("Literal")>>::Value,
    ) -> Self {
        Self::Literal(value)
    }
}
pub enum __PartialExpr<__F0__: MapType, __F1__: MapType> {
    Plus(<__F0__ as MapType>::Map<Plus<Expr>>),
    Literal(<__F1__ as MapType>::Map<Literal>),
}
pub enum __PartialRefExpr<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Plus(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Plus<Expr>>>),
    Literal(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Literal>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialExpr<__F0__, __F1__> {
    type Target = Expr;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefExpr<'__a__, __R__, __F0__, __F1__> {
    type Target = Expr;
}
impl HasExtractor for Expr {
    type Extractor = __PartialExpr<IsPresent, IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Plus(value) => __PartialExpr::Plus(value),
            Self::Literal(value) => __PartialExpr::Literal(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialExpr::Plus(value) => Self::Plus(value),
            __PartialExpr::Literal(value) => Self::Literal(value),
        }
    }
}
impl HasExtractorRef for Expr {
    type ExtractorRef<'__a__> = __PartialRefExpr<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Plus(value) => __PartialRefExpr::Plus(value),
            Self::Literal(value) => __PartialRefExpr::Literal(value),
        }
    }
}
impl HasExtractorMut for Expr {
    type ExtractorMut<'__a__> = __PartialRefExpr<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Plus(value) => __PartialRefExpr::Plus(value),
            Self::Literal(value) => __PartialRefExpr::Literal(value),
        }
    }
}
impl FinalizeExtract for __PartialExpr<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefExpr<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Plus")>
for __PartialExpr<IsPresent, __F1__> {
    type Value = Plus<Expr>;
    type Remainder = __PartialExpr<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Plus")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Plus")>>::Value,
        <Self as ExtractField<Symbol!("Plus")>>::Remainder,
    > {
        match self {
            __PartialExpr::Plus(value) => Ok(value),
            __PartialExpr::Literal(value) => Err(__PartialExpr::Literal(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Literal")>
for __PartialExpr<__F0__, IsPresent> {
    type Value = Literal;
    type Remainder = __PartialExpr<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Literal")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Literal")>>::Value,
        <Self as ExtractField<Symbol!("Literal")>>::Remainder,
    > {
        match self {
            __PartialExpr::Plus(value) => Err(__PartialExpr::Plus(value)),
            __PartialExpr::Literal(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Plus")>
for __PartialRefExpr<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Plus<Expr>>;
    type Remainder = __PartialRefExpr<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Plus")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Plus")>>::Value,
        <Self as ExtractField<Symbol!("Plus")>>::Remainder,
    > {
        match self {
            __PartialRefExpr::Plus(value) => Ok(value),
            __PartialRefExpr::Literal(value) => Err(__PartialRefExpr::Literal(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Literal")>
for __PartialRefExpr<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Literal>;
    type Remainder = __PartialRefExpr<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Literal")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Literal")>>::Value,
        <Self as ExtractField<Symbol!("Literal")>>::Remainder,
    > {
        match self {
            __PartialRefExpr::Plus(value) => Err(__PartialRefExpr::Plus(value)),
            __PartialRefExpr::Literal(value) => Ok(value),
        }
    }
}
impl<__Context__, Code> Computer<__Context__, Code, Plus<Expr>> for EvalAdd
where
    __Context__: CanCompute<Code, Expr, Output = u64>,
{
    type Output = u64;
    fn compute(
        __context__: &__Context__,
        code: PhantomData<Code>,
        Plus { left, right }: Plus<Expr>,
    ) -> u64 {
        __context__.compute(code, *left) + __context__.compute(code, *right)
    }
}
impl<__Context__, Code> IsProviderFor<ComputerComponent, __Context__, (Code, Plus<Expr>)>
for EvalAdd
where
    __Context__: CanCompute<Code, Expr, Output = u64>,
{}
pub struct EvalAdd;
impl<__Context__, Code> Computer<__Context__, Code, Literal> for EvalLiteral {
    type Output = u64;
    fn compute(
        __context__: &__Context__,
        _code: PhantomData<Code>,
        Literal(value): Literal,
    ) -> u64 {
        value
    }
}
impl<__Context__, Code> IsProviderFor<ComputerComponent, __Context__, (Code, Literal)>
for EvalLiteral {}
pub struct EvalLiteral;
pub struct Interp;
impl DelegateComponent<ComputerComponent> for Interp {
    type Delegate = RedirectLookup<Interp, Path!(@ComputerComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerComponent, __Context__, __Params__> for Interp
where
    RedirectLookup<
        Interp,
        Path!(@ComputerComponent),
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
impl<
    Code,
    __Wildcard__,
> DelegateComponent<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Expr, __Wildcard__>>>,
> for Interp {
    type Delegate = MatchWithValueHandlers;
}
impl<
    Code,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Expr, __Wildcard__>>>,
    __Context__,
    __Params__,
> for Interp
where
    MatchWithValueHandlers: IsProviderFor<
        PathCons<ComputerComponent, PathCons<Code, PathCons<Expr, __Wildcard__>>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    Code,
    __Wildcard__,
> DelegateComponent<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Plus<Expr>, __Wildcard__>>>,
> for Interp {
    type Delegate = EvalAdd;
}
impl<
    Code,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Plus<Expr>, __Wildcard__>>>,
    __Context__,
    __Params__,
> for Interp
where
    EvalAdd: IsProviderFor<
        PathCons<ComputerComponent, PathCons<Code, PathCons<Plus<Expr>, __Wildcard__>>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    Code,
    __Wildcard__,
> DelegateComponent<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Literal, __Wildcard__>>>,
> for Interp {
    type Delegate = EvalLiteral;
}
impl<
    Code,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Literal, __Wildcard__>>>,
    __Context__,
    __Params__,
> for Interp
where
    EvalLiteral: IsProviderFor<
        PathCons<ComputerComponent, PathCons<Code, PathCons<Literal, __Wildcard__>>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckInterp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckInterp<ComputerComponent, (Eval, Expr)> for Interp {}
impl __CheckInterp<ComputerComponent, (Eval, Plus<Expr>)> for Interp {}
fn main() {}
