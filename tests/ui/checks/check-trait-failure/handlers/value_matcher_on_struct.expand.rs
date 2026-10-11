#![feature(prelude_import)]
//! `MatchWithValueHandlers` applied to a struct.
//!
//! A `[CGP-E201]` leaf on `HasFieldHandlers<…>`; nothing says the matchers need an enum.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::prelude::*;
pub struct Scene {
    pub width: f64,
}
impl HasField<Symbol!("width")> for Scene {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &Self::Value {
        &self.width
    }
}
impl HasFieldMut<Symbol!("width")> for Scene {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &mut Self::Value {
        &mut self.width
    }
}
impl HasFields for Scene {
    type Fields = Struct! { width: f64 };
}
impl HasFieldsRef for Scene {
    type FieldsRef<'__a> = Struct! { width: &'__a f64 }
    where
        Self: '__a;
}
impl FromFields for Scene {
    fn from_fields(Cons(width, Nil): Self::Fields) -> Self {
        Self { width: width.value }
    }
}
impl ToFields for Scene {
    fn to_fields(self) -> Self::Fields {
        Cons(self.width.into(), Nil)
    }
}
impl ToFieldsRef for Scene {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.width).into(), Nil)
    }
}
pub struct __PartialScene<__F0__: MapType> {
    pub width: <__F0__ as MapType>::Map<f64>,
}
impl HasBuilder for Scene {
    type Builder = __PartialScene<IsNothing>;
    fn builder() -> Self::Builder {
        __PartialScene { width: () }
    }
}
impl IntoBuilder for Scene {
    type Builder = __PartialScene<IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialScene {
            width: self.width,
        }
    }
}
impl<__F0__: MapType> PartialData for __PartialScene<__F0__> {
    type Target = Scene;
}
impl FinalizeBuild for __PartialScene<IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Scene { width: self.width }
    }
}
impl<__M1__: MapType, __M2__: MapType> UpdateField<Symbol!("width"), __M2__>
for __PartialScene<__M1__> {
    type Value = f64;
    type Mapper = __M1__;
    type Output = __PartialScene<__M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("width")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (self.width, __PartialScene { width: value })
    }
}
impl HasField<Symbol!("width")> for __PartialScene<IsPresent> {
    type Value = f64;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &Self::Value {
        &self.width
    }
}
pub struct ComputeArea;
impl<Context, Code> Computer<Context, Code, f64> for ComputeArea {
    type Output = f64;
    fn compute(_context: &Context, _code: PhantomData<Code>, value: f64) -> f64 {
        value
    }
}
impl<Context, Code> IsProviderFor<ComputerComponent, Context, (Code, f64)>
for ComputeArea {}
pub struct App;
impl DelegateComponent<ComputerComponent> for App {
    type Delegate = MatchWithValueHandlers<ComputeArea>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerComponent, __Context__, __Params__> for App
where
    MatchWithValueHandlers<
        ComputeArea,
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ComputerComponent, ((), Scene)> for App {}
fn main() {}
