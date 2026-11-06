//! Two blanket impls of one trait that differ only in a projected argument, and the walk choosing
//! between them. `Pick<<u8 as Assoc>::Out>` is `Pick<A>` and `Pick<<u16 as Assoc>::Out>` is
//! `Pick<B>`, so the two impls are coherent, and each requires a different field. The provider needs
//! `Self: Pick<B>`, and the context has neither field, so the root cause is the missing `beta`
//! that the second impl requires.
//!
//! Property: the walk matches an obligation against an impl only where the impl's trait ref, its
//! projections reduced, equals the obligation's. An impl matched with its projection unreduced
//! relates `<u8 as Assoc>::Out` to `B` as a goal left for later, so the first impl looks like a
//! match and the cause would name `alpha`, the field the first impl requires, which `Pick<B>` does
//! not need.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

use cgp::prelude::*;

pub struct A;
pub struct B;

pub trait Assoc {
    type Out;
}

impl Assoc for u8 {
    type Out = A;
}

impl Assoc for u16 {
    type Out = B;
}

pub trait Pick<T> {}

impl<Ctx> Pick<<u8 as Assoc>::Out> for Ctx where Ctx: HasField<Symbol!("alpha"), Value = u32> {}
impl<Ctx> Pick<<u16 as Assoc>::Out> for Ctx where Ctx: HasField<Symbol!("beta"), Value = u32> {}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetPicked)]
#[uses(Pick<B>)]
impl Greeter {
    fn greet(&self) {}
}

#[derive(HasField)]
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        GreeterComponent: GreetPicked,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
