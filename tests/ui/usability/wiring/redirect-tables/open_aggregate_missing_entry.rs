//! Usability failure: a missing entry in an aggregate provider's own `open` table is reported as if
//! it were missing from the context. The context `App` opens `ComputerComponent` and routes the
//! `Sink` code to the aggregate provider `ByteSink`, which opens `ComputerComponent` in its own
//! table and dispatches on the *input*, the component's second parameter, with a two-segment
//! `@ComputerComponent.<Code> Code.Bytes` key. `ByteSink` has no entry for the `Digest` input the
//! check asks for, so the real root cause is the unmet
//! `ByteSink: DelegateComponent<@ComputerComponent.Sink.Digest>`.
//!
//! The root cause is present, and the path is right, but two labels misname the table. The second
//! `[CGP-E104]` redirect hop, the one that runs inside `ByteSink` (`RedirectLookup<ByteSink, …>`),
//! is rendered `redirect lookup to @ComputerComponent in App`, because the hop takes the walk's
//! context rather than the redirect's own table. And the `[CGP-E107]` leaf names the right owner
//! but calls it a context: `context ByteSink does not contain any delegate entry for …`, where a
//! provider table missing a key is otherwise the `[CGP-E110]` "provider … does not contain" leaf.
//! A reader cannot tell from the tree which table to add the entry to without knowing that the
//! second hop is the aggregate's.
//!
//! See cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::extra::handler::{Computer, ComputerComponent};
use cgp::prelude::*;

/// The code that routes a computation to the sink aggregate.
pub struct Sink;

/// An input the sink handles, and one it does not.
pub struct Bytes(pub Vec<u8>);
pub struct Digest(pub [u8; 4]);

#[cgp_impl(new WriteBytes)]
impl<Code> Computer<Code, Bytes> {
    type Output = usize;

    fn compute(&self, _code: PhantomData<Code>, input: Bytes) -> usize {
        input.0.len()
    }
}

// The aggregate provider dispatches on the input through its own `open` table.
delegate_components! {
    new ByteSink {
        open ComputerComponent;

        @ComputerComponent.<Code> Code.Bytes: WriteBytes,
    }
}

pub struct App;

// The context routes the `Sink` code, with any input, to the aggregate.
delegate_components! {
    App {
        open ComputerComponent;

        @ComputerComponent.Sink: ByteSink,
    }
}

check_components! {
    App {
        ComputerComponent: (Sink, Digest),
    }
}

fn main() {}
