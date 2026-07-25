#![feature(prelude_import)]
//! Acceptable: a raw error that never names a CGP construct, traced back to the one CGP cause.
//! `App` lacks the `name` field `HandleGreet` needs. Besides the clean `check_components!` failure
//! (`CanHandleApi`), the hand-written `impl CanHandleApiSend<GreetApi> for App` — the transfer
//! example's `Send`-recovery shape, forwarding to the wired `handle_api` — makes the same failure
//! resurface as an `E0271` opaque-future type mismatch mentioning only `CanHandleApiSend`/`Future`,
//! no CGP trait. The resolver anchors it on the enclosing hand-written impl (whose `CanHandleApi`
//! supertrait is a CGP consumer trait), traces the chain to the same missing-field root cause, and
//! heads the tree with the wrapper the programmer wrote: `CanHandleApiSend`, then its `CanHandleApi`
//! supertrait, down to the missing field. The header reads `[CGP-E009] the trait
//! \`CanHandleApiSend<GreetApi>\`` — a wrapper is a plain trait, not a CGP consumer — with the
//! `E0271` code kept. Being a distinct trait from the check entry's `CanHandleApi`, it is its own
//! block. The `.rust.stderr` baseline shows the full un-traced cascade for contrast.
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
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ApiHandlerComponent, GreetApi> for App {}
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
fn main() {}
