//! A call-site `E0277` whose input writes its type in every shape the call-site anchor reads.
//!
//! The program is `call_site_tuple_input`'s: `Branch<ReadName>` runs `ReadName` on the first
//! component of a tuple input, and `ReadName` reads a `name` field `App` lacks. Here every element
//! of the input writes its type: literals of each definite kind (`"text"`, `1u8`, `true`, `'c'`,
//! `b'a'`, `2.0f32`), a struct literal, a constant, a call to a non-generic function, and a
//! `PhantomData` whose type argument holds a tuple, a reference, a slice, and generic paths. The
//! anchor recovers the whole input type from the call, so the consumer trait in the header names
//! it in full.
//!
//! See cgp-knowledge-base/cgp/errors/hidden/unsatisfied-dependency.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E105 CGP-E106

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent};
use cgp::prelude::*;

/// A conditional program that runs its inner `Cond` on the first component of a tuple input.
pub struct Branch<Cond>(pub PhantomData<Cond>);

/// Inner program marker: read the context's `name`.
pub struct ReadName;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

/// Interpreter for `ReadName`: reads the context's `name` field — the buried root cause.
#[async_trait]
#[cgp_impl(new HandleReadName)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<ReadName, Input>
where
    Self: HasName,
    Input: Send,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<ReadName>, _input: Input) -> Result<String, Error> {
        Ok(self.name().to_owned())
    }
}

/// Interpreter for `Branch<Cond>`: destructures its input as a 2-tuple and runs `Cond` on the first
/// component. Its input *shape* is a tuple, so a call that does not type that tuple must still have
/// its structure recovered for this impl to match.
#[async_trait]
#[cgp_impl(new HandleBranch)]
#[use_type(HasErrorType.Error)]
impl<Cond, CondInput, Rest> Handler<Branch<Cond>, (CondInput, Rest)>
where
    Self: CanHandle<Cond, CondInput, Output = String>,
    Rest: Send,
{
    type Output = String;

    async fn handle(
        &self,
        _tag: PhantomData<Branch<Cond>>,
        (cond_input, _rest): (CondInput, Rest),
    ) -> Result<String, Error> {
        self.handle(PhantomData::<Cond>, cond_input).await
    }
}

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        @cgp.extra.handler.HandlerComponent.<Cond> Branch<Cond>:
            HandleBranch,

        @cgp.extra.handler.HandlerComponent.ReadName:
            HandleReadName,
    }
}

#[derive(HasField)]
pub struct App {
    // No `name` field — `HandleReadName`'s `Self: HasName` dependency cannot be met.
}

delegate_components! {
    App {
        namespace MyNamespace;
    }
}

pub struct Marker {
    pub id: u32,
}

pub const LIMIT: u32 = 3;

pub fn make_label() -> String {
    String::new()
}

async fn run_app(app: &App) -> Result<(), String> {
    // Every element of the input writes its own type: literals, a struct literal, a constant, a
    // non-generic call, and a `PhantomData` whose type argument holds a tuple, a reference, a slice,
    // and generic paths.
    app.handle(
        PhantomData::<Branch<ReadName>>,
        (
            ("text", 1u8, true, 'c', b'a', 2.0f32),
            (
                Marker { id: 0 },
                LIMIT,
                make_label(),
                PhantomData::<(u8, &'static [u16], Vec<String>)>,
            ),
        ),
    )
    .await?;
    Ok(())
}

fn main() {
    let _ = run_app;
}
