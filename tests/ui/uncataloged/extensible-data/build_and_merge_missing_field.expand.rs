#![feature(prelude_import)]
//! A builder output whose field name does not match the merged target's.
//!
//! `Http { timeout_url }` is merged into an `App` that expects `url`. The output is byte-identical to
//! `build_and_merge_extra_field`: the same `CanBuildFrom<Http>` headline and tree, naming no field.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::extra::dispatch::BuildAndMergeOutputs;
use cgp::prelude::*;
pub struct Db {
    pub db_path: String,
}
impl HasField<Symbol!("db_path")> for Db {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("db_path")>,
    ) -> &Self::Value {
        &self.db_path
    }
}
impl HasFieldMut<Symbol!("db_path")> for Db {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("db_path")>,
    ) -> &mut Self::Value {
        &mut self.db_path
    }
}
impl HasFields for Db {
    type Fields = Struct! { db_path: String };
}
impl HasFieldsRef for Db {
    type FieldsRef<'__a> = Struct! { db_path: &'__a String }
    where
        Self: '__a;
}
impl FromFields for Db {
    fn from_fields(Cons(db_path, Nil): Self::Fields) -> Self {
        Self { db_path: db_path.value }
    }
}
impl ToFields for Db {
    fn to_fields(self) -> Self::Fields {
        Cons(self.db_path.into(), Nil)
    }
}
impl ToFieldsRef for Db {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.db_path).into(), Nil)
    }
}
pub struct __PartialDb<__F0__: MapType> {
    pub db_path: <__F0__ as MapType>::Map<String>,
}
impl HasBuilder for Db {
    type Builder = __PartialDb<IsNothing>;
    fn builder() -> Self::Builder {
        __PartialDb { db_path: () }
    }
}
impl IntoBuilder for Db {
    type Builder = __PartialDb<IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialDb {
            db_path: self.db_path,
        }
    }
}
impl<__F0__: MapType> PartialData for __PartialDb<__F0__> {
    type Target = Db;
}
impl FinalizeBuild for __PartialDb<IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Db { db_path: self.db_path }
    }
}
impl<__M1__: MapType, __M2__: MapType> UpdateField<Symbol!("db_path"), __M2__>
for __PartialDb<__M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialDb<__M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("db_path")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (self.db_path, __PartialDb { db_path: value })
    }
}
impl HasField<Symbol!("db_path")> for __PartialDb<IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("db_path")>,
    ) -> &Self::Value {
        &self.db_path
    }
}
pub struct BuildDb;
impl<Context, Code, Input> Computer<Context, Code, Input> for BuildDb {
    type Output = Db;
    fn compute(_context: &Context, _code: PhantomData<Code>, _input: Input) -> Db {
        Db { db_path: "db".into() }
    }
}
impl<Context, Code, Input> IsProviderFor<ComputerComponent, Context, (Code, Input)>
for BuildDb {}
pub struct BuildHttp;
impl<Context, Code, Input> Computer<Context, Code, Input> for BuildHttp {
    type Output = Http;
    fn compute(_context: &Context, _code: PhantomData<Code>, _input: Input) -> Http {
        Http::default()
    }
}
impl<Context, Code, Input> IsProviderFor<ComputerComponent, Context, (Code, Input)>
for BuildHttp {}
pub struct App {
    pub db_path: String,
    pub url: String,
}
impl HasField<Symbol!("db_path")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("db_path")>,
    ) -> &Self::Value {
        &self.db_path
    }
}
impl HasFieldMut<Symbol!("db_path")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("db_path")>,
    ) -> &mut Self::Value {
        &mut self.db_path
    }
}
impl HasField<Symbol!("url")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("url")>,
    ) -> &Self::Value {
        &self.url
    }
}
impl HasFieldMut<Symbol!("url")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("url")>,
    ) -> &mut Self::Value {
        &mut self.url
    }
}
impl HasFields for App {
    type Fields = Struct! { db_path: String, url: String };
}
impl HasFieldsRef for App {
    type FieldsRef<'__a> = Struct! { db_path: &'__a String, url: &'__a String }
    where
        Self: '__a;
}
impl FromFields for App {
    fn from_fields(Cons(db_path, Cons(url, Nil)): Self::Fields) -> Self {
        Self {
            db_path: db_path.value,
            url: url.value,
        }
    }
}
impl ToFields for App {
    fn to_fields(self) -> Self::Fields {
        Cons(self.db_path.into(), Cons(self.url.into(), Nil))
    }
}
impl ToFieldsRef for App {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.db_path).into(), Cons((&self.url).into(), Nil))
    }
}
pub struct __PartialApp<__F0__: MapType, __F1__: MapType> {
    pub db_path: <__F0__ as MapType>::Map<String>,
    pub url: <__F1__ as MapType>::Map<String>,
}
impl HasBuilder for App {
    type Builder = __PartialApp<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialApp {
            db_path: (),
            url: (),
        }
    }
}
impl IntoBuilder for App {
    type Builder = __PartialApp<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialApp {
            db_path: self.db_path,
            url: self.url,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialApp<__F0__, __F1__> {
    type Target = App;
}
impl FinalizeBuild for __PartialApp<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        App {
            db_path: self.db_path,
            url: self.url,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("db_path"), __M2__> for __PartialApp<__M1__, __F1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialApp<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("db_path")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.db_path,
            __PartialApp {
                db_path: value,
                url: self.url,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("url"), __M2__> for __PartialApp<__F0__, __M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialApp<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("url")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.url,
            __PartialApp {
                db_path: self.db_path,
                url: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("db_path")> for __PartialApp<IsPresent, __F1__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("db_path")>,
    ) -> &Self::Value {
        &self.db_path
    }
}
impl<__F0__: MapType> HasField<Symbol!("url")> for __PartialApp<__F0__, IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("url")>,
    ) -> &Self::Value {
        &self.url
    }
}
pub struct AppBuilder;
impl DelegateComponent<ComputerComponent> for AppBuilder {
    type Delegate = BuildAndMergeOutputs<App, Product![BuildDb, BuildHttp]>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerComponent, __Context__, __Params__> for AppBuilder
where
    BuildAndMergeOutputs<
        App,
        Product![BuildDb, BuildHttp],
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
trait __CheckAppBuilder<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckAppBuilder<ComputerComponent, ((), ())> for AppBuilder {}
fn main() {}
pub struct Http {
    pub timeout_url: String,
}
#[automatically_derived]
impl ::core::default::Default for Http {
    #[inline]
    fn default() -> Self {
        Http {
            timeout_url: ::core::default::Default::default(),
        }
    }
}
impl HasField<Symbol!("timeout_url")> for Http {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("timeout_url")>,
    ) -> &Self::Value {
        &self.timeout_url
    }
}
impl HasFieldMut<Symbol!("timeout_url")> for Http {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("timeout_url")>,
    ) -> &mut Self::Value {
        &mut self.timeout_url
    }
}
impl HasFields for Http {
    type Fields = Struct! { timeout_url: String };
}
impl HasFieldsRef for Http {
    type FieldsRef<'__a> = Struct! { timeout_url: &'__a String }
    where
        Self: '__a;
}
impl FromFields for Http {
    fn from_fields(Cons(timeout_url, Nil): Self::Fields) -> Self {
        Self {
            timeout_url: timeout_url.value,
        }
    }
}
impl ToFields for Http {
    fn to_fields(self) -> Self::Fields {
        Cons(self.timeout_url.into(), Nil)
    }
}
impl ToFieldsRef for Http {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.timeout_url).into(), Nil)
    }
}
pub struct __PartialHttp<__F0__: MapType> {
    pub timeout_url: <__F0__ as MapType>::Map<String>,
}
impl HasBuilder for Http {
    type Builder = __PartialHttp<IsNothing>;
    fn builder() -> Self::Builder {
        __PartialHttp { timeout_url: () }
    }
}
impl IntoBuilder for Http {
    type Builder = __PartialHttp<IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialHttp {
            timeout_url: self.timeout_url,
        }
    }
}
impl<__F0__: MapType> PartialData for __PartialHttp<__F0__> {
    type Target = Http;
}
impl FinalizeBuild for __PartialHttp<IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Http {
            timeout_url: self.timeout_url,
        }
    }
}
impl<__M1__: MapType, __M2__: MapType> UpdateField<Symbol!("timeout_url"), __M2__>
for __PartialHttp<__M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialHttp<__M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("timeout_url")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.timeout_url,
            __PartialHttp {
                timeout_url: value,
            },
        )
    }
}
impl HasField<Symbol!("timeout_url")> for __PartialHttp<IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("timeout_url")>,
    ) -> &Self::Value {
        &self.timeout_url
    }
}
