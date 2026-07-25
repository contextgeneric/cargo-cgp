#![feature(prelude_import)]
//! Acceptable failure: two nested `#[use_type]` imports whose `in Context` clauses
//! reference *each other*, so there is no valid order in which to ground them.
//!
//! `HasA.A in B` resolves `A` against `B`, and `HasB.B in A` resolves `B` against
//! `A` — a cycle. Grounding runs to a fixpoint and deliberately stops making
//! progress on a cycle rather than looping, so the context aliases are never
//! resolved and the rewrite leaves the bare `A` and `B` from the `in` clauses in
//! type position. CGP lowers the input faithfully and defers to the compiler,
//! which reports `E0425` "cannot find type" with the caret on the unresolved
//! context alias the user wrote in the attribute.
//!
//! An *acyclic* chain in any order (`HasC.C in B, HasB.B in A, HasA.A` written
//! back-to-front) grounds fine — see the passing `use_type_fn_reverse_order`
//! behavioral test. Only a genuine cycle, which has no valid ordering, fails.
//!
//! See cgp-knowledge-base/cgp/reference/attributes/use_type.md and
//! cgp-knowledge-base/cgp/errors/lowering/unresolved-imported-type.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasA {
    type A;
}
impl<__Context__> HasA for __Context__
where
    __Context__: ATypeProvider<__Context__>,
{
    type A = <__Context__ as ATypeProvider<__Context__>>::A;
}
pub trait ATypeProvider<
    __Context__,
>: IsProviderFor<ATypeProviderComponent, __Context__, ()> {
    type A;
}
impl<__Provider__, __Context__> ATypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ATypeProviderComponent>
        + IsProviderFor<ATypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ATypeProviderComponent,
    >>::Delegate: ATypeProvider<__Context__>,
{
    type A = <<__Provider__ as DelegateComponent<
        ATypeProviderComponent,
    >>::Delegate as ATypeProvider<__Context__>>::A;
}
pub struct ATypeProviderComponent;
impl<__Context__> ATypeProvider<__Context__> for UseContext
where
    __Context__: HasA,
{
    type A = <__Context__ as HasA>::A;
}
impl<__Context__> IsProviderFor<ATypeProviderComponent, __Context__, ()> for UseContext
where
    __Context__: HasA,
{}
impl<__Context__, __Components__, __Path__> ATypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: ATypeProvider<__Context__>,
{
    type A = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as ATypeProvider<__Context__>>::A;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ATypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ATypeProviderComponent, __Context__, ()>
        + ATypeProvider<__Context__>,
{}
impl<A, __Context__> ATypeProvider<__Context__> for UseType<A> {
    type A = A;
}
impl<A, __Context__> IsProviderFor<ATypeProviderComponent, __Context__, ()>
for UseType<A> {}
impl<__Provider__, A, __Context__> ATypeProvider<__Context__>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, ATypeProviderComponent, Type = A>,
{
    type A = A;
}
impl<__Provider__, A, __Context__> IsProviderFor<ATypeProviderComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, ATypeProviderComponent, Type = A>,
{}
pub trait HasB {
    type B;
}
impl<__Context__> HasB for __Context__
where
    __Context__: BTypeProvider<__Context__>,
{
    type B = <__Context__ as BTypeProvider<__Context__>>::B;
}
pub trait BTypeProvider<
    __Context__,
>: IsProviderFor<BTypeProviderComponent, __Context__, ()> {
    type B;
}
impl<__Provider__, __Context__> BTypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<BTypeProviderComponent>
        + IsProviderFor<BTypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        BTypeProviderComponent,
    >>::Delegate: BTypeProvider<__Context__>,
{
    type B = <<__Provider__ as DelegateComponent<
        BTypeProviderComponent,
    >>::Delegate as BTypeProvider<__Context__>>::B;
}
pub struct BTypeProviderComponent;
impl<__Context__> BTypeProvider<__Context__> for UseContext
where
    __Context__: HasB,
{
    type B = <__Context__ as HasB>::B;
}
impl<__Context__> IsProviderFor<BTypeProviderComponent, __Context__, ()> for UseContext
where
    __Context__: HasB,
{}
impl<__Context__, __Components__, __Path__> BTypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: BTypeProvider<__Context__>,
{
    type B = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as BTypeProvider<__Context__>>::B;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<BTypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<BTypeProviderComponent, __Context__, ()>
        + BTypeProvider<__Context__>,
{}
impl<B, __Context__> BTypeProvider<__Context__> for UseType<B> {
    type B = B;
}
impl<B, __Context__> IsProviderFor<BTypeProviderComponent, __Context__, ()>
for UseType<B> {}
impl<__Provider__, B, __Context__> BTypeProvider<__Context__>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, BTypeProviderComponent, Type = B>,
{
    type B = B;
}
impl<__Provider__, B, __Context__> IsProviderFor<BTypeProviderComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, BTypeProviderComponent, Type = B>,
{}
pub trait Deep
where
    <<<A as HasB>::B as HasA>::A as HasB>::B: HasA,
    <<<B as HasA>::A as HasB>::B as HasA>::A: HasB,
{
    fn deep(&self) -> <<<<A as HasB>::B as HasA>::A as HasB>::B as HasA>::A;
}
impl<__Context__> Deep for __Context__
where
    <<<A as HasB>::B as HasA>::A as HasB>::B: HasA,
    <<<B as HasA>::A as HasB>::B as HasA>::A: HasB,
{
    fn deep(&self) -> <<<<A as HasB>::B as HasA>::A as HasB>::B as HasA>::A {
        ::core::panicking::panic("not yet implemented")
    }
}
fn main() {}
