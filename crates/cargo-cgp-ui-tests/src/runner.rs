//! Scheduling fixtures across a pool of workers, each with its own throwaway crate, reporting each
//! result through a `done` callback in completion order; see
//! `cgp-knowledge-base/cargo-cgp/implementation/testing.md`.

use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// The most workers to start by default. Capped low because each worker rebuilds cgp in
/// its own target directory, so past a handful the extra `cgp` builds cost more in
/// compilation and disk than the added parallelism saves; `--jobs` overrides it upward
/// for a machine that can afford more.
const DEFAULT_MAX_JOBS: usize = 8;

/// The default worker count when `--jobs` is not given: the machine's parallelism, but
/// never more than `DEFAULT_MAX_JOBS` and never more workers than there are fixtures
/// (extra workers would only idle and rebuild cgp for nothing). Always at least one.
pub fn default_jobs(fixture_count: usize) -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(DEFAULT_MAX_JOBS)
        .min(fixture_count)
        .max(1)
}

/// Run `task` over every fixture across `jobs` workers, calling `done` with each result as
/// soon as it is ready. Worker `w` (in `0..jobs`) is passed to `task` alongside the
/// fixture so the task can address that worker's own crate; the same worker index recurs
/// for every fixture that worker picks up, so its crate stays warm across them. `done` is
/// serialized (the runner never calls it from two workers at once), so it can print
/// without interleaving; the results arrive in completion order.
pub fn run<T, Task, Done>(jobs: usize, fixtures: &[&Path], task: Task, done: Done)
where
    Task: Fn(usize, &Path) -> T + Sync,
    Done: Fn(T) + Sync,
{
    let next = AtomicUsize::new(0);
    let done_lock = Mutex::new(());

    thread::scope(|scope| {
        for worker in 0..jobs {
            let next = &next;
            let task = &task;
            let done = &done;
            let done_lock = &done_lock;
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(fixture) = fixtures.get(index) else {
                        break;
                    };
                    let result = task(worker, fixture);
                    let _guard = done_lock.lock().unwrap();
                    done(result);
                }
            });
        }
    });
}
