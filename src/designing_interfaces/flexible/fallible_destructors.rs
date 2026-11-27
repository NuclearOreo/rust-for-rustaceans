//! # Fallible and Blocking Destructors
//!
//! > Types centered on I/O often need to perform cleanup when they're dropped.
//! > This may include flushing writes to disk, closing files, or gracefully
//! > terminating connections to remote hosts. The natural place to perform this
//! > cleanup is in the type's `Drop` implementation. Unfortunately, once a value
//! > is dropped, we no longer have a way to communicate errors to the user except
//! > by panicking.
//!
//! > For that reason, we need to provide best-effort cleanup through `Drop` . If
//! > cleanup errors, at least we tried—we swallow the error and move on.
//!
//! > However, we ought to provide a better alternative for users who wish to
//! > leave no loose threads. We can do this by providing an explicit destructor.
//! > This usually takes the form of a method that takes ownership of `self` and
//! > exposes any errors (using `-> Result<_, _>` ) or asynchrony (using `async
//! > fn` ) that are inherent to the destruction.
//!
//! > **NOTE** _Make sure you highlight the explicit destructor in your
//! > documentation!_
//!
//! Two types show the shape, and the `compile_fail` doctests show why it cannot
//! be written the obvious way:
//!
//! | Type | Explicit destructor | Goodbye path | Workaround |
//! | --- | --- | --- | --- |
//! | [`Connection`] | `close(self) -> Result<_, CloseError>` | `Drop` flushes and swallows | newtype around `Option<Session>` |
//! | [`BatchWriter`] | `finish(self) -> Result<_, CloseError>` | `Drop` flushes and swallows | `std::mem::take` on a field with an empty value |
//!
//! The third workaround from the book, `ManuallyDrop`, is at the end: it removes
//! the `unwrap`s at the cost of an `unsafe` block.
//!
//! > The moment you add an explicit destructor, you will run into two issues.
//! > First, since your type implements `Drop` , you can no longer move out of any
//! > of that type's fields in the destructor. … Second, `drop` takes `&mut self`
//! > , not `self` , so your `Drop` implementation cannot simply call your
//! > explicit destructor and ignore its result.

use std::fmt;
use std::mem::{ManuallyDrop, size_of};
use std::sync::{Arc, Mutex};

/// A record of what a connection did, shared between the value and the test.
pub type Log = Arc<Mutex<Vec<String>>>;

/// A host that never accepts anything, so both destructors have an error to
/// report.
pub const UNREACHABLE: &str = "unreachable.example";

/// What a close can report. `Drop` has nowhere to put this, which is the entire
/// reason the explicit destructor exists.
#[derive(Debug, PartialEq, Eq)]
pub struct CloseError {
    host: String,
    unsent: usize,
}

impl CloseError {
    /// How many lines were still queued when the connection failed.
    pub fn unsent(&self) -> usize {
        self.unsent
    }
}

impl fmt::Display for CloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} unreachable, {} line(s) unsent",
            self.host, self.unsent
        )
    }
}

impl std::error::Error for CloseError {}

/// Deliver everything queued, or fail with what is left over.
fn flush(host: &str, pending: &mut Vec<String>, log: &Log) -> Result<usize, CloseError> {
    let unsent = pending.len();
    if host == UNREACHABLE {
        return Err(CloseError {
            host: host.to_owned(),
            unsent,
        });
    }

    for line in pending.drain(..) {
        log.lock().unwrap().push(format!("{host}: {line}"));
    }
    Ok(unsent)
}

// ---------------------------------------------------------------------------
// Workaround 1: the outer type wraps an Option
// ---------------------------------------------------------------------------

/// The inner state. It deliberately does **not** implement `Drop`, which is what
/// lets the outer type move its fields out.
struct Session {
    host: String,
    pending: Vec<String>,
    log: Log,
}

impl Session {
    fn flush(&mut self) -> Result<usize, CloseError> {
        flush(&self.host, &mut self.pending, &self.log)
    }
}

/// A connection whose cleanup can fail.
///
/// The explicit destructor is [`Connection::close`]. `Drop` is only the fallback
/// for callers who forget it.
///
/// > The first is to make your top-level type a newtype wrapper around an
/// > `Option` , which in turn holds some inner type that holds all of the type's
/// > fields. You can then use `Option::take` in both destructors, and call the
/// > inner type's explicit destructor only if the inner type has not already been
/// > taken.
///
/// Why not move the fields straight out of `Connection`? Because `Connection:
/// Drop`, and a destructor still to come means no field may leave:
///
/// ```compile_fail,E0509
/// struct Connection {
///     host: String,
/// }
///
/// impl Connection {
///     fn close(self) -> String {
///         self.host   // cannot move out of type `Connection`,
///                     // which implements the `Drop` trait
///     }
/// }
///
/// impl Drop for Connection {
///     fn drop(&mut self) {}
/// }
/// ```
pub struct Connection {
    inner: Option<Session>,
}

