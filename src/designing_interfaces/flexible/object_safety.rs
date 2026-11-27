//! # Object Safety
//!
//! > When you define a new trait, whether or not that trait is object-safe (see
//! > the end of "Compilation and Dispatch" in Chapter 2) is an unwritten part of
//! > the trait's contract. If the trait is object-safe, users can treat different
//! > types that implement your trait as a single common type using `dyn Trait` .
//!
//! > You should prefer your traits to be object-safe even if that comes at a
//! > slight cost to the ergonomics of using them (such as taking `impl
//! > AsRef<str>` over `&str` ), since object safety enables new ways to use your
//! > traits.
//!
//! > If your trait must have a generic method, consider whether its generic
//! > parameters can be on the trait itself or if its generic arguments can also
//! > use dynamic dispatch to preserve the object safety of the trait.
//! > Alternatively, you can add a `where Self: Sized` trait bound to that method,
//! > which makes it possible to call the method only with a concrete instance of
//! > the trait (and not through `dyn Trait` ).
//!
//! The demos take the book's own examples in order: a heterogeneous collection
//! of `dyn Notifier` (what object safety buys), the `where Self: Sized`
//! convenience method that comes along for free (what it does not cost), `dyn
//! Seek` as a trait object that is legal and nearly useless, and the `drop` entry
//! that every vtable carries.
//!
//! > Remember that object safety is a part of your public interface! If you
//! > modify a trait in an otherwise backward compatible way, such as by adding a
//! > method with a default implementation, but it makes the trait not
//! > object-safe, you need to bump your major semantic version number.
//!
//! Two `compile_fail` doctests pin the removals down: a default method that
//! mentions `Self` in its return type, and a standard-library trait whose method
//! has no receiver.

use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// A trait designed to stay dyn compatible
// ---------------------------------------------------------------------------

/// The vtable-visible surface is deliberately plain: `&self` in, owned `String`
/// out, no generics, no `Self` beyond the receiver.
///
/// The section's other half is that an added method can still break object
/// safety even when it looks backward compatible:
///
/// ```compile_fail,E0038
/// trait Notifier {
///     fn notify(&self, message: &str);
///     fn duplicate(&self) -> Self;   // a default body would not help:
///                                    // the return type uses `Self`
/// }
///
/// let _: &dyn Notifier;   // the trait `Notifier` is not dyn compatible
/// ```
pub trait Notifier {
    /// Everything the vtable needs: a plain receiver, a plain argument, a plain
    /// return type.
    fn notify(&self, message: &str) -> String;

    /// A generic convenience method. `where Self: Sized` keeps it out of the
    /// vtable, which is exactly what lets the trait remain `dyn` compatible.
    fn notify_all<M: AsRef<str>>(&self, messages: &[M]) -> Vec<String>
    where
        Self: Sized,
    {
        messages.iter().map(|m| self.notify(m.as_ref())).collect()
    }
}

/// One implementor: prints to a terminal.
pub struct Terminal;

impl Notifier for Terminal {
    fn notify(&self, message: &str) -> String {
        format!("terminal: {message}")
    }
}

/// Another implementor: posts to a chat channel.
pub struct Slack {
    pub channel: &'static str,
}

impl Notifier for Slack {
    fn notify(&self, message: &str) -> String {
        format!("#{}: {message}", self.channel)
    }
}

// ---------------------------------------------------------------------------
// What object safety buys
// ---------------------------------------------------------------------------

/// > If the trait is object-safe, users can treat different types that implement
/// > your trait as a single common type using `dyn Trait` .
///
/// Two unrelated types, one `Vec`, one loop: the call site never mentions
/// `Terminal` or `Slack` again. Without object safety the only alternative would
/// be a generic parameter on the collection — which cannot hold both.
pub fn object_safety_lets_different_types_share_one_type() {
    let notifiers: Vec<Box<dyn Notifier>> =
        vec![Box::new(Terminal), Box::new(Slack { channel: "general" })];

    let sent: Vec<String> = notifiers.iter().map(|n| n.notify("deploy")).collect();

    assert_eq!(sent, ["terminal: deploy", "#general: deploy"]);
}

/// > Alternatively, you can add a `where Self: Sized` trait bound to that method,
/// > which makes it possible to call the method only with a concrete instance of
/// > the trait (and not through `dyn Trait` ).
///
/// The generic convenience method is callable on a concrete value and absent
/// from the vtable. That is the trade the book recommends: the trait stays
/// object-safe, and the method that could not be in the vtable stays callable
/// where a concrete type is known.
pub fn a_sized_convenience_method_stays_on_concrete_values() {
    let sent = Terminal.notify_all(&["one", "two"]);

    assert_eq!(sent, ["terminal: one", "terminal: two"]);

    // The vtable is unaffected: the trait object still works, and `notify_all`
    // is simply not reachable through it.
    let object: &dyn Notifier = &Terminal;
    assert_eq!(object.notify("three"), "terminal: three");
    // `object.notify_all(&["four"])` is refused with "the `notify_all` method
    // cannot be invoked on a trait object" — a diagnostic with no error code,
    // so it is recorded here rather than as a doctest.
}

