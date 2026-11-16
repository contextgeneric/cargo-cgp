//! Holding the diagnostics rustc emits during its `analysis` query, and transforming them once the
//! query ends.
//!
//! rustc emits each diagnostic while holding its diagnostic context's lock, and the emitter's
//! transforms force compiler queries. A query that emits re-enters that lock, which panics on one
//! thread and blocks under the parallel front end. So while `analysis` runs, the emitter only queues
//! what arrives ([`defer`]), and a wrapper around the query ([`override_analysis`]) transforms the
//! queue once type checking has finished and no lock is held. Diagnostics emitted before or after
//! `analysis`, such as name resolution's, keep the streaming path. See
//! `cgp-knowledge-base/cargo-cgp/implementation/driver/program.md`.

use std::any::Any;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use rustc_data_structures::FatalErrorMarker;
use rustc_errors::DiagInner;
use rustc_middle::ty::TyCtxt;
use rustc_middle::util::Providers;
use rustc_session::Session;

use crate::emitter::output::lock_output;

/// The most batches the processing takes from the queue. Each batch after the first holds the
/// diagnostics that the previous batch's queries emitted; past the cap, the rest is kept
/// untransformed, so a transform whose queries keep emitting cannot loop.
pub const MAX_ROUNDS: usize = 8;

/// Whether `analysis` is running, so a diagnostic arriving now is queued rather than transformed.
static DEFERRING: AtomicBool = AtomicBool::new(false);

/// The diagnostics queued while `analysis` runs or while the processing holds the emitter, in
/// arrival order.
static QUEUE: Mutex<Vec<DiagInner>> = Mutex::new(Vec::new());

/// The `analysis` provider the compiler would have run, which the wrapper calls.
static ORIGINAL_ANALYSIS: OnceLock<for<'tcx> fn(TyCtxt<'tcx>, ())> = OnceLock::new();

/// What the processing can do with the emitter, without knowing its inner emitter's type.
pub trait DeferredTarget {
    /// Transform `diag` and buffer the result, as a diagnostic on the streaming path is.
    fn transform(&mut self, diag: DiagInner);

    /// Buffer `diag` as rustc built it.
    fn keep_untransformed(&mut self, diag: DiagInner);
}

thread_local! {
    /// The emitter the wrapper processes the queue with, registered on the thread that built it.
    static TARGET: RefCell<Option<Weak<Mutex<dyn DeferredTarget>>>> = const { RefCell::new(None) };
}

/// Register `target` as the emitter that processes the queue on this thread.
pub fn register_target<T: DeferredTarget + 'static>(target: &Arc<Mutex<T>>) {
    let weak: Weak<Mutex<dyn DeferredTarget>> = Arc::downgrade(target) as _;
    TARGET.with(|registered| *registered.borrow_mut() = Some(weak));
}

/// Whether a diagnostic arriving now must be queued, because `analysis` is running.
pub fn is_deferring() -> bool {
    DEFERRING.load(Ordering::SeqCst)
}

/// Queue `diag` for the processing.
pub fn defer(diag: DiagInner) {
    lock_output(&QUEUE).push(diag);
}

/// Take every queued diagnostic, in arrival order, leaving the queue empty.
pub fn take_queued() -> Vec<DiagInner> {
    std::mem::take(&mut *lock_output(&QUEUE))
}

/// Replace the `analysis` provider with one that processes the queue when the query ends. Passed
/// to the compiler as `Config::override_queries`.
pub fn override_analysis(_session: &Session, providers: &mut Providers) {
    let _ = ORIGINAL_ANALYSIS.set(providers.queries.analysis);
    providers.queries.analysis = analysis_then_transform;
}

/// Run the original `analysis` with the diagnostics queued, then transform the queue.
///
/// `analysis` ends a failing crate by raising a `FatalErrorMarker` itself once type checking has
/// reported an error, so the wrapper catches that unwind, transforms the queue while the compiler is
/// still usable, and resumes it. Any other unwind is a crash: the panic hook has printed the buffer
/// and the queue already, so it resumes at once. An unwind out of the processing, such as a fatal
/// error from a query a transform forced, is resumed in turn once the queue is buffered.
fn analysis_then_transform(tcx: TyCtxt<'_>, key: ()) {
    let original = *ORIGINAL_ANALYSIS
        .get()
        .expect("the original analysis is stored with the override");
    let Some(target) = registered_target() else {
        return original(tcx, key);
    };

    DEFERRING.store(true, Ordering::SeqCst);
    let result = panic::catch_unwind(AssertUnwindSafe(|| original(tcx, key)));
    DEFERRING.store(false, Ordering::SeqCst);
    let unwound = match result {
        Ok(()) => None,
        Err(payload) if payload.is::<FatalErrorMarker>() => Some(payload),
        Err(payload) => panic::resume_unwind(payload),
    };

    // A query a transform forces may emit; that diagnostic finds the emitter locked and is queued,
    // to be taken in the next batch.
    let stopped = {
        let mut target = lock_output(&target);
        drain_queue(take_queued, MAX_ROUNDS, |diag, handling| match handling {
            Handling::Transform => target.transform(diag),
            Handling::KeepUntransformed => target.keep_untransformed(diag),
        })
    };
    if let Some(payload) = unwound.or(stopped) {
        panic::resume_unwind(payload);
    }
}

/// The emitter registered on this thread, while it is alive.
fn registered_target() -> Option<Arc<Mutex<dyn DeferredTarget>>> {
    TARGET
        .try_with(|registered| registered.borrow().as_ref().and_then(Weak::upgrade))
        .ok()
        .flatten()
}

/// How [`drain_queue`] hands an item on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handling {
    /// Transform the item.
    Transform,
    /// Keep the item as it was queued.
    KeepUntransformed,
}

/// Hand every item `take` yields to `handle`, in the order taken: the first `max_rounds` batches to
/// be transformed, and the rest, which only items queued while transforming can fill, to be kept
/// untransformed.
///
/// An unwind out of a transform stops the transforms: that item, the rest of its batch, and every
/// later batch are kept untransformed, and the payload is returned for the caller to resume.
///
/// Postcondition: `take` is called until it yields an empty batch, and every item it yields is handed
/// to `handle` once, in the order taken, as `Transform` until the cap or an unwind and as
/// `KeepUntransformed` after it; the result is the unwind's payload, or `None`.
pub fn drain_queue<T: Clone>(
    mut take: impl FnMut() -> Vec<T>,
    max_rounds: usize,
    mut handle: impl FnMut(T, Handling),
) -> Option<Box<dyn Any + Send>> {
    for _ in 0..max_rounds {
        let batch = take();
        if batch.is_empty() {
            return None;
        }
        let mut rest = batch.into_iter();
        while let Some(item) = rest.next() {
            let kept = item.clone();
            let transformed =
                panic::catch_unwind(AssertUnwindSafe(|| handle(item, Handling::Transform)));
            if let Err(payload) = transformed {
                handle(kept, Handling::KeepUntransformed);
                rest.for_each(|item| handle(item, Handling::KeepUntransformed));
                keep_remaining(&mut take, &mut handle);
                return Some(payload);
            }
        }
    }
    keep_remaining(&mut take, &mut handle);
    None
}

/// Hand every item `take` still yields to `handle` as `KeepUntransformed`.
fn keep_remaining<T>(take: &mut impl FnMut() -> Vec<T>, handle: &mut impl FnMut(T, Handling)) {
    loop {
        let batch = take();
        if batch.is_empty() {
            return;
        }
        batch
            .into_iter()
            .for_each(|item| handle(item, Handling::KeepUntransformed));
    }
}
