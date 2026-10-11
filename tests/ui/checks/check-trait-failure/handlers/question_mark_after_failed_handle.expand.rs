#![feature(prelude_import)]
//! A `BuildAndMergeOutputs` builder whose second output carries a field the target lacks, called with
//! `?` and no check.
//!
//! The call-site anchor recovers `[CGP-E001] the consumer trait `CanHandle<(), ()>`` and walks the
//! builder pipeline to `__PartialApp<IsPresent>: CanBuildFrom<Http>`, the merge that needs the `url`
//! field `App` lacks; the field itself is not named. Distilled from the application builder with its
//! `http_client` field removed.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::dispatch::BuildAndMergeOutputs;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent};
use cgp::prelude::*;
mod backend {
    use core::error::Error as StdError;
    use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
    use cgp::prelude::*;
    pub struct Error(pub String);
    #[automatically_derived]
    impl ::core::fmt::Debug for Error {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Error", &&self.0)
        }
    }
    pub struct RaiseBackendError;
    impl<__Context__, E> ErrorRaiser<__Context__, E> for RaiseBackendError
    where
        E: StdError + Send + Sync + 'static,
        __Context__: HasErrorType<Error = crate::backend::Error>,
    {
        fn raise_error(e: E) -> <__Context__ as HasErrorType>::Error {
            crate::backend::Error(e.to_string())
        }
    }
    impl<__Context__, E> IsProviderFor<ErrorRaiserComponent, __Context__, (E)>
    for RaiseBackendError
    where
        E: StdError + Send + Sync + 'static,
        __Context__: HasErrorType<Error = crate::backend::Error>,
    {}
}
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
pub struct Http {
    pub url: String,
}
impl HasField<Symbol!("url")> for Http {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("url")>,
    ) -> &Self::Value {
        &self.url
    }
}
impl HasFieldMut<Symbol!("url")> for Http {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("url")>,
    ) -> &mut Self::Value {
        &mut self.url
    }
}
impl HasFields for Http {
    type Fields = Struct! { url: String };
}
impl HasFieldsRef for Http {
    type FieldsRef<'__a> = Struct! { url: &'__a String }
    where
        Self: '__a;
}
impl FromFields for Http {
    fn from_fields(Cons(url, Nil): Self::Fields) -> Self {
        Self { url: url.value }
    }
}
impl ToFields for Http {
    fn to_fields(self) -> Self::Fields {
        Cons(self.url.into(), Nil)
    }
}
impl ToFieldsRef for Http {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.url).into(), Nil)
    }
}
pub struct __PartialHttp<__F0__: MapType> {
    pub url: <__F0__ as MapType>::Map<String>,
}
impl HasBuilder for Http {
    type Builder = __PartialHttp<IsNothing>;
    fn builder() -> Self::Builder {
        __PartialHttp { url: () }
    }
}
impl IntoBuilder for Http {
    type Builder = __PartialHttp<IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialHttp { url: self.url }
    }
}
impl<__F0__: MapType> PartialData for __PartialHttp<__F0__> {
    type Target = Http;
}
impl FinalizeBuild for __PartialHttp<IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Http { url: self.url }
    }
}
impl<__M1__: MapType, __M2__: MapType> UpdateField<Symbol!("url"), __M2__>
for __PartialHttp<__M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialHttp<__M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("url")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (self.url, __PartialHttp { url: value })
    }
}
impl HasField<Symbol!("url")> for __PartialHttp<IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("url")>,
    ) -> &Self::Value {
        &self.url
    }
}
pub struct App {
    pub db_path: String,
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
impl HasFields for App {
    type Fields = Struct! { db_path: String };
}
impl HasFieldsRef for App {
    type FieldsRef<'__a> = Struct! { db_path: &'__a String }
    where
        Self: '__a;
}
impl FromFields for App {
    fn from_fields(Cons(db_path, Nil): Self::Fields) -> Self {
        Self { db_path: db_path.value }
    }
}
impl ToFields for App {
    fn to_fields(self) -> Self::Fields {
        Cons(self.db_path.into(), Nil)
    }
}
impl ToFieldsRef for App {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.db_path).into(), Nil)
    }
}
pub struct __PartialApp<__F0__: MapType> {
    pub db_path: <__F0__ as MapType>::Map<String>,
}
impl HasBuilder for App {
    type Builder = __PartialApp<IsNothing>;
    fn builder() -> Self::Builder {
        __PartialApp { db_path: () }
    }
}
impl IntoBuilder for App {
    type Builder = __PartialApp<IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialApp {
            db_path: self.db_path,
        }
    }
}
impl<__F0__: MapType> PartialData for __PartialApp<__F0__> {
    type Target = App;
}
impl FinalizeBuild for __PartialApp<IsPresent> {
    fn finalize_build(self) -> Self::Target {
        App { db_path: self.db_path }
    }
}
impl<__M1__: MapType, __M2__: MapType> UpdateField<Symbol!("db_path"), __M2__>
for __PartialApp<__M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialApp<__M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("db_path")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (self.db_path, __PartialApp { db_path: value })
    }
}
impl HasField<Symbol!("db_path")> for __PartialApp<IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("db_path")>,
    ) -> &Self::Value {
        &self.db_path
    }
}
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for BuildDb
where
    __Context__: HasErrorType,
{
    type Output = Db;
    async fn handle(
        __context__: &__Context__,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Db, <__Context__ as HasErrorType>::Error> {
        Ok(Db { db_path: "db".to_owned() })
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for BuildDb
where
    __Context__: HasErrorType,
{}
pub struct BuildDb;
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for BuildHttp
where
    __Context__: HasErrorType,
{
    type Output = Http;
    async fn handle(
        __context__: &__Context__,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Http, <__Context__ as HasErrorType>::Error> {
        Ok(Http { url: "url".to_owned() })
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for BuildHttp
where
    __Context__: HasErrorType,
{}
pub struct BuildHttp;
pub struct AppBuilder;
impl DelegateComponent<ErrorTypeProviderComponent> for AppBuilder {
    type Delegate = UseType<backend::Error>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for AppBuilder
where
    UseType<
        backend::Error,
    >: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ErrorRaiserComponent> for AppBuilder {
    type Delegate = backend::RaiseBackendError;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorRaiserComponent, __Context__, __Params__> for AppBuilder
where
    backend::RaiseBackendError: IsProviderFor<
        ErrorRaiserComponent,
        __Context__,
        __Params__,
    >,
{}
impl DelegateComponent<HandlerComponent> for AppBuilder {
    type Delegate = BuildAndMergeOutputs<App, Product![BuildDb, BuildHttp]>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerComponent, __Context__, __Params__> for AppBuilder
where
    BuildAndMergeOutputs<
        App,
        Product![BuildDb, BuildHttp],
    >: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
pub async fn run() -> Result<(), backend::Error> {
    let _app = AppBuilder.handle(PhantomData::<()>, ()).await?;
    Ok(())
}
fn main() {}
