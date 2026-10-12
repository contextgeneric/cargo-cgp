//! A raw error that never names a CGP construct, traced back to the one CGP cause.
//! `App` lacks the `name` field `HandleGreet` needs. Besides the clean `check_components!` failure
//! (`CanHandleApi`), the hand-written `impl CanHandleApiSend<GreetApi> for App` — the transfer
//! example's `Send`-recovery shape, forwarding to the wired `handle_api` — makes the same failure
//! resurface as an `E0271` opaque-future type mismatch mentioning only `CanHandleApiSend`/`Future`,
//! no CGP trait. The resolver anchors it on the enclosing hand-written impl (whose `CanHandleApi`
//! supertrait is a CGP consumer trait), traces the chain to the same missing-field root cause, and
//! heads the tree with the wrapper the programmer wrote: `CanHandleApiSend`, then its `CanHandleApi`
//! supertrait, down to the missing field. Both failures share that cause, so they coalesce into one
//! block headed by the cause, with a caret at each site: the check entry comes first in the source,
//! so the block takes its `[CGP-E001]` class and its `E0277` code, and the wrapper's chain, which
//! runs through `CanHandleApi`, shows both routes in one tree. The `.rust.stderr` baseline shows the
//! full un-traced cascade for contrast.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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