impl Connection {
    pub fn open(host: &str, log: &Log) -> Self {
        Self {
            inner: Some(Session {
                host: host.to_owned(),
                pending: Vec::new(),
                log: log.clone(),
            }),
        }
    }

    pub fn send(&mut self, line: &str) {
        if let Some(session) = self.inner.as_mut() {
            session.pending.push(line.to_owned());
        }
    }

    /// The explicit destructor: it takes `self`, so it is free to report the
    /// error that `Drop` would have swallowed. Calling it consumes the
    /// connection, and the `Option` is left as `None` so `Drop` has nothing to
    /// do afterwards.
    pub fn close(mut self) -> Result<usize, CloseError> {
        let mut session = self.inner.take().expect("a live connection");
        session.flush()
    }
}

/// > we need to provide best-effort cleanup through `Drop` . If cleanup errors,
/// > at least we tried—we swallow the error and move on.
///
/// And why can this not just call [`Connection::close`] and ignore the result? A
/// destructor receives `&mut self`, not `self`:
///
/// ```compile_fail,E0507
/// struct Connection;
///
/// impl Connection {
///     fn close(self) -> Result<(), ()> {
///         Ok(())
///     }
/// }
///
/// impl Drop for Connection {
///     fn drop(&mut self) {
///         let _ = self.close();   // cannot move out of `*self`,
///                                 // which is behind a mutable reference
///     }
/// }
/// ```
impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(mut session) = self.inner.take() {
            match session.flush() {
                Ok(flushed) => session
                    .log
                    .lock()
                    .unwrap()
                    .push(format!("drop: flushed {flushed}")),
                Err(error) => session
                    .log
                    .lock()
                    .unwrap()
                    .push(format!("drop: swallowed {error}")),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Workaround 2: takeable fields
// ---------------------------------------------------------------------------

/// A writer whose queue is a field with a sane empty value.
///
/// > The second workaround is to make each of your fields _takeable_ . … For
/// > example, you can take a `Vec` or `HashMap` by simply replacing them with
/// > their cheap-to-construct default values— `std::mem::take` is your friend
/// > here.
///
/// > I would err on the side of going with the second option, and switching to
/// > the others only if you find yourself in a sea of `Option` s.
///
/// The explicit destructor is [`BatchWriter::finish`].
pub struct BatchWriter {
    host: String,
    pending: Vec<String>,
    log: Log,
}

impl BatchWriter {
    pub fn open(host: &str, log: &Log) -> Self {
        Self {
            host: host.to_owned(),
            pending: Vec::new(),
            log: log.clone(),
        }
    }

    pub fn queue(&mut self, line: &str) {
        self.pending.push(line.to_owned());
    }

    /// The explicit destructor. No `Option` and no `unwrap`: the work is taken
    /// out of the field with `std::mem::take`, which leaves an empty `Vec`
    /// behind for `Drop` to find.
    pub fn finish(mut self) -> Result<usize, CloseError> {
        // `self.pending` cannot be moved out of `*self`, so it is replaced.
        let mut pending = std::mem::take(&mut self.pending);
        flush(&self.host, &mut pending, &self.log)
    }
}

impl Drop for BatchWriter {
    fn drop(&mut self) {
        // An empty queue is this type's `None`: `finish` has already taken the
        // work, so there is nothing best-effort left to do.
        if self.pending.is_empty() {
            return;
        }

        let mut pending = std::mem::take(&mut self.pending);
        if let Err(error) = flush(&self.host, &mut pending, &self.log) {
            self.log
                .lock()
                .unwrap()
                .push(format!("drop: swallowed {error}"));
        }
    }
}

// ---------------------------------------------------------------------------
// The two paths, observed
// ---------------------------------------------------------------------------

/// > This usually takes the form of a method that takes ownership of `self` and
/// > exposes any errors (using `-> Result<_, _>` ) … that are inherent to the
/// > destruction.
///
/// A successful close delivers the queued lines and reports how many. Because
/// `close` consumed the `Option`, the `Drop` that runs afterwards finds nothing
/// and adds no second entry — cleanup happened exactly once.
pub fn an_explicit_destructor_reports_what_drop_would_swallow() {
    let log = Log::default();
    let mut live = Connection::open("example.com", &log);
    live.send("hello");
    live.send("world");

    let flushed = live.close().unwrap();

    assert_eq!(flushed, 2);
    assert_eq!(
        *log.lock().unwrap(),
        ["example.com: hello", "example.com: world"]
    );
}

/// > If cleanup errors, at least we tried—we swallow the error and move on.
///
/// Without an explicit close, the same failure produces no panic and no `Result`:
/// the error is turned into a log line by `Drop` and exists nowhere else. That is
/// the best-effort contract.
pub fn drop_is_the_best_effort_fallback() {
    let log = Log::default();
    {
        let mut doomed = Connection::open(UNREACHABLE, &log);
        doomed.send("hello");
    } // no close: `Drop` runs and swallows

    let entries = log.lock().unwrap().clone();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].starts_with("drop: swallowed"));
    assert!(entries[0].contains("1 line(s) unsent"));

    // A reachable host still gets its data flushed from `Drop`.
    let log = Log::default();
    {
        let mut ok = Connection::open("example.com", &log);
        ok.send("bye");
    }
    assert_eq!(
        *log.lock().unwrap(),
        ["example.com: bye", "drop: flushed 1"]
    );
}

