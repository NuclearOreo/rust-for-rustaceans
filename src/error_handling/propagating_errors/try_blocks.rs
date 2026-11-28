//! # Propagating Errors — `Try` and try blocks
//!
//! > The second aspect of `?` to be aware of is that this operator is really just
//! > syntax sugar for a trait tentatively called `Try` . At the time of writing,
//! > the `Try` trait has not yet been stabilized, but by the time you read this,
//! > it's likely that it, or something very similar, will have been settled on.
//!
//! > At its heart, `Try` defines a wrapper type whose state is either one where
//! > further computation is useful (the happy path), or one where it is not.
//!
//! > An `Option<T>` , for example, follows the same pattern … This pattern
//! > extends to more complex types, like `Poll<Result<T, E>>` , whose happy path
//! > type is `Poll<T>` , which makes `?` apply in far more cases than you might
//! > expect.
//!
//! On rustc 1.100 both halves of the promise are still unfulfilled:
//! `std::ops::Try` is `E0658` (`try_trait_v2`), and `try { }` is `E0658`
//! (`try_blocks`). Naming the trait at all is the rejection:
//!
//! ```compile_fail,E0658
//! use std::ops::Try;   // `try_trait_v2` is unstable
//! ```
//!
//! And Listing 4-3's syntax is the other one:
//!
//! ```compile_fail,E0658
//! fn do_the_thing() -> Result<(), ()> {
//!     let thing = Ok::<(), ()>(());
//!     let result = try {
//!         // .. code that uses `thing` and `?` ..
//!     };
//!     thing
//! }
//! ```
//!
//! A custom wrapper needs `Try` too, so `?` on one is `E0277` — the compiler
//! even names the nightly-only trait in its help:
//!
//! ```compile_fail,E0277
//! struct MyResult(u8);
//!
//! fn twice(value: MyResult) -> MyResult {
//!     let _inner = value?;   // the `?` operator can only be applied to
//!                            // values that implement `Try`
//!     value
//! }
//! ```
//!
//! What *is* stable is the operator's behaviour on the two stabilized
//! implementors — `?` on `Result` and on `Option`, both in
//! [`question_mark`](super::question_mark) — and the scoping problem try blocks
//! were designed for, which is the demo here.
//!
//! > This won't quite work as expected. Any `?` between `setup` and `cleanup`
//! > will cause an early return from the entire function, which would skip the
//! > cleanup code! This is the problem _try blocks_ are intended to solve.
//!
//! > A try block acts pretty much like a single-iteration loop, where `?` uses
//! > `break` instead of `return` , and the final expression of the block has an
//! > implicit `break` .
//!
//! Listing 4-2 is [`do_the_thing`], which really does skip its cleanup. Listing
//! 4-3 is the try block, which is a `compile_fail,E0658` doctest; the runnable
//! equivalents are a closure (the scope is the closure body) and a labelled
//! block (the scope is the block, and `break` is the early exit).

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

/// The per-demo log the book's `Thing` records itself into.
pub type Log = Rc<RefCell<Vec<&'static str>>>;

/// The error a step reports.
#[derive(Debug, PartialEq, Eq)]
pub struct StepFailed;

impl fmt::Display for StepFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a step of the thing failed")
    }
}

impl Error for StepFailed {}

/// The book's `Thing`: setup, a fallible step, and cleanup that must run.
pub struct Thing {
    log: Log,
}

impl Thing {
    /// Sets the value up and records it.
    ///
    /// > consider the function in Listing 4-2
    ///
    /// # Errors
    ///
    /// Never in this demo, but the signature is fallible like the book's.
    pub fn setup(log: &Log) -> Result<Self, StepFailed> {
        log.borrow_mut().push("setup");
        Ok(Self {
            log: Rc::clone(log),
        })
    }

    /// The work that can fail.
    ///
    /// # Errors
    ///
    /// Always, so that the cleanup path is the one under test.
    pub fn step(&self) -> Result<(), StepFailed> {
        self.log.borrow_mut().push("step");
        Err(StepFailed)
    }

    /// Releases the value. Must run even when a step failed.
    pub fn cleanup(&self) {
        self.log.borrow_mut().push("cleanup");
    }
}

// ---------------------------------------------------------------------------
// Listing 4-2, and the two stable spellings of Listing 4-3
// ---------------------------------------------------------------------------

/// Listing 4-2: a multi-step fallible function whose `?` skips `cleanup`.
///
/// # Errors
///
/// [`StepFailed`] from the step.
pub fn do_the_thing(log: &Log) -> Result<(), StepFailed> {
    let thing = Thing::setup(log)?;
    thing.step()?;
    // Unreachable when the step fails: `?` returned from the whole function.
    thing.cleanup();
    Ok(())
}

/// Listing 4-3 with a closure as the scope: `?` returns from the closure, so
/// `cleanup` still runs.
///
/// # Errors
///
/// [`StepFailed`] from the step, passed through after cleanup.
// The closure call *is* the try block. `redundant_closure_call` would inline it,
// which is exactly the change that breaks the scoping.
#[allow(clippy::redundant_closure_call)]
pub fn do_the_thing_scoped(log: &Log) -> Result<(), StepFailed> {
    let thing = Thing::setup(log)?;
    let result = (|| -> Result<(), StepFailed> {
        thing.step()?;
        Ok(())
    })();
    thing.cleanup();
    result
}

/// The same scoping with a labelled block, where `break` is the try block's
/// early exit and the block's final expression is its implicit `break`.
///
/// # Errors
///
/// [`StepFailed`] from the step, after cleanup.
pub fn do_the_thing_labelled(log: &Log) -> Result<(), StepFailed> {
    let thing = Thing::setup(log)?;
    let result: Result<(), StepFailed> = 'scoped: {
        if let Err(error) = thing.step() {
            break 'scoped Err(error);
        }
        Ok(())
    };
    thing.cleanup();
    result
}

// ---------------------------------------------------------------------------
// Demos
// ---------------------------------------------------------------------------

/// > Any `?` between `setup` and `cleanup` will cause an early return from the
/// > entire function, which would skip the cleanup code!
///
/// The log is the evidence: the broken version never records `cleanup`; both
/// scoped versions do.
pub fn a_try_block_is_what_keeps_cleanup_on_the_error_path() {
    let log = Log::default();
    assert_eq!(do_the_thing(&log), Err(StepFailed));
    assert_eq!(
        *log.borrow(),
        ["setup", "step"],
        "Listing 4-2 skipped cleanup"
    );

    let log = Log::default();
    assert_eq!(do_the_thing_scoped(&log), Err(StepFailed));
    assert_eq!(*log.borrow(), ["setup", "step", "cleanup"]);

    let log = Log::default();
    assert_eq!(do_the_thing_labelled(&log), Err(StepFailed));
    assert_eq!(*log.borrow(), ["setup", "step", "cleanup"]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_try_block_is_what_keeps_cleanup_on_the_error_path_runs() {
        a_try_block_is_what_keeps_cleanup_on_the_error_path();
    }
}
