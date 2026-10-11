#![feature(prelude_import)]
//! A local wiring entry on an upstream context that joins a namespace.
//!
//! `Broadcaster`, from the `cgp-test-crate-b` auxiliary crate, joins `DefaultNamespace`, so that
//! crate has a blanket `DelegateComponent<__Key__>` impl forwarding every key the namespace
//! answers. This crate registers its own `LabelerComponent` into `DefaultNamespace` with
//! `#[default_impl]` and also wires it on `Broadcaster` directly, which the orphan rule allows,
//! since the key is local. The two now overlap (E0119). rustc names the upstream impl in a note
//! rather than at a labelled span, so the conflict classifier finds no local "first
//! implementation" to compare the entry with.
//!
//! Without the first entry the tool cannot say what the key collides with, so both `E0119`s keep
//! rustc's wording, each naming the upstream impl in its note. Calling the conflict a duplicate
//! entry would be false: the context has one entry for the key, and it collides with the namespace
//! the context joins upstream.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use cgp_test_crate_b::Broadcaster;
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
impl DelegateComponent<LabelerComponent> for Broadcaster {
    type Delegate = FixedLabel;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<LabelerComponent, __Context__, __Params__> for Broadcaster
where
    FixedLabel: IsProviderFor<LabelerComponent, __Context__, __Params__>,
{}
fn main() {}
