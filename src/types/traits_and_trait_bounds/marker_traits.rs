//! # Marker Traits
//!
//! > Usually, we use traits to denote functionality that multiple types can
//! > support … But not all traits are functional in this way. Some traits, called
//! > _marker traits_, instead indicate a property of the implementing type.
//! > Marker traits have no methods or associated types and serve just to tell you
//! > that a particular type can or cannot be used in a certain way.
//!
//! > Most of these (all except `Copy`) are also _auto-traits_; the compiler
//! > automatically implements them for types unless the type contains something
//! > that does not implement the marker trait.
//!
//! > Similar to marker traits are _marker types_. These are unit types (like
//! > `struct MyMarker;`) that hold no data and have no methods.
//!
//! Three things are shown here: a marker trait of our own, the standard
//! library's auto traits (`Send`, `Sync`) and how a field decides them, and the
//! typestate pattern that marker *types* enable.

use std::cell::Cell;
use std::mem::size_of;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// A marker trait of our own
// ---------------------------------------------------------------------------

/// A marker trait: no methods, no associated types, no implementation to write.
pub trait Validated {}

/// The only type that is `Validated`. Being a unit struct, it occupies nothing.
pub struct ValidatedInput;

impl Validated for ValidatedInput {}

/// > There are no methods associated with this behavior; it's just a fact about
/// > the type.
///
/// A marker trait is a bound that carries a fact, and it costs nothing at run
/// time: the marker type is zero-sized and the bound is erased after checking.
pub fn a_marker_trait_records_a_fact_without_a_method() {
    fn accept<T: Validated>(value: T) -> T {
        value
    }

    let input = accept(ValidatedInput);

    assert_eq!(size_of::<ValidatedInput>(), 0);
    assert_eq!(size_of_val_of(&input), 0);
}

fn size_of_val_of<T>(value: &T) -> usize {
    std::mem::size_of_val(value)
}

// ---------------------------------------------------------------------------
// Auto traits
// ---------------------------------------------------------------------------

/// > if a type implements the `Send` marker trait, it is safe to send across
/// > thread boundaries. If it does not implement this marker trait, it isn't
/// > safe to send.
///
/// Nothing in `Vec<u8>` mentions `Send`; the compiler inferred it, and the bound
/// is how a function says "I am relying on that inference".
pub fn auto_traits_are_inferred_from_the_type() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<Vec<u8>>();
    assert_sync::<Vec<u8>>();
    assert_send::<String>();

    // A `Cell` is `Send` — it can be moved to another thread — but it is not
    // `Sync`, because `&Cell` would allow shared mutation from two threads.
    assert_send::<Cell<u8>>();
}

/// > the compiler automatically implements them for types unless the type
/// > contains something that does not implement the marker trait.
///
/// `Rc` is the classic opt-out for `Send`: its reference count is not atomic, so
/// two threads could corrupt it.
///
/// ```compile_fail,E0277
/// fn assert_send<T: Send>() {}
///
/// assert_send::<std::rc::Rc<u8>>();   // `Rc<u8>` cannot be sent between threads
/// ```
///
/// `Cell` is `Send` but not `Sync`:
///
/// ```compile_fail,E0277
/// fn assert_sync<T: Sync>() {}
///
/// assert_sync::<std::cell::Cell<u8>>();   // `Cell<u8>` cannot be shared
///                                         // between threads safely
/// ```
///
/// And a struct is only as `Send` as its fields — auto traits are inherited, not
/// declared:
///
/// ```compile_fail,E0277
/// struct Holder {
///     value: std::rc::Rc<u8>,
/// }
///
/// fn assert_send<T: Send>() {}
///
/// assert_send::<Holder>();   // `Rc<u8>` cannot be sent between threads safely
/// ```
pub fn auto_traits_are_inherited_from_fields() {
    /// `Rc` inside a struct: the struct inherits the opt-out.
    pub struct Holder {
        pub value: Rc<u8>,
    }

    let holder = Holder { value: Rc::new(1) };

    // The facts the compile-fail doctests above are about, checked from the
    // ungated side: `Rc` is still perfectly usable on one thread.
    assert_eq!(*holder.value, 1);
    assert_eq!(Rc::strong_count(&holder.value), 1);
}

// ---------------------------------------------------------------------------
// Marker types (typestate)
// ---------------------------------------------------------------------------

/// The two states a connection can be in. Each is a marker type: a unit struct
/// used only as a type parameter.
pub struct Unauthenticated;
pub struct Authenticated;

/// > consider a type like `SshConnection`, which may or may not have been
/// > authenticated yet. You could add a generic type argument to `SshConnection`
/// > and then create two marker types: `Unauthenticated` and `Authenticated`.
pub struct SshConnection<State> {
    pub host: String,
    pub state: State,
}

/// > In its `impl` block, you provide only a single method: `connect`. The
/// > `connect` method returns a `SshConnection<Authenticated>`, and it's only in
/// > that `impl` block that you provide the remaining methods for running
/// > commands and such.
impl SshConnection<Unauthenticated> {
    pub fn connect(host: &str) -> SshConnection<Authenticated> {
        SshConnection {
            host: host.to_string(),
            state: Authenticated,
        }
    }
}

impl SshConnection<Authenticated> {
    pub fn run(&self, command: &str) -> String {
        format!("{}: {command}", self.host)
    }
}

/// The state is enforced by the type system, and costs nothing: `state` is a
/// zero-sized field, so `SshConnection<Authenticated>` is the same size as
/// `SshConnection<Unauthenticated>` and as a bare `String`.
pub fn marker_types_make_illegal_states_unrepresentable() {
    let connection = SshConnection::<Unauthenticated>::connect("example.com");
    assert_eq!(connection.run("ls"), "example.com: ls");

    assert_eq!(size_of::<Authenticated>(), 0);
    assert_eq!(size_of::<Unauthenticated>(), 0);
    assert_eq!(
        size_of::<SshConnection<Authenticated>>(),
        size_of::<String>()
    );
}

/// Calling `run` before authenticating is not a runtime error to be handled — it
/// is a method that does not exist for that type:
///
/// ```compile_fail,E0599
/// use rust_for_rustaceans::types::traits_and_trait_bounds::marker_traits::{
///     SshConnection, Unauthenticated,
/// };
///
/// let connection = SshConnection {
///     host: "example.com".to_string(),
///     state: Unauthenticated,
/// };
/// connection.run("ls");   // no method named `run` found for
///                         // `SshConnection<Unauthenticated>`
/// ```
pub fn an_unauthenticated_connection_has_no_run_method() {
    // The positive half: the type that *does* have `run` is the one `connect`
    // returns, so the only way to reach `run` is through `connect`.
    let connection = SshConnection::<Unauthenticated>::connect("example.com");
    assert_eq!(connection.run("whoami"), "example.com: whoami");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_marker_trait_records_a_fact_without_a_method_runs() {
        a_marker_trait_records_a_fact_without_a_method();
    }

    #[test]
    fn auto_traits_are_inferred_from_the_type_runs() {
        auto_traits_are_inferred_from_the_type();
    }

    #[test]
    fn auto_traits_are_inherited_from_fields_runs() {
        auto_traits_are_inherited_from_fields();
    }

    #[test]
    fn marker_types_make_illegal_states_unrepresentable_runs() {
        marker_types_make_illegal_states_unrepresentable();
    }

    #[test]
    fn an_unauthenticated_connection_has_no_run_method_runs() {
        an_unauthenticated_connection_has_no_run_method();
    }
}
