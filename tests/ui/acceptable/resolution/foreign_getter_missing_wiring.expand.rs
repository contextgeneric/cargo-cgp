#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasCredentialType {
    type Credential;
}
impl<__Context__> HasCredentialType for __Context__
where
    __Context__: CredentialTypeProvider<__Context__>,
{
    type Credential = <__Context__ as CredentialTypeProvider<__Context__>>::Credential;
}
pub trait CredentialTypeProvider<
    __Context__,
>: IsProviderFor<CredentialTypeProviderComponent, __Context__, ()> {
    type Credential;
}
impl<__Provider__, __Context__> CredentialTypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<CredentialTypeProviderComponent>
        + IsProviderFor<CredentialTypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        CredentialTypeProviderComponent,
    >>::Delegate: CredentialTypeProvider<__Context__>,
{
    type Credential = <<__Provider__ as DelegateComponent<
        CredentialTypeProviderComponent,
    >>::Delegate as CredentialTypeProvider<__Context__>>::Credential;
}
pub struct CredentialTypeProviderComponent;
impl<__Context__> CredentialTypeProvider<__Context__> for UseContext
where
    __Context__: HasCredentialType,
{
    type Credential = <__Context__ as HasCredentialType>::Credential;
}
impl<__Context__> IsProviderFor<CredentialTypeProviderComponent, __Context__, ()>
for UseContext
where
    __Context__: HasCredentialType,
{}
impl<__Context__, __Components__, __Path__> CredentialTypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: CredentialTypeProvider<__Context__>,
{
    type Credential = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as CredentialTypeProvider<__Context__>>::Credential;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<CredentialTypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<CredentialTypeProviderComponent, __Context__, ()>
        + CredentialTypeProvider<__Context__>,
{}
impl<Credential, __Context__> CredentialTypeProvider<__Context__>
for UseType<Credential> {
    type Credential = Credential;
}
impl<
    Credential,
    __Context__,
> IsProviderFor<CredentialTypeProviderComponent, __Context__, ()>
for UseType<Credential> {}
impl<__Provider__, Credential, __Context__> CredentialTypeProvider<__Context__>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<
        __Context__,
        CredentialTypeProviderComponent,
        Type = Credential,
    >,
{
    type Credential = Credential;
}
impl<
    __Provider__,
    Credential,
    __Context__,
> IsProviderFor<CredentialTypeProviderComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<
        __Context__,
        CredentialTypeProviderComponent,
        Type = Credential,
    >,
{}
pub trait HasCredential<App>
where
    App: HasCredentialType,
{
    fn credential(&self) -> &Option<App::Credential>;
}
impl<__Context__, App> HasCredential<App> for __Context__
where
    App: HasCredentialType,
    __Context__: HasField<Symbol!("credential"), Value = Option<App::Credential>>,
{
    fn credential(&self) -> &Option<App::Credential> {
        self.get_field(::core::marker::PhantomData::<Symbol!("credential")>)
    }
}
pub trait CanAuthenticate<Request> {
    fn authenticate(&self, request: Request) -> bool;
}
impl<__Context__, Request> CanAuthenticate<Request> for __Context__
where
    __Context__: Authenticator<__Context__, Request>,
{
    fn authenticate(&self, request: Request) -> bool {
        __Context__::authenticate(self, request)
    }
}
pub trait Authenticator<
    __Context__,
    Request,
>: IsProviderFor<AuthenticatorComponent, __Context__, (Request)> {
    fn authenticate(__context__: &__Context__, request: Request) -> bool;
}
impl<__Provider__, __Context__, Request> Authenticator<__Context__, Request>
for __Provider__
where
    __Provider__: DelegateComponent<AuthenticatorComponent>
        + IsProviderFor<AuthenticatorComponent, __Context__, (Request)>,
    <__Provider__ as DelegateComponent<
        AuthenticatorComponent,
    >>::Delegate: Authenticator<__Context__, Request>,
{
    fn authenticate(__context__: &__Context__, request: Request) -> bool {
        <__Provider__ as DelegateComponent<
            AuthenticatorComponent,
        >>::Delegate::authenticate(__context__, request)
    }
}
pub struct AuthenticatorComponent;
impl<__Context__, Request> Authenticator<__Context__, Request> for UseContext
where
    __Context__: CanAuthenticate<Request>,
{
    fn authenticate(__context__: &__Context__, request: Request) -> bool {
        __Context__::authenticate(__context__, request)
    }
}
impl<__Context__, Request> IsProviderFor<AuthenticatorComponent, __Context__, (Request)>
for UseContext
where
    __Context__: CanAuthenticate<Request>,
{}
impl<__Context__, Request, __Components__, __Path__> Authenticator<__Context__, Request>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Request)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Request)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Request)>>::Output,
    >>::Delegate: Authenticator<__Context__, Request>,
{
    fn authenticate(__context__: &__Context__, request: Request) -> bool {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Request)>>::Output,
        >>::Delegate::authenticate(__context__, request)
    }
}
impl<
    __Context__,
    Request,
    __Components__,
    __Path__,
> IsProviderFor<AuthenticatorComponent, __Context__, (Request)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Request)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Request)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Request)>>::Output,
    >>::Delegate: IsProviderFor<AuthenticatorComponent, __Context__, (Request)>
        + Authenticator<__Context__, Request>,
{}
impl<__Context__, Request> Authenticator<__Context__, Request> for AcceptAll {
    fn authenticate(__context__: &__Context__, _request: Request) -> bool {
        true
    }
}
impl<__Context__, Request> IsProviderFor<AuthenticatorComponent, __Context__, (Request)>
for AcceptAll {}
pub struct AcceptAll;
impl<__Context__, Request, InAuth> Authenticator<__Context__, Request>
for RequireCredential<InAuth>
where
    Request: HasCredential<__Context__>,
    __Context__: HasCredentialType,
    InAuth: Authenticator<__Context__, Request>,
{
    fn authenticate(__context__: &__Context__, request: Request) -> bool {
        if request.credential().is_none() {
            return false;
        }
        InAuth::authenticate(__context__, request)
    }
}
impl<
    __Context__,
    Request,
    InAuth,
> IsProviderFor<AuthenticatorComponent, __Context__, (Request)>
for RequireCredential<InAuth>
where
    Request: HasCredential<__Context__>,
    __Context__: HasCredentialType,
    InAuth: IsProviderFor<AuthenticatorComponent, __Context__, (Request)>
        + Authenticator<__Context__, Request>,
{}
pub struct RequireCredential<InAuth>(pub ::core::marker::PhantomData<InAuth>);
pub struct LoginRequest {
    pub credential: Option<String>,
}
impl HasField<Symbol!("credential")> for LoginRequest {
    type Value = Option<String>;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("credential")>,
    ) -> &Self::Value {
        &self.credential
    }
}
impl HasFieldMut<Symbol!("credential")> for LoginRequest {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("credential")>,
    ) -> &mut Self::Value {
        &mut self.credential
    }
}
pub struct App;
impl DelegateComponent<AuthenticatorComponent> for App {
    type Delegate = RequireCredential<AcceptAll>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<AuthenticatorComponent, __Context__, __Params__> for App
where
    RequireCredential<
        AcceptAll,
    >: IsProviderFor<AuthenticatorComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<AuthenticatorComponent, LoginRequest> for App {}
fn main() {}
