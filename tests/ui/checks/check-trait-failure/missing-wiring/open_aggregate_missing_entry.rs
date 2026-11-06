//! A missing entry in an aggregate provider's own `open` table. The context
//! `App` opens `ComputerComponent` and routes the `Sink` code to the aggregate provider `ByteSink`,
//! which opens `ComputerComponent` in its own table and dispatches on the *input*, the component's
//! second parameter, with a two-segment `@ComputerComponent.<Code> Code.Bytes` key. `ByteSink` has no
//! entry for the `Digest` input the check asks for, so the root cause is the unmet
//! `ByteSink: DelegateComponent<@ComputerComponent.Sink.Digest>`.
//!
//! The tree names the table each lookup runs in. The first `[CGP-E103]` hop is the context's own
//! redirect, `redirect lookup to @ComputerComponent in App`; the second runs inside the aggregate
//! (`RedirectLookup<ByteSink, …>`) and reads `… in ByteSink`. The leaf is the `[CGP-E114]`
//! provider-table leaf, `missing entry for @ComputerComponent.Sink.Digest in provider
//! ByteSink`, so the reader adds the entry to `ByteSink` rather than to `App`.
//!
//! See cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E103, CGP-E114).
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E103 CGP-E114

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
