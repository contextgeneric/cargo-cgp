#![feature(prelude_import)]
//! A field transform with no conversion for one of the builder's markers.
//!
//! `OnlyAbsent` converts only absent fields, but `port` is already set. The output is an `E0277` on
//! `OnlyAbsent: TransformMap<IsPresent, IsPresent, u16>`, which names the field by its value type
//! rather than its name. rustc reports it twice, at the method name and at the whole call, and the
//! tool merges the repeat into the first block, whose labels already cover the call.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::traits::{TransformMap, TransformMapFields};
use cgp::prelude::*;
pub struct OnlyAbsent;
impl<T: Default> TransformMap<IsNothing, IsPresent, T> for OnlyAbsent {
    fn transform_mapped(_value: ()) -> T {
        T::default()
    }
}
pub struct Config {
    pub port: u16,
    pub verbose: bool,
}
impl HasField<Symbol!("port")> for Config {
    type Value = u16;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("port")>,
    ) -> &Self::Value {
        &self.port
    }
}
impl HasFieldMut<Symbol!("port")> for Config {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("port")>,
    ) -> &mut Self::Value {
        &mut self.port
    }
}
impl HasField<Symbol!("verbose")> for Config {
    type Value = bool;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("verbose")>,
    ) -> &Self::Value {
        &self.verbose
    }
}
impl HasFieldMut<Symbol!("verbose")> for Config {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("verbose")>,
    ) -> &mut Self::Value {
        &mut self.verbose
    }
}
impl HasFields for Config {
    type Fields = Struct! { port: u16, verbose: bool };
}
impl HasFieldsRef for Config {
    type FieldsRef<'__a> = Struct! { port: &'__a u16, verbose: &'__a bool }
    where
        Self: '__a;
}
impl FromFields for Config {
    fn from_fields(Cons(port, Cons(verbose, Nil)): Self::Fields) -> Self {
        Self {
            port: port.value,
            verbose: verbose.value,
        }
    }
}
impl ToFields for Config {
    fn to_fields(self) -> Self::Fields {
        Cons(self.port.into(), Cons(self.verbose.into(), Nil))
    }
}
impl ToFieldsRef for Config {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.port).into(), Cons((&self.verbose).into(), Nil))
    }
}
pub struct __PartialConfig<__F0__: MapType, __F1__: MapType> {
    pub port: <__F0__ as MapType>::Map<u16>,
    pub verbose: <__F1__ as MapType>::Map<bool>,
}
impl HasBuilder for Config {
    type Builder = __PartialConfig<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialConfig {
            port: (),
            verbose: (),
        }
    }
}
impl IntoBuilder for Config {
    type Builder = __PartialConfig<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialConfig {
            port: self.port,
            verbose: self.verbose,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialConfig<__F0__, __F1__> {
    type Target = Config;
}
impl FinalizeBuild for __PartialConfig<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Config {
            port: self.port,
            verbose: self.verbose,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("port"), __M2__> for __PartialConfig<__M1__, __F1__> {
    type Value = u16;
    type Mapper = __M1__;
    type Output = __PartialConfig<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("port")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.port,
            __PartialConfig {
                port: value,
                verbose: self.verbose,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("verbose"), __M2__> for __PartialConfig<__F0__, __M1__> {
    type Value = bool;
    type Mapper = __M1__;
    type Output = __PartialConfig<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("verbose")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.verbose,
            __PartialConfig {
                port: self.port,
                verbose: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("port")> for __PartialConfig<IsPresent, __F1__> {
    type Value = u16;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("port")>,
    ) -> &Self::Value {
        &self.port
    }
}
impl<__F0__: MapType> HasField<Symbol!("verbose")>
for __PartialConfig<__F0__, IsPresent> {
    type Value = bool;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("verbose")>,
    ) -> &Self::Value {
        &self.verbose
    }
}
fn main() {
    let partial = Config::builder().build_field(PhantomData::<Symbol!("port")>, 8080);
    let _ = TransformMapFields::<OnlyAbsent, IsPresent>::transform_map_fields(partial)
        .finalize_build();
}
