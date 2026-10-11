#![feature(prelude_import)]
//! A local key registered into an upstream namespace and into the parent it inherits.
//!
//! `AppNamespace`, from the `cgp-test-crate-a` auxiliary crate, inherits `DefaultNamespace`, so
//! that crate has a blanket impl of `AppNamespace` forwarding every key the parent answers. This
//! crate registers its own `LabelerComponent` into both, which the orphan rule allows, since the key
//! is local. The registration in `AppNamespace` then overlaps the upstream inheritance blanket
//! (E0119). rustc names the upstream impl in a note rather than at a labelled span, so the conflict
//! classifier finds no local "first implementation" to compare the entry with, and the diagnostic
//! keeps rustc's wording.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use cgp_test_crate_a::AppNamespace;
pub trait CanLabel {
    fn label(&self) -> String;
}
impl<__Context__> CanLabel for __Context__
where
    __Context__: Labeler<__Context__>,
{
    fn label(&self) -> String {
        __Context__::label(self)
    }
}
pub trait Labeler<__Context__>: IsProviderFor<LabelerComponent, __Context__, ()> {
    fn label(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Labeler<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<LabelerComponent>
        + IsProviderFor<LabelerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        LabelerComponent,
    >>::Delegate: Labeler<__Context__>,
{
    fn label(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            LabelerComponent,
        >>::Delegate::label(__context__)
    }
}
pub struct LabelerComponent;
impl<__Context__> Labeler<__Context__> for UseContext
where
    __Context__: CanLabel,
{
    fn label(__context__: &__Context__) -> String {
        __Context__::label(__context__)
    }
}
impl<__Context__> IsProviderFor<LabelerComponent, __Context__, ()> for UseContext
where
    __Context__: CanLabel,
{}
impl<__Context__, __Components__, __Path__> Labeler<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Labeler<__Context__>,
{
    fn label(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::label(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<LabelerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<LabelerComponent, __Context__, ()>
        + Labeler<__Context__>,
{}
impl<__Context__> Labeler<__Context__> for FixedLabel {
    fn label(__context__: &__Context__) -> String {
        "label".to_owned()
    }
}
impl<__Context__> IsProviderFor<LabelerComponent, __Context__, ()> for FixedLabel {}
pub struct FixedLabel;
impl<__Components__> DefaultNamespace<__Components__> for LabelerComponent {
    type Delegate = FixedLabel;
}
impl<__Context__> Labeler<__Context__> for OtherLabel {
    fn label(__context__: &__Context__) -> String {
        "other".to_owned()
    }
}
impl<__Context__> IsProviderFor<LabelerComponent, __Context__, ()> for OtherLabel {}
pub struct OtherLabel;
impl<__Components__> AppNamespace<__Components__> for LabelerComponent {
    type Delegate = OtherLabel;
}
fn main() {}
