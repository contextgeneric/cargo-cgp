//! A registration that code on any of the compiler's threads can look up.
//!
//! The emitter registers its output for the panic hook, and its transforming half for the wrapper
//! around rustc's `analysis` query. Under the parallel front end, a crash or that query can come on
//! any of rustc's threads, so a registration kept on the thread that installed the emitter would
//! not be found there. A [`Registry`] keeps it in a process-wide slot whenever rustc runs
//! thread-safe, and in a thread-local slot otherwise, when the compiler runs on the one thread that
//! registered it.

use std::cell::RefCell;
use std::sync::{Arc, Mutex, Weak};
use std::thread::LocalKey;

use rustc_data_structures::marker::{DynSend, DynSync};
use rustc_data_structures::sync::{FromDyn, check_dyn_thread_safe};

use crate::emitter::output::lock_output;

/// The thread-local slot of a [`Registry`].
pub type LocalSlot<T> = RefCell<Option<Weak<T>>>;

/// A weak registration of one value, found from any thread when rustc runs thread-safe.
///
/// The value is held weakly, so the registration ends when its owner drops it.
pub struct Registry<T: ?Sized + 'static> {
    /// The registration when rustc runs thread-safe. `FromDyn` makes a value whose type rustc
    /// shares between its threads (`DynSend`) sendable, and is built only in that mode.
    shared: Mutex<Option<FromDyn<Weak<T>>>>,
    /// The registration otherwise, on the thread that made it.
    local: &'static LocalKey<LocalSlot<T>>,
}

impl<T: ?Sized + DynSend + DynSync + 'static> Registry<T> {
    /// A registry with nothing registered, whose thread-local slot is `local`.
    pub const fn new(local: &'static LocalKey<LocalSlot<T>>) -> Self {
        Self {
            shared: Mutex::new(None),
            local,
        }
    }

    /// Register `value`, replacing any earlier registration.
    ///
    /// Precondition: rustc's thread-safe mode is set, as it is before the compiler builds its
    /// session, where the emitter registers itself.
    pub fn register(&self, value: &Arc<T>) {
        let weak = Arc::downgrade(value);
        match check_dyn_thread_safe() {
            Some(proof) => *lock_output(&self.shared) = Some(proof.derive(weak)),
            None => self.local.with(|slot| *slot.borrow_mut() = Some(weak)),
        }
    }

    /// The registered value, while its owner keeps it: from the process-wide slot, or from this
    /// thread's.
    pub fn get(&self) -> Option<Arc<T>> {
        let shared = lock_output(&self.shared)
            .as_ref()
            .and_then(|weak| weak.upgrade());
        shared.or_else(|| {
            self.local
                .try_with(|slot| slot.borrow().as_ref().and_then(Weak::upgrade))
                .ok()
                .flatten()
        })
    }
}
