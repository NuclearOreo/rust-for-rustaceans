//! # Obvious — `#[must_use]`
//!
//! > Another small but useful tool in making interfaces obvious is the
//! > `# [must_use]` annotation. Add it to any type, trait, or function, and the
//! > compiler will issue a warning if the user's code receives an element of that
//! > type or trait, or calls that function, and does not explicitly handle it. You
//! > may already have seen this in the context of `Result` : if a function
//! > returns a `Result` and you do not assign its return value somewhere, you get
//! > a compiler warning.
//!
//! > Be careful not to overuse this annotation, though—add it only if the user is
//! > very likely to make a mistake if they are not using the return value.
//!
//! The annotation is a lint, not a rule of the type system: the unhandled value
//! still compiles and still runs. That half is what can be asserted here, and the
//! diagnostic it produces is quoted instead:
//!
//! ```text
//! warning: unused return value of `must_use::flush` that must be used
//!  --> src/main.rs:3:5
//!   |
//! 3 |     flush(&mut pending, &log);
//!   |     ^^^^^^^^^^^^^^^^^^^^^^^^^
//!   |
//!   = note: `#[warn(unused_must_use)]` on by default
//! ```
//!
//! The annotation is shown on all three things the book lists — a function, a
//! type, and a trait — because the warning is attached to the *value* the caller
//! receives, and each of those is a way of saying "this value matters".

use std::sync::{Arc, Mutex};

/// A record of what happened, shared between the value and the test.
pub type Log = Arc<Mutex<Vec<&'static str>>>;

// ---------------------------------------------------------------------------
// On a function
// ---------------------------------------------------------------------------

/// Delivers everything queued and reports how many lines went out.
///
/// The number is the only report the caller gets, so discarding it is exactly the
/// mistake `#[must_use]` is for — and discarding it is still legal.
#[must_use = "the number of flushed lines is the only report you get"]
pub fn flush(pending: &mut Vec<&'static str>, log: &Log) -> usize {
    let flushed = pending.len();
    for line in pending.drain(..) {
        log.lock().unwrap().push(line);
    }
    flushed
}

// ---------------------------------------------------------------------------
// On a type
// ---------------------------------------------------------------------------

/// A lease whose whole point is being held: dropping it ends the lease.
///
/// > the compiler will issue a warning if the user's code receives an element of
/// > that type … and does not explicitly handle it
#[must_use = "dropping the lease ends it"]
pub struct Lease {
    log: Log,
}

impl Lease {
    pub fn acquire(log: &Log) -> Self {
        log.lock().unwrap().push("acquired");
        Self { log: log.clone() }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.log.lock().unwrap().push("released");
    }
}

// ---------------------------------------------------------------------------
// On a trait
// ---------------------------------------------------------------------------

/// A connection that must be closed on purpose. The annotation on the *trait*
/// covers every value that implements it, including the `impl Handle` below.
#[must_use = "the connection must be closed"]
pub trait Handle {
    fn close(self);
}

/// A handle that records both ends of its life.
pub struct Session {
    log: Log,
}

impl Handle for Session {
    fn close(self) {
        self.log.lock().unwrap().push("closed");
    }
}

/// Returns an opaque `impl Handle`, so the annotation on the trait is what the
/// caller meets.
pub fn connect(log: &Log) -> impl Handle {
    log.lock().unwrap().push("connected");
    Session { log: log.clone() }
}

// ---------------------------------------------------------------------------
// What the annotation does, and what it does not
// ---------------------------------------------------------------------------

/// > the compiler will issue a warning if the user's code … does not explicitly
/// > handle it
///
/// Ignoring the value is a warning, not an error: the code below compiles, runs,
/// and delivers the lines. `#[allow(unused_must_use)]` is what a caller writes
/// when they mean to ignore the report — the same escape hatch `let _ =` gives,
/// spelled at the call site.
pub fn an_ignored_must_use_value_still_runs() {
    let log = Log::default();
    let mut pending = vec!["alpha", "beta"];

    // Would be `warning: unused return value … that must be used` without this.
    #[allow(unused_must_use)]
    flush(&mut pending, &log);

    // The work happened anyway: the lint cannot change behaviour.
    assert_eq!(*log.lock().unwrap(), ["alpha", "beta"]);
    assert!(pending.is_empty());

    // The two explicit forms: `let _ =` says "I know", binding says "I use it".
    let mut pending = vec!["gamma"];
    let flushed = flush(&mut pending, &log);
    assert_eq!(flushed, 1);
}

/// > add it only if the user is very likely to make a mistake if they are not
/// > using the return value
///
/// The type annotation earns its keep when the value *is* the point — a lease
/// that ends when it is dropped, a connection that must be closed:
///
/// - `let lease = Lease::acquire(&log);` keeps the lease alive to the end of the
///   scope, so `acquired` and `released` are far apart.
/// - `let _ = Lease::acquire(&log);` drops it immediately, so the two events are
///   adjacent — the caller who wrote that probably meant to hold it.
///
/// And the trait annotation covers the value returned by [`connect`], even though
/// `connect` itself carries no attribute.
pub fn a_must_use_type_and_trait_mark_the_value_as_the_point() {
    // Bound: held to the end of the statement's scope.
    let log = Log::default();
    {
        let _lease = Lease::acquire(&log);
        log.lock().unwrap().push("still holding");
    }
    assert_eq!(
        *log.lock().unwrap(),
        ["acquired", "still holding", "released"]
    );

    // Discarded: dropped at the end of the same statement.
    let log = Log::default();
    let _ = Lease::acquire(&log);
    log.lock().unwrap().push("after the lease");
    assert_eq!(
        *log.lock().unwrap(),
        ["acquired", "released", "after the lease"]
    );

    // The trait annotation reaches the returned `impl Handle`.
    let log = Log::default();
    connect(&log).close();
    assert_eq!(*log.lock().unwrap(), ["connected", "closed"]);

    let log = Log::default();
    let _ = connect(&log); // discarded: never closed, which is the mistake
    assert_eq!(*log.lock().unwrap(), ["connected"]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_ignored_must_use_value_still_runs_runs() {
        an_ignored_must_use_value_still_runs();
    }

    #[test]
    fn a_must_use_type_and_trait_mark_the_value_as_the_point_runs() {
        a_must_use_type_and_trait_mark_the_value_as_the_point();
    }
}