/// > The second workaround is to make each of your fields _takeable_ .
///
/// `finish` empties the field, so the `Drop` that follows does nothing; `Drop`
/// alone still flushes best-effort, and still swallows the error. No `Option` and
/// no `unwrap` appear anywhere in the type.
pub fn a_takeable_field_replaces_the_option() {
    let log = Log::default();
    let mut writer = BatchWriter::open("example.com", &log);
    writer.queue("one");
    writer.queue("two");

    assert_eq!(writer.finish().unwrap(), 2);
    assert_eq!(
        *log.lock().unwrap(),
        ["example.com: one", "example.com: two"]
    );

    let log = Log::default();
    {
        let mut writer = BatchWriter::open(UNREACHABLE, &log);
        writer.queue("one");
    }

    let entries = log.lock().unwrap().clone();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].starts_with("drop: swallowed"));
}

// ---------------------------------------------------------------------------
// Workaround 3: ManuallyDrop, and what it costs
// ---------------------------------------------------------------------------

/// > The third option is to hold the data inside the `ManuallyDrop` type, which
/// > dereferences to the inner type, so there's no need for unwraps.
///
/// > The primary downside of this approach is that `ManuallyDrop::take` is
/// > `unsafe` . There are no safety mechanisms in place to ensure that you don't
/// > try to use the value inside the `ManuallyDrop` after you've called `take`
/// > or that you don't call `take` multiple times.
///
/// The safe path mirrors the two destructors above: the wrapper dereferences, and
/// `into_inner` consumes it, so no moved-out value is left behind. Only `take` —
/// the operation both destructors need — is unsafe, because it takes `&mut self`
/// and leaves the wrapper looking full:
///
/// ```compile_fail,E0133
/// use std::mem::ManuallyDrop;
///
/// fn take_out(slot: &mut ManuallyDrop<String>) -> String {
///     ManuallyDrop::take(slot)   // call to unsafe function is unsafe and
///                                // requires unsafe block
/// }
/// ```
pub fn manually_drop_promises_no_unwraps_but_take_is_unsafe() {
    let wrapped = ManuallyDrop::new(String::from("ferris"));

    // `Deref`, so the fields are reachable without any `unwrap`.
    assert_eq!(wrapped.len(), 6);
    assert_eq!(size_of::<ManuallyDrop<String>>(), size_of::<String>());

    // Consuming the wrapper is safe: nothing is left claiming to hold the value.
    let inner = ManuallyDrop::into_inner(wrapped);
    assert_eq!(inner, "ferris");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_explicit_destructor_reports_what_drop_would_swallow_runs() {
        an_explicit_destructor_reports_what_drop_would_swallow();
    }

    #[test]
    fn drop_is_the_best_effort_fallback_runs() {
        drop_is_the_best_effort_fallback();
    }

    #[test]
    fn a_takeable_field_replaces_the_option_runs() {
        a_takeable_field_replaces_the_option();
    }

    #[test]
    fn manually_drop_promises_no_unwraps_but_take_is_unsafe_runs() {
        manually_drop_promises_no_unwraps_but_take_is_unsafe();
    }
}
