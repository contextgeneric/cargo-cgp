//! A per-entry generic on a `delegate_components!` mapping that appears only in the provider, not
//! in the key.
//!
//! The macro lowers `<T> GreeterComponent: GreetWith<T>` to
//! `impl<T> DelegateComponent<GreeterComponent> for Person { type Delegate = GreetWith<T>; }` and
//! its `IsProviderFor` twin, where neither the trait, the self type, nor a predicate binds `T`, so
//! the compiler rejects `T` with `E0207` once per impl. Its two suggestions contradict each other,
//! and neither compiles: adding `T` to `Person` leaves the struct's parameter unused, and removing
//! `<T>` leaves `GreetWith<T>` naming an undeclared type. The tool reports one `[CGP-E019]` naming
//! the entry, with a help to write a concrete type in place of `T` and remove it from the entry's
//! generic parameters, which compiles.
//!
//! A per-entry generic is well formed only when it reaches the key, as in
//! `<T2> BazKey<T1, T2>: BarValue<T1>`, where `DelegateComponent<BazKey<..>>` binds it. The macro
//! lowers an entry whose generic never reaches the key as written, so the compiler rejects it as it
//! would a hand-written impl with an unused parameter; this is not a macro defect.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).

//@quality: acceptable
//@codes: CGP-E019

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct GreetWith<T>(pub PhantomData<T>);

#[cgp_provider]
impl<Context, T> Greeter<Context> for GreetWith<T> {
    fn greet(_context: &Context) {}
}

pub struct Person;

// `T` is declared for the entry but only used in the value `GreetWith<T>`.
delegate_components! {
    Person {
        <T> GreeterComponent: GreetWith<T>,
    }
}

fn main() {}
