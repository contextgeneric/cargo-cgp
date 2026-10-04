//! The function-to-provider macros `#[cgp_computer]` and `#[cgp_producer]`, in
//! every shape their expansion distinguishes: a synchronous computer returning a
//! value and one returning `Result<T, E>`, an async computer returning each, a
//! generic computer, a computer taking a reference, and a producer. The program
//! compiles clean, so the snapshot of cargo-cgp's output is empty; the
//! `.expand.rs` pins the code each macro generates: the provider struct and impl,
//! its `IsProviderFor` impl, and the promotion wiring's `DelegateComponent`
//! impls.

use core::fmt::Display;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{Computer, Producer, TryComputer};
use cgp::prelude::*;

#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}

#[cgp_computer]
fn checked_add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "overflow".to_owned())
}

#[cgp_computer]
async fn add_async(a: u64, b: u64) -> u64 {
    a + b
}

#[cgp_computer]
async fn checked_add_async(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "overflow".to_owned())
}

#[cgp_computer]
fn add_generic<T: core::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

#[cgp_computer]
fn to_string_ref<Value: Display>(value: &Value) -> String {
    value.to_string()
}

#[cgp_producer]
fn magic_number() -> u64 {
    42
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}

fn main() {
    let code = PhantomData::<()>;

    assert_eq!(Add::compute(&App, code, (1, 2)), 3);
    assert_eq!(CheckedAdd::try_compute(&App, code, (1, 2)), Ok(3));
    assert_eq!(AddGeneric::compute(&App, code, (1, 2)), 3);
    assert_eq!(ToStringRef::compute(&App, code, &1), "1");
    assert_eq!(MagicNumber::produce(&App, code), 42);

    let _ = AddAsync::compute_async(&App, code, (1, 2));
    let _ = CheckedAddAsync::compute_async(&App, code, (1, 2));
}
