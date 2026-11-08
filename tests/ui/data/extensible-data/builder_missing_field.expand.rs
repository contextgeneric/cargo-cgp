#![feature(prelude_import)]
//! A `BuildWithHandlers` list without a step for one field.
//!
//! The check reads the builder's presence markers when its `FinalizeBuild` bound fails:
//! ``[CGP-E001] field `port` of `Config` is not set``, under the chain through `BuildWithHandlers`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::extra::dispatch::{BuildAndSetField, BuildWithHandlers};
use cgp::prelude::*;
pub struct Config {
    pub host: String,
    pub port: u16,
}
impl HasField<Symbol!("host")> for Config {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &Self::Value {
        &self.host
    }
}
impl HasFieldMut<Symbol!("host")> for Config {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &mut Self::Value {
        &mut self.host
    }
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
impl HasFields for Config {
    type Fields = Struct! { host: String, port: u16 };
}
impl HasFieldsRef for Config {
    type FieldsRef<'__a> = Struct! { host: &'__a String, port: &'__a u16 }
    where
        Self: '__a;
}
impl FromFields for Config {
    fn from_fields(Cons(host, Cons(port, Nil)): Self::Fields) -> Self {
        Self {
            host: host.value,
            port: port.value,
        }
    }
}
impl ToFields for Config {
    fn to_fields(self) -> Self::Fields {
        Cons(
            ::core::convert::Into::into(self.host),
            Cons(::core::convert::Into::into(self.port), Nil),
        )
    }
}
impl ToFieldsRef for Config {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons(
            ::core::convert::Into::into(&self.host),
            Cons(::core::convert::Into::into(&self.port), Nil),
        )
    }
}
///The partial builder for [`Config`].
pub struct __PartialConfig<__F0__: MapType, __F1__: MapType> {
    pub host: <__F0__ as MapType>::Map<String>,
    pub port: <__F1__ as MapType>::Map<u16>,
}
impl HasBuilder for Config {
    type Builder = __PartialConfig<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialConfig {
            host: (),
            port: (),
        }
    }
}
impl IntoBuilder for Config {
    type Builder = __PartialConfig<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialConfig {
            host: self.host,
            port: self.port,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialConfig<__F0__, __F1__> {
    type Target = Config;
}
impl FinalizeBuild for __PartialConfig<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Config {
            host: self.host,
            port: self.port,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("host"), __M2__> for __PartialConfig<__M1__, __F1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialConfig<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("host")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.host,
            __PartialConfig {
                host: value,
                port: self.port,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("port"), __M2__> for __PartialConfig<__F0__, __M1__> {
    type Value = u16;
    type Mapper = __M1__;
    type Output = __PartialConfig<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("port")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.port,
            __PartialConfig {
                host: self.host,
                port: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("host")> for __PartialConfig<IsPresent, __F1__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &Self::Value {
        &self.host
    }
}
impl<__F0__: MapType> HasField<Symbol!("port")> for __PartialConfig<__F0__, IsPresent> {
    type Value = u16;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("port")>,
    ) -> &Self::Value {
        &self.port
    }
}
pub struct DefaultHost;
impl<Context, Code, Builder> Computer<Context, Code, &Builder> for DefaultHost {
    type Output = String;
    fn compute(
        _context: &Context,
        _code: PhantomData<Code>,
        _builder: &Builder,
    ) -> String {
        "localhost".to_owned()
    }
}
impl<Context, Code, Builder> IsProviderFor<ComputerComponent, Context, (Code, &Builder)>
for DefaultHost {}
pub struct App;
impl DelegateComponent<ComputerComponent> for App {
    type Delegate = BuildWithHandlers<
        Config,
        Product![BuildAndSetField<Symbol!("host"), DefaultHost>],
    >;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerComponent, __Context__, __Params__> for App
where
    BuildWithHandlers<
        Config,
        Product![BuildAndSetField<Symbol!("host"), DefaultHost>],
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ComputerComponent, ((), ())> for App {}
fn main() {}
