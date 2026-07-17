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

use core::future::Future;
use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(ApiHandler)]
#[async_trait]
pub trait CanHandleApi<Api> {
    type Response;

    async fn handle_api(&self, api: PhantomData<Api>) -> Self::Response;
}

pub struct GreetApi;

#[cgp_impl(new HandleGreet)]
#[uses(HasName)]
impl<Api> ApiHandler<Api> {
    type Response = String;

    async fn handle_api(&self, _api: PhantomData<Api>) -> String {
        self.name().to_string()
    }
}

// The single mistake: `App` never carries the `name` field `HandleGreet` reads.
pub struct App;

delegate_components! {
    App {
        ApiHandlerComponent: HandleGreet,
    }
}

check_components! {
    App {
        ApiHandlerComponent: GreetApi,
    }
}

// The `Send`-recovery workaround (the transfer example's `CanHandleApiSend`). Written with an
// explicit `-> impl Future` and an `async move` block rather than an `async fn`, so the forwarding
// failure surfaces as an `E0271` on the returned future rather than an opaque-type cycle.
pub trait CanHandleApiSend<Api>: CanHandleApi<Api, Response: Send> + Send + Sync {
    fn handle_api_send(&self, api: PhantomData<Api>)
    -> impl Future<Output = Self::Response> + Send;
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
