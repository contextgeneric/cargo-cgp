#![feature(prelude_import)]
//! Acceptable: a wiring gap surfaced through a plain trait implemented on a *foreign* wrapper type,
//! whose `where`-clause chain reaches a CGP consumer several hops down — the shape the
//! `cgp-examples/transfer` walkthrough hits with `impl CanAddApiRoutes for Router<Arc<MockApp>>`,
//! traced to a root-cause tree by the wrapper-chain anchor.
//!
//! Unlike [`manual_supertrait_impl`](../../use-site/manual_supertrait_impl.rs) and
//! [`traced_send_wrapper`](traced_send_wrapper.rs) — where the wrapper is implemented *on the context
//! itself* and carries the CGP consumer trait as a *direct* supertrait — here the routing trait
//! `CanAddAppRoutes` is implemented for the foreign `Box<App>`, and the CGP consumer failure sits two
//! hops down a chain of ordinary user-trait `where`-clauses (`Box<App>: CanAddRoute<App, GreetApi>` →
//! `App: CanHandleApi<GreetApi>`). The context `App` appears only as a type *argument*, never as the
//! impl's `Self`, so the impl-site anchor's "`Self` is a local context whose direct supertrait is a
//! CGP consumer" recovery does not fire; the wrapper-chain anchor descends the `where`-clauses to
//! reach it instead.
//!
//! The routing impl carries the associated-type bound `Ctx::Response: Send` (the transfer example's
//! `App::Request: FromRequestParts` shape): it is the projection over the broken `CanHandleApi` that
//! makes the downstream `CanAddRoute` genuinely fail — the direct `App: CanHandleApiSend<GreetApi>`
//! bound is instead *assumed to hold* off its ill-formed impl, so the anchor reaches the real cause
//! through the projection's base trait (`App: CanHandleApi<GreetApi>`) rather than that bound. The
//! tree is headed by the code the programmer wrote (`CanAddAppRoutes`), and the header names the
//! foreign `Box<App>` plainly — never mislabelling it a "context".
//!
//! The one mistake is the missing `name` field `HandleGreet` reads. It surfaces both at the
//! hand-written `impl CanHandleApiSend<GreetApi> for App` (traced by the impl-site anchor) and — the
//! case this fixture pins — at the foreign `impl CanAddAppRoutes for Box<App>` several hops removed.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md (the
//! wrapper-chain path).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::future::Future;
use core::marker::PhantomData;
use cgp::prelude::*;
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("name")>).as_str()
    }
}
pub trait CanHandleApi<Api> {
    type Response;
    fn handle_api(
        &self,
        api: PhantomData<Api>,
    ) -> impl ::core::future::Future<Output = Self::Response>;
}
impl<__Context__, Api> CanHandleApi<Api> for __Context__
where
    __Context__: ApiHandler<__Context__, Api>,
{
    type Response = <__Context__ as ApiHandler<__Context__, Api>>::Response;
    async fn handle_api(&self, api: PhantomData<Api>) -> Self::Response {
        __Context__::handle_api(self, api).await
    }
}
pub trait ApiHandler<
    __Context__,
    Api,
>: IsProviderFor<ApiHandlerComponent, __Context__, (Api)> {
    type Response;
    fn handle_api(
        __context__: &__Context__,
        api: PhantomData<Api>,
    ) -> impl ::core::future::Future<Output = Self::Response>;
}
impl<__Provider__, __Context__, Api> ApiHandler<__Context__, Api> for __Provider__
where
    __Provider__: DelegateComponent<ApiHandlerComponent>
        + IsProviderFor<ApiHandlerComponent, __Context__, (Api)>,
    <__Provider__ as DelegateComponent<
        ApiHandlerComponent,
    >>::Delegate: ApiHandler<__Context__, Api>,
{
    type Response = <<__Provider__ as DelegateComponent<
        ApiHandlerComponent,
    >>::Delegate as ApiHandler<__Context__, Api>>::Response;
    async fn handle_api(
        __context__: &__Context__,
        api: PhantomData<Api>,
    ) -> Self::Response {
        <__Provider__ as DelegateComponent<
            ApiHandlerComponent,
        >>::Delegate::handle_api(__context__, api)
            .await
    }
}
pub struct ApiHandlerComponent;
impl<__Context__, Api> ApiHandler<__Context__, Api> for UseContext
where
    __Context__: CanHandleApi<Api>,
{
    type Response = <__Context__ as CanHandleApi<Api>>::Response;
    async fn handle_api(
        __context__: &__Context__,
        api: PhantomData<Api>,
    ) -> Self::Response {
        __Context__::handle_api(__context__, api).await
    }
}
impl<__Context__, Api> IsProviderFor<ApiHandlerComponent, __Context__, (Api)>
for UseContext
where
    __Context__: CanHandleApi<Api>,
{}
impl<__Context__, Api, __Components__, __Path__> ApiHandler<__Context__, Api>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Api)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Api)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Api)>>::Output,
    >>::Delegate: ApiHandler<__Context__, Api>,
{
    type Response = <<__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Api)>>::Output,
    >>::Delegate as ApiHandler<__Context__, Api>>::Response;
    async fn handle_api(
        __context__: &__Context__,
        api: PhantomData<Api>,
    ) -> Self::Response {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Api)>>::Output,
        >>::Delegate::handle_api(__context__, api)
            .await
    }
}
impl<
    __Context__,
    Api,
    __Components__,
    __Path__,
> IsProviderFor<ApiHandlerComponent, __Context__, (Api)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Api)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Api)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Api)>>::Output,
    >>::Delegate: IsProviderFor<ApiHandlerComponent, __Context__, (Api)>
        + ApiHandler<__Context__, Api>,
{}
pub struct GreetApi;
impl<__Context__, Api> ApiHandler<__Context__, Api> for HandleGreet
where
    __Context__: HasName,
{
    type Response = String;
    async fn handle_api(__context__: &__Context__, _api: PhantomData<Api>) -> String {
        __context__.name().to_string()
    }
}
impl<__Context__, Api> IsProviderFor<ApiHandlerComponent, __Context__, (Api)>
for HandleGreet
where
    __Context__: HasName,
{}
pub struct HandleGreet;
pub struct App;
impl DelegateComponent<ApiHandlerComponent> for App {
    type Delegate = HandleGreet;
}
impl<__Context__, __Params__> IsProviderFor<ApiHandlerComponent, __Context__, __Params__>
for App
where
    HandleGreet: IsProviderFor<ApiHandlerComponent, __Context__, __Params__>,
{}
pub trait CanHandleApiSend<Api>: CanHandleApi<Api, Response: Send> + Send + Sync {
    fn handle_api_send(
        &self,
        api: PhantomData<Api>,
    ) -> impl Future<Output = Self::Response> + Send;
}
impl CanHandleApiSend<GreetApi> for App {
    fn handle_api_send(
        &self,
        api: PhantomData<GreetApi>,
    ) -> impl Future<Output = Self::Response> + Send {
        async move { self.handle_api(api).await }
    }
}
pub trait CanAddRoute<Ctx, Api> {
    fn add_route(self);
}
impl<Ctx, Api> CanAddRoute<Ctx, Api> for Box<Ctx>
where
    Ctx: CanHandleApiSend<Api>,
    Ctx::Response: Send,
{
    fn add_route(self) {}
}
pub trait CanAddAppRoutes: CanAddRoute<App, GreetApi> {}
impl CanAddAppRoutes for Box<App> {}
fn main() {}
