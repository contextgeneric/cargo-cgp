//! The emitter's output half: the real inner emitter and the diagnostics buffered for it.
//!
//! The emitter holds every diagnostic until its flush, so a crash in the compiler would print
//! rustc's crash report before the diagnostics that preceded it, and an unwinding flush could print
//! them only afterwards. To keep rustc's order, the panic hook must be able to emit the buffer
//! before the report, so this state lives behind a shared lock that the emitter registers for the
//! hook to reach ([`register_output`], [`flush_registered_output`]).

use std::cell::RefCell;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use rustc_errors::emitter::Emitter;

use crate::emitter::buffer::BufEntry;

/// The inner emitter and the buffered diagnostics, shared between the emitter and the panic hook.
pub(crate) struct Output<E> {
    /// The compiler's own emitter for the active error format, which renders every diagnostic.
    pub(crate) inner: E,
    /// Every diagnostic kept so far, in arrival order, held until the flush so separate failures
    /// sharing one root cause can be coalesced into one block. The compiler streams diagnostics one
    /// at a time with no end-of-compilation hook, so a headline listing every consumer one mistake
    /// breaks can only be built once they have all arrived, which is why the emitter flushes from
    /// its `Drop`. A coalescible [`BufEntry::Resolved`] joins the group of every failure it shares a
    /// root cause with; anything else is a [`BufEntry::Plain`] emitted at its original position.
    pub(crate) buffer: Vec<BufEntry>,
}

/// Lock `output`, recovering it if a panic poisoned the lock: the state stays usable, and the
/// diagnostics in it are still worth printing.
pub(crate) fn lock_output<T: ?Sized>(output: &Mutex<T>) -> MutexGuard<'_, T> {
    output.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the panic hook can do with a registered output without knowing its emitter type.
trait FlushOnCrash {
    /// Emit every buffered diagnostic as rustc built it, emptying the buffer.
    fn flush_on_crash(&mut self);
}

impl<E: Emitter> FlushOnCrash for Output<E> {
    fn flush_on_crash(&mut self) {
        // A crash leaves no room for the flush's own rendering, which could panic again and abort,
        // so every entry goes out untransformed.
        for entry in std::mem::take(&mut self.buffer) {
            self.inner.emit_diagnostic(entry.untransformed(None));
        }
    }
}

thread_local! {
    /// The output of the emitter installed on this thread, for the panic hook. The compiler emits
    /// diagnostics on the thread that installed the emitter, which is also where its panics occur.
    static REGISTERED: RefCell<Option<Weak<Mutex<dyn FlushOnCrash>>>> = const { RefCell::new(None) };
}

/// Register `output` as the one the panic hook flushes on this thread.
pub(crate) fn register_output<E: Emitter + 'static>(output: &Arc<Mutex<Output<E>>>) {
    let weak: Weak<Mutex<dyn FlushOnCrash>> = Arc::downgrade(output) as _;
    REGISTERED.with(|registered| *registered.borrow_mut() = Some(weak));
}

/// Emit the registered output's buffered diagnostics, untransformed. Called by the panic hook on a
/// crash, before rustc's report. Does nothing when no output is registered on this thread, or
/// when the panic struck while the output was locked, since the emitter is then mid-way through
/// writing it.
pub(crate) fn flush_registered_output() {
    let output = REGISTERED
        .try_with(|registered| registered.borrow().as_ref().and_then(Weak::upgrade))
        .ok()
        .flatten();
    let Some(output) = output else {
        return;
    };
    let mut guard = match output.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => return,
    };
    guard.flush_on_crash();
}
