//! Tests for the registration the panic hook and the `analysis` wrapper look up
//! ([`cargo_cgp_driver::emitter::Registry`]).
//!
//! rustc's thread-safe mode is set once per process, so this binary runs these tests in that mode,
//! the parallel front end's; the one-thread path, the thread-local slot, runs in every fixture of
//! the UI suite's one-thread run, which passes `-Zthreads=1`.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread;

use cargo_cgp_driver::emitter::{LocalSlot, Registry};
use rustc_data_structures::sync::set_dyn_thread_safe_mode;

thread_local! {
    static LOCAL: LocalSlot<Mutex<u32>> = const { RefCell::new(None) };
}

static REGISTRY: Registry<Mutex<u32>> = Registry::new(&LOCAL);

/// The registered value, read on a new thread.
fn read_on_another_thread() -> Option<u32> {
    thread::spawn(|| REGISTRY.get().map(|value| *value.lock().unwrap()))
        .join()
        .unwrap()
}

// Concern: under the parallel front end a crash, or the `analysis` query, can come on a worker
//   thread, where a registration kept on the thread that made it is not found.
// Handled: while rustc runs thread-safe, the registry keeps the registration process-wide.
// Property: for all values registered while rustc runs thread-safe, `get` on another thread
//   returns the latest registered value while its owner keeps it, and nothing once the owner drops
//   it.
#[test]
fn a_registration_is_found_from_another_thread_while_its_owner_keeps_it() {
    set_dyn_thread_safe_mode(true);
    let first = Arc::new(Mutex::new(7));
    REGISTRY.register(&first);
    assert_eq!(
        read_on_another_thread(),
        Some(7),
        "found from another thread"
    );

    let second = Arc::new(Mutex::new(8));
    REGISTRY.register(&second);
    assert_eq!(
        read_on_another_thread(),
        Some(8),
        "a new registration replaces the old"
    );

    drop(second);
    assert_eq!(
        read_on_another_thread(),
        None,
        "the registration ends with its owner"
    );
    drop(first);
}
