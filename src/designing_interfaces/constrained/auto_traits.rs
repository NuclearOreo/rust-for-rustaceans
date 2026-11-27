//! # Constrained — Auto-Traits
//!
//! > Rust has a handful of traits that are automatically implemented for every
//! > type depending on what that type contains. The most relevant of these for
//! > this discussion are `Send` and `Sync` , though the `Unpin` , `Sized` , and
//! > `UnwindSafe` traits have similar issues. By their very nature, these add a
//! > hidden promise made by nearly every type in your interface. These traits even
//! > propagate through otherwise type-erased types like `impl Trait` .
//!
//! > So, if you have a public type `A` that contains a private type `B` , and you
//! > change `B` so that it is no longer `Send` , then `A` is now _also_ not `Send`
//! > . That is a breaking change!
//!
//! > To catch these cases before they happen, it's good practice to include some
//! > simple tests in your test suite that check that all your types implement these
//! > traits the way you expect. _Listing 3-8_ gives an example of what such a test
//! > might look like.
//!
//! > Notice that this test does not run any code, but simply tests that the code
//! > compiles.
//!
//! That sentence is the reason this file's evidence is inverted: the *test* is a
//! type-check that must succeed, and the demonstration of the failure is a
//! `compile_fail` doctest that must be rejected. [`is_normal`] is Listing 3-8
//! verbatim, made `pub` so its doctests are collected.

use std::cell::RefCell;
use std::rc::Rc;

/// Listing 3-8, verbatim apart from being `pub`: a function that only
/// instantiates its bounds.
pub fn is_normal<T: Sized + Send + Sync + Unpin>() {}

/// The `UnwindSafe` half of the same family, as a separate helper.
pub fn is_unwind_safe<T: std::panic::UnwindSafe>() {}

/// A public type whose private fields are all ordinary: `String` and `usize` are
/// `Send + Sync + Unpin`, so the type is too.
pub struct Normal {
    text: String,
    count: usize,
}

impl Normal {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            count: 0,
        }
    }
}

impl Default for Normal {
    fn default() -> Self {
        Self::new()
    }
}

/// The same public type after one private field changed to `Rc<()>`.
///
/// > if you have a public type `A` that contains a private type `B` , and you
/// > change `B` so that it is no longer `Send` , then `A` is now _also_ not `Send`
///
/// Nothing about `NotSend`'s public surface changed — it is still a public struct
/// with no public fields — but the promise that came for free is gone:
///
/// ```compile_fail,E0277
/// use rust_for_rustaceans::designing_interfaces::constrained::auto_traits::{
///     is_normal, NotSend,
/// };
///
/// // `Rc<()>` cannot be sent between threads safely
/// is_normal::<NotSend>();
/// ```
pub struct NotSend {
    shared: Rc<()>,
}

impl NotSend {
    pub fn new() -> Self {
        Self {
            shared: Rc::new(()),
        }
    }
}

impl Default for NotSend {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// The same promise, hidden behind `impl Trait`
// ---------------------------------------------------------------------------

/// A helper shaped like an assertion: the argument has to be `Send`.
pub fn assert_send<T: Send>(value: T) -> T {
    value
}

/// An opaque iterator that owns an `Rc` — the return type has no name, but the
/// auto-trait promise is computed from the hidden type all the same.
///
/// ```compile_fail,E0277
/// use rust_for_rustaceans::designing_interfaces::constrained::auto_traits::{
///     assert_send, opaque_shared,
/// };
///
/// // `Rc<()>` cannot be sent between threads safely
/// let _ = assert_send(opaque_shared());
/// ```
pub fn opaque_shared() -> impl Iterator<Item = u64> {
    let shared = Rc::new(());
    (0..1u64).map(move |i| {
        let _ = &shared;
        i + 1
    })
}

/// The same signature with a `Send` hidden type: the caller cannot tell the two
/// apart from the signature alone, which is exactly what makes the change to the
/// hidden type a breaking one.
pub fn opaque_owned() -> impl Iterator<Item = u64> {
    (0..3u64).map(|i| i + 1)
}

// ---------------------------------------------------------------------------
// What the tests can and cannot see
// ---------------------------------------------------------------------------

/// > This test … simply tests that the code compiles. If `MyType` no longer
/// > implements `Sync` , the test code will not compile, and you will know that
/// > the change you just broke the auto-trait implementation.
///
/// The positive half runs: [`Normal`] and the primitives pass the same check that
/// [`NotSend`] fails. Nothing here executes; the assertion is that this function
/// compiled at all, plus a trivial fact to keep the demo in the usual shape.
pub fn listing_3_8_a_compile_only_test_catches_a_lost_auto_trait() {
    is_normal::<Normal>();
    is_normal::<String>();
    is_normal::<u8>();
    is_normal::<Vec<Normal>>();

    let normal = Normal::new();
    assert!(normal.text.is_empty());
    assert_eq!(normal.count, 0);
}

/// > These traits even propagate through otherwise type-erased types like `impl
/// > Trait` .
///
/// The owned iterator is `Send`, so it survives the assertion; the `Rc`-holding
/// one does not compile at all (doctest above). The two signatures are otherwise
/// identical.
pub fn auto_traits_propagate_through_impl_trait() {
    let owned = assert_send(opaque_owned());

    assert_eq!(owned.collect::<Vec<_>>(), [1, 2, 3]);

    // `NotSend` exists and can be built; it is only the *promise* that is gone.
    let not_send = NotSend::new();
    assert_eq!(Rc::strong_count(&not_send.shared), 1);
}

/// > though the `Unpin` , `Sized` , and `UnwindSafe` traits have similar issues
///
/// `UnwindSafe` is a good second example because the failure is about *interior
/// mutability* seen through a reference: an owned `RefCell<u8>` is unwind-safe,
/// a `&RefCell<u8>` is not, so the same helper accepts one and refuses the other.
///
/// ```compile_fail,E0277
/// use std::cell::RefCell;
///
/// use rust_for_rustaceans::designing_interfaces::constrained::auto_traits::is_unwind_safe;
///
/// // `UnsafeCell<u8>` may contain interior mutability and a reference may not be
/// // safely transferable across a catch_unwind boundary
/// is_unwind_safe::<&RefCell<u8>>();
/// ```
pub fn unwind_safe_is_in_the_same_family() {
    is_unwind_safe::<u8>();
    is_unwind_safe::<RefCell<u8>>();
    is_unwind_safe::<String>();

    // The refused type is the *reference*, not the `RefCell` itself.
    let cell = RefCell::new(8u8);
    assert_eq!(*cell.borrow(), 8);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_3_8_a_compile_only_test_catches_a_lost_auto_trait_runs() {
        listing_3_8_a_compile_only_test_catches_a_lost_auto_trait();
    }

    #[test]
    fn auto_traits_propagate_through_impl_trait_runs() {
        auto_traits_propagate_through_impl_trait();
    }

    #[test]
    fn unwind_safe_is_in_the_same_family_runs() {
        unwind_safe_is_in_the_same_family();
    }
}
