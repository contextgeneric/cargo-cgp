//! The emitter's output half: the real inner emitter and the diagnostics buffered for it.
//!
//! The emitter holds every diagnostic until its flush, so a crash in the compiler would print
//! rustc's crash report before the diagnostics that preceded it, and an unwinding flush could print
//! them only afterwards. To keep rustc's order, the panic hook must be able to emit the buffer
//! before the report, so this state lives behind a shared lock that the emitter registers for the
//! hook to reach ([`register_output`], [`flush_registered_output`]). The pass that reshapes the
//! name-resolution errors after resolution reaches the buffer the same way
//! ([`with_registered_buffer`]), and records there the requirements the emitter's overflow stage
//! reads.

use std::cell::RefCell;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use rustc_errors::emitter::Emitter;

use crate::emitter::buffer::BufEntry;

/// The inner emitter and the buffered diagnostics, shared between the emitter and the panic hook.
pub struct Output<E> {
    /// The compiler's own emitter for the active error format, which renders every diagnostic.
    pub inner: E,
    /// Every diagnostic kept so far, in arrival order, held until the flush so separate failures
    /// sharing one root cause can be coalesced into one block. The compiler streams diagnostics one
    /// at a time with no end-of-compilation hook, so a headline listing every consumer one mistake
    /// breaks can only be built once they have all arrived, which is why the emitter flushes from
    /// its `Drop`. A coalescible [`BufEntry::Resolved`] joins the group of every failure it shares a
    /// root cause with; anything else is a [`BufEntry::Plain`] emitted at its original position.
    pub buffer: Vec<BufEntry>,
    /// The requirements that wiring entries with an unresolved value leave unprovable, recorded by
    /// the name-resolution pass after expansion, so the emitter drops a later `E0275` overflow on
    /// one as that entry's consequence.
    pub unprovable_requirements: Vec<String>,
}

/// Lock `output`, recovering it if a panic poisoned the lock: the state stays usable, and the
/// diagnostics in it are still worth printing.
pub fn lock_output<T: ?Sized>(output: &Mutex<T>) -> MutexGuard<'_, T> {
    output.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the panic hook and the name-resolution pass can do with a registered output without knowing
/// its emitter type.
trait RegisteredOutput {
    /// Emit every buffered diagnostic as rustc built it, emptying the buffer.
    fn flush_on_crash(&mut self);

    /// The buffered diagnostics, to reshape in place.
    fn buffer_mut(&mut self) -> &mut Vec<BufEntry>;

    /// The requirements recorded as unprovable, to add to.
    fn unprovable_requirements_mut(&mut self) -> &mut Vec<String>;
}

impl<E: Emitter> RegisteredOutput for Output<E> {
    fn flush_on_crash(&mut self) {
        // A crash leaves no room for the flush's own rendering, which could panic again and abort,
        // so every entry goes out untransformed.
        for entry in std::mem::take(&mut self.buffer) {
            self.inner.emit_diagnostic(entry.untransformed(None));
        }
    }

    fn buffer_mut(&mut self) -> &mut Vec<BufEntry> {
        &mut self.buffer
    }

    fn unprovable_requirements_mut(&mut self) -> &mut Vec<String> {
        &mut self.unprovable_requirements
    }
}

thread_local! {
    /// The output of the emitter installed on this thread, for the panic hook. The compiler emits
    /// diagnostics on the thread that installed the emitter, which is also where its panics occur.
    static REGISTERED: RefCell<Option<Weak<Mutex<dyn RegisteredOutput>>>> = const { RefCell::new(None) };
}

/// Register `output` as the one the panic hook flushes on this thread.
pub fn register_output<E: Emitter + 'static>(output: &Arc<Mutex<Output<E>>>) {
    let weak: Weak<Mutex<dyn RegisteredOutput>> = Arc::downgrade(output) as _;
    REGISTERED.with(|registered| *registered.borrow_mut() = Some(weak));
}

/// Emit the registered output's buffered diagnostics, untransformed. Called by the panic hook on a
/// crash, before rustc's report. Does nothing when no output is registered on this thread, or
/// when the panic struck while the output was locked, since the emitter is then mid-way through
/// writing it.
pub fn flush_registered_output() {
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

/// Run `f` on the buffered diagnostics of the output registered on this thread, under its lock, or
/// return `None` when no output is registered. `f` must not run compiler queries: one that emits a
/// diagnostic would try to take the same lock.
pub fn with_registered_buffer<R>(f: impl FnOnce(&mut Vec<BufEntry>) -> R) -> Option<R> {
    with_registered_output(|output| f(output.buffer_mut()))
}

/// Add `requirements` to the unprovable requirements of the output registered on this thread, if
/// one is registered.
pub fn record_unprovable_requirements(requirements: Vec<String>) {
    with_registered_output(|output| output.unprovable_requirements_mut().extend(requirements));
}

/// Run `f` on the output registered on this thread, under its lock, or return `None` when no output
/// is registered.
fn with_registered_output<R>(f: impl FnOnce(&mut dyn RegisteredOutput) -> R) -> Option<R> {
    let output = REGISTERED
        .try_with(|registered| registered.borrow().as_ref().and_then(Weak::upgrade))
        .ok()
        .flatten()?;
    let mut guard = lock_output(&output);
    Some(f(&mut *guard))
}
