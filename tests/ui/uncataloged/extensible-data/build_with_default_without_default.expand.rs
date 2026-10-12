#![feature(prelude_import)]
//! Building with defaults when a field the source lacks has no `Default`.
//!
//! `Server::build_with_default(Host { … })` fails on `CanFinalizeWithDefault` for
//! `__PartialServer<IsPresent, IsNothing>`. The output never names `port` or the missing
//! `Port: Default`, which the optional-field reference quotes as the error.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/hidden-root-cause.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::field::impls::CanBuildWithDefault;
use cgp::prelude::*;
pub struct Port(pub u16);
pub struct Host {
    pub host: String,
}
impl HasField<Symbol!("host")> for Host {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &Self::Value {
        &self.host
    }
}
impl HasFieldMut<Symbol!("host")> for Host {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &mut Self::Value {
        &mut self.host
    }
}
impl HasFields for Host {
    type Fields = Struct! { host: String };
}
impl HasFieldsRef for Host {
    type FieldsRef<'__a> = Struct! { host: &'__a String }
    where
        Self: '__a;
}
impl FromFields for Host {
    fn from_fields(Cons(host, Nil): Self::Fields) -> Self {
        Self { host: host.value }
    }
}
impl ToFields for Host {
    fn to_fields(self) -> Self::Fields {
        Cons(self.host.into(), Nil)
    }
}
impl ToFieldsRef for Host {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.host).into(), Nil)
    }
}
pub struct __PartialHost<__F0__: MapType> {
    pub host: <__F0__ as MapType>::Map<String>,
}
impl HasBuilder for Host {
    type Builder = __PartialHost<IsNothing>;
    fn builder() -> Self::Builder {
        __PartialHost { host: () }
    }
}
impl IntoBuilder for Host {
    type Builder = __PartialHost<IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialHost { host: self.host }
    }
}
impl<__F0__: MapType> PartialData for __PartialHost<__F0__> {
    type Target = Host;
}
impl FinalizeBuild for __PartialHost<IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Host { host: self.host }
    }
}
impl<__M1__: MapType, __M2__: MapType> UpdateField<Symbol!("host"), __M2__>
for __PartialHost<__M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialHost<__M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("host")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (self.host, __PartialHost { host: value })
    }
}
impl HasField<Symbol!("host")> for __PartialHost<IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &Self::Value {
        &self.host
    }
}
pub struct Server {
    pub host: String,
    pub port: Port,
}
impl HasField<Symbol!("host")> for Server {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &Self::Value {
        &self.host
    }
}
impl HasFieldMut<Symbol!("host")> for Server {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &mut Self::Value {
        &mut self.host
    }
}
impl HasField<Symbol!("port")> for Server {
    type Value = Port;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("port")>,
    ) -> &Self::Value {
        &self.port
    }
}
impl HasFieldMut<Symbol!("port")> for Server {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("port")>,
    ) -> &mut Self::Value {
        &mut self.port
    }
}
impl HasFields for Server {
    type Fields = Struct! { host: String, port: Port };
}
impl HasFieldsRef for Server {
    type FieldsRef<'__a> = Struct! { host: &'__a String, port: &'__a Port }
    where
        Self: '__a;
}
impl FromFields for Server {
    fn from_fields(Cons(host, Cons(port, Nil)): Self::Fields) -> Self {
        Self {
            host: host.value,
            port: port.value,
        }
    }
}
impl ToFields for Server {
    fn to_fields(self) -> Self::Fields {
        Cons(self.host.into(), Cons(self.port.into(), Nil))
    }
}
impl ToFieldsRef for Server {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.host).into(), Cons((&self.port).into(), Nil))
    }
}
pub struct __PartialServer<__F0__: MapType, __F1__: MapType> {
    pub host: <__F0__ as MapType>::Map<String>,
    pub port: <__F1__ as MapType>::Map<Port>,
}
impl HasBuilder for Server {
    type Builder = __PartialServer<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialServer {
            host: (),
            port: (),
        }
    }
}
impl IntoBuilder for Server {
    type Builder = __PartialServer<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialServer {
            host: self.host,
            port: self.port,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialServer<__F0__, __F1__> {
    type Target = Server;
}
impl FinalizeBuild for __PartialServer<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Server {
            host: self.host,
            port: self.port,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("host"), __M2__> for __PartialServer<__M1__, __F1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialServer<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("host")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.host,
            __PartialServer {
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
> UpdateField<Symbol!("port"), __M2__> for __PartialServer<__F0__, __M1__> {
    type Value = Port;
    type Mapper = __M1__;
    type Output = __PartialServer<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("port")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.port,
            __PartialServer {
                host: self.host,
                port: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("host")> for __PartialServer<IsPresent, __F1__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("host")>,
    ) -> &Self::Value {
        &self.host
    }
}
impl<__F0__: MapType> HasField<Symbol!("port")> for __PartialServer<__F0__, IsPresent> {
    type Value = Port;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("port")>,
    ) -> &Self::Value {
        &self.port
    }
}
fn main() {
    let _ = Server::build_with_default(Host { host: "h".into() });
}
