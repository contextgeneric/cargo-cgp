//! Acceptable failure: a getter on a *request* type resolves to the context's missing
//! wiring, not the opaque getter bound. The higher-order provider `RequireCredential`
//! reaches the context's credential type two ways — directly (`Self: HasCredentialType`, via
//! `#[uses]`) and through a getter on the request (`LoginRequest: HasCredential<Self>`,
//! whose `#[cgp_auto_getter]` blanket impl needs `App: HasCredentialType` because its return
//! type names `App::Credential`). Leaving `CredentialTypeProviderComponent` unwired is the
//! single mistake. The resolver descends the getter's blanket impl into its context-side
//! dependency, so the getter branch bottoms out on the same missing wiring as the direct
//! branch and the two collapse to one root cause under a `CGP-E001` header — rather than the
//! getter branch stopping at the bare foreign bound `LoginRequest: HasCredential<App>` and
//! reporting it as a second, misleading root cause (and leaking it as the header).
//!
//! This is the shape the transfer example's `UseBasicAuth` provider produces:
//! `Request: HasBasicAuthHeader<MockApp>` beside the real missing password-type wiring.

use cgp::prelude::*;

// The abstract credential type — deliberately left unwired on `App`, the one mistake.
#[cgp_type]
pub trait HasCredentialType {
    type Credential;
}

// A getter on the *request* type (not the context), so it must be a getter trait rather
// than an implicit argument. Its `#[cgp_auto_getter]` blanket impl requires the context to
// supply the credential type, because the returned value names `App::Credential`.
#[cgp_auto_getter]
pub trait HasCredential<App>
where
    App: HasCredentialType,
{
    fn credential(&self) -> &Option<App::Credential>;
}

#[cgp_component(Authenticator)]
pub trait CanAuthenticate<Request> {
    fn authenticate(&self, request: Request) -> bool;
}

// The inner endpoint: trivially accepts every request.
#[cgp_impl(new AcceptAll)]
impl<Request> Authenticator<Request> {
    fn authenticate(&self, _request: Request) -> bool {
        true
    }
}

// A higher-order wrapper (like the transfer example's `UseBasicAuth`): it reads the
// request's credential, then delegates to the inner handler. Its dependencies reach the
// context's credential type two ways — directly (`#[uses(HasCredentialType)]`) and through
// the request getter (`Request: HasCredential<Self>`) — so a single unwired type surfaces
// twice.
#[cgp_impl(new RequireCredential<InAuth>)]
#[uses(HasCredentialType)]
#[use_provider(InAuth: Authenticator<Request>)]
impl<Request, InAuth> Authenticator<Request>
where
    Request: HasCredential<Self>,
{
    fn authenticate(&self, request: Request) -> bool {
        if request.credential().is_none() {
            return false;
        }
        InAuth::authenticate(self, request)
    }
}

#[derive(HasField)]
pub struct LoginRequest {
    pub credential: Option<String>,
}

pub struct App;

delegate_components! {
    App {
        AuthenticatorComponent: RequireCredential<AcceptAll>,
    }
}

check_components! {
    App {
        AuthenticatorComponent: LoginRequest,
    }
}

fn main() {}
