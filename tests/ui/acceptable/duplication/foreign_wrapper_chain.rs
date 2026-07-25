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

// A `Send`-recovery wrapper carrying the CGP consumer trait `CanHandleApi` as a supertrait,
// implemented directly on the context — the transfer example's `CanHandleApiSend` shape.
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

// A generic routing trait whose blanket impl for the *foreign* wrapper `Box<Ctx>` depends on the
// wrapper capability on `Ctx` and on its associated `Response` type — the `impl CanAddRoute<App, ..>
// for Router<Arc<App>>` shape. `Ctx` appears only as a type argument, never as the impl's `Self`.
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

// The convenience-alias impl the programmer writes, binding the routing to the concrete context —
// the `impl CanAddApiRoutes for Router<Arc<MockApp>>` shape. The caret lands here; its supertrait
// `Box<App>: CanAddRoute<App, GreetApi>` fails several hops from the real cause.
pub trait CanAddAppRoutes: CanAddRoute<App, GreetApi> {}

impl CanAddAppRoutes for Box<App> {}

fn main() {}
