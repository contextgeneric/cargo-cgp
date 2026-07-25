#![feature(prelude_import)]
//! A present-but-underived field beside a genuinely missing one on the same struct.
//!
//! `Shape` carries `width` but no `#[derive(HasField)]`, and does not carry `depth` at
//! all; the provider reads both. The two shortcomings are distinct fixes — add the
//! derive for `width`, add the field for `depth` — so they must stay *two* root causes:
//! the underived-field coalescing (which merges several underived fields on one struct
//! into a single add-the-derive cause, as in `base_area_2`) keys on a *group* of
//! present-but-underived fields and must leave a lone underived field, and the missing
//! field beside it, untouched. Pins that boundary.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md
//! (derive-missing and missing-field variants together).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCalculateVolume {
    fn volume(&self) -> f64;
}
impl<__Context__> CanCalculateVolume for __Context__
where
    __Context__: VolumeCalculator<__Context__>,
{
    fn volume(&self) -> f64 {
        __Context__::volume(self)
    }
}
pub trait VolumeCalculator<
    __Context__,
>: IsProviderFor<VolumeCalculatorComponent, __Context__, ()> {
    fn volume(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> VolumeCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<VolumeCalculatorComponent>
        + IsProviderFor<VolumeCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        VolumeCalculatorComponent,
    >>::Delegate: VolumeCalculator<__Context__>,
{
    fn volume(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            VolumeCalculatorComponent,
        >>::Delegate::volume(__context__)
    }
}
pub struct VolumeCalculatorComponent;
impl<__Context__> VolumeCalculator<__Context__> for UseContext
where
    __Context__: CanCalculateVolume,
{
    fn volume(__context__: &__Context__) -> f64 {
        __Context__::volume(__context__)
    }
}
impl<__Context__> IsProviderFor<VolumeCalculatorComponent, __Context__, ()>
for UseContext
where
    __Context__: CanCalculateVolume,
{}
impl<__Context__, __Components__, __Path__> VolumeCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: VolumeCalculator<__Context__>,
{
    fn volume(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::volume(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<VolumeCalculatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<VolumeCalculatorComponent, __Context__, ()>
        + VolumeCalculator<__Context__>,
{}
pub trait HasShapeFields {
    fn width(&self) -> f64;
    fn depth(&self) -> f64;
}
impl<__Context__> HasShapeFields for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
    __Context__: HasField<Symbol!("depth"), Value = f64>,
{
    fn width(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
    }
    fn depth(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("depth")>).clone()
    }
}
impl<__Context__> VolumeCalculator<__Context__> for ShapeVolume
where
    __Context__: HasShapeFields,
{
    fn volume(__context__: &__Context__) -> f64 {
        __context__.width() * __context__.width() * __context__.depth()
    }
}
impl<__Context__> IsProviderFor<VolumeCalculatorComponent, __Context__, ()>
for ShapeVolume
where
    __Context__: HasShapeFields,
{}
pub struct ShapeVolume;
pub struct Shape {
    pub width: f64,
}
impl DelegateComponent<VolumeCalculatorComponent> for Shape {
    type Delegate = ShapeVolume;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<VolumeCalculatorComponent, __Context__, __Params__> for Shape
where
    ShapeVolume: IsProviderFor<VolumeCalculatorComponent, __Context__, __Params__>,
{}
trait __CheckShape<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckShape<VolumeCalculatorComponent, ()> for Shape {}
fn main() {}