// ---------------------------------------------------------------------------
// A trait object is only as useful as the trait
// ---------------------------------------------------------------------------

/// > `std::io::Seek` is fairly useless as a trait object on its own, because the
/// > only thing you would be able to do with such a trait object is seek, without
/// > being able to read or write.
///
/// This is legal code, and it still cannot read: `read` is not in `Seek`'s
/// vtable. The bytes are read afterwards through the concrete type.
///
/// The opposite case is a trait that cannot be a trait object at all, because
/// there would be nothing to build a vtable from — `FromIterator::from_iter`
/// takes no `self`:
///
/// ```compile_fail,E0038
/// // the trait `FromIterator` is not dyn compatible:
/// // method `from_iter` has no receiver
/// let _: &dyn FromIterator<u8>;
/// ```
pub fn a_trait_object_is_only_as_useful_as_the_trait() {
    let mut cursor = Cursor::new(b"hello".to_vec());

    let at = {
        let seekable: &mut dyn Seek = &mut cursor;
        seekable.seek(SeekFrom::End(-2)).unwrap()
    };
    assert_eq!(at, 3);

    // Reading had to go through the concrete type; `&mut dyn Seek` cannot.
    let mut rest = String::new();
    cursor.read_to_string(&mut rest).unwrap();
    assert_eq!(rest, "lo");
    assert_eq!(cursor.position(), 5);
}

// ---------------------------------------------------------------------------
// Drop trait objects
// ---------------------------------------------------------------------------

pub type Log = Arc<Mutex<Vec<&'static str>>>;

/// Only needs to be droppable and describable later; the `Drop` impl is what the
/// demo watches.
pub struct Loud {
    name: &'static str,
    log: Log,
}

impl Loud {
    pub fn new(name: &'static str, log: &Log) -> Self {
        Self {
            name,
            log: log.clone(),
        }
    }
}

impl Drop for Loud {
    fn drop(&mut self) {
        self.log.lock().unwrap().push(self.name);
    }
}

/// A trait that promises nothing except "this can be handled later".
pub trait Deferred {
    fn label(&self) -> &'static str;
}

impl Deferred for Loud {
    fn label(&self) -> &'static str {
        self.name
    }
}

/// > You might think that the `Drop` trait is also useless as a trait object,
/// > since all you can do with `Drop` as a trait object is to drop it. But it
/// > turns out there are some libraries that specifically just want to be able to
/// > drop arbitrary types.
///
/// > Interestingly enough, the story of `Drop` doesn't end there; since Rust
/// > needs to be able to drop trait objects too, _every_ vtable contains the
/// > `drop` method. Effectively, every `dyn Trait` is also a `dyn Drop` .
///
/// Two observations, in order. First: a `Box<dyn Deferred>` was never required to
/// implement `Drop` as far as the trait is concerned, yet clearing the queue
/// still runs each concrete type's destructor — drop glue is not part of the
/// trait's promises, it is part of the vtable. Second: `Drop` itself is a legal
/// trait object, which is how a deferred-drop library holds values of completely
/// unrelated types.
#[allow(dyn_drop)] // the lint exists to say drop glue is what runs; that is the demo
pub fn every_dyn_trait_is_also_a_dyn_drop() {
    let log = Log::default();

    let queue: Vec<Box<dyn Deferred>> = vec![
        Box::new(Loud::new("alpha", &log)),
        Box::new(Loud::new("beta", &log)),
    ];
    assert_eq!(queue[0].label(), "alpha");
    assert!(log.lock().unwrap().is_empty()); // still alive

    drop(queue);
    assert_eq!(*log.lock().unwrap(), ["alpha", "beta"]);

    // `Drop` as the trait object: the only thing you can do is drop it, which is
    // precisely what a deferred-cleanup queue needs.
    let deferred: Vec<Box<dyn Drop>> = vec![Box::new(Loud::new("gamma", &log))];
    drop(deferred);
    assert_eq!(*log.lock().unwrap(), ["alpha", "beta", "gamma"]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn object_safety_lets_different_types_share_one_type_runs() {
        object_safety_lets_different_types_share_one_type();
    }

    #[test]
    fn a_sized_convenience_method_stays_on_concrete_values_runs() {
        a_sized_convenience_method_stays_on_concrete_values();
    }

    #[test]
    fn a_trait_object_is_only_as_useful_as_the_trait_runs() {
        a_trait_object_is_only_as_useful_as_the_trait();
    }

    #[test]
    fn every_dyn_trait_is_also_a_dyn_drop_runs() {
        every_dyn_trait_is_also_a_dyn_drop();
    }
}
