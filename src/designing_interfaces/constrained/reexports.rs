//! # Constrained — Re-Exports
//!
//! > If any part of your interface exposes foreign types, then any change to one
//! > of those foreign types is _also_ a change to your interface.
//!
//! > To mitigate issues like this, it's often best to wrap foreign types using the
//! > newtype pattern, and then expose only the parts of the foreign type that you
//! > think are useful. In many cases, you can avoid the newtype wrapper altogether
//! > by using `impl Trait` to provide only the very minimal contract to the
//! > caller. By promising less, you make fewer changes breaking.
//!
//! The example needs two major versions of one dependency, so the two `itercrate`
//! modules below stand in for them. They have the same name, the same fields, and
//! the same shape — the book's point is that none of that makes them the same
//! type:
//!
//! | Version | `Empty<T>` | Move semantics |
//! | --- | --- | --- |
//! | `itercrate_1` | `Empty<T> { marker: T }` | not `Copy` |
//! | `itercrate_2` | `Empty<T> { marker: T }` | `Copy` |
//!
//! The `Copy` difference is the semver-trick aside made concrete: 2.0 "added a
//! `#[derive(Copy)]`", so the type moved differently even though nothing about
//! its fields changed. Layout equality is demonstrated next to identity
//! inequality, which is the whole reason the compiler refuses the assignment in
//! Listing 3-7.

use std::any::type_name;
use std::mem::size_of;

// ---------------------------------------------------------------------------
// Two major versions of one dependency
// ---------------------------------------------------------------------------

/// Stands in for `itercrate` 1.0.
pub mod itercrate_1 {
    /// The 1.0 empty iterator: owns a marker, and is not `Copy`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Empty<T> {
        pub marker: T,
    }

    impl<T> Empty<T> {
        pub fn new(marker: T) -> Self {
            Self { marker }
        }
    }
}

/// Stands in for `itercrate` 2.0: same name, same field, now `Copy`.
pub mod itercrate_2 {
    /// The 2.0 empty iterator.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Empty<T> {
        pub marker: T,
    }

    impl<T> Empty<T> {
        pub fn new(marker: T) -> Self {
            Self { marker }
        }
    }
}

/// Listing 3-7, the "your crate: bestiter" side, pinned to 1.0.
pub fn iter_1<T>(marker: T) -> itercrate_1::Empty<T> {
    itercrate_1::Empty::new(marker)
}

/// The same function after the dependency moved to 2.0: a different return type
/// whose name is identical.
pub fn iter_2<T>(marker: T) -> itercrate_2::Empty<T> {
    itercrate_2::Empty::new(marker)
}

// ---------------------------------------------------------------------------
// Listing 3-7: the same name is not the same type
// ---------------------------------------------------------------------------

/// > If your crate moves from `itercrate 1.0` to `itercrate 2.0` but otherwise
/// > does not change, the code in this listing will no longer compile. Even though
/// > no types have changed, the compiler believes (correctly) that
/// > `itercrate1.0::Empty` and `itercrate2.0::Empty` are _different_ types.
///
/// The user's struct names the 2.0 type, and the function still returns the 1.0
/// one:
///
/// ```compile_fail,E0308
/// use rust_for_rustaceans::designing_interfaces::constrained::reexports::{iter_1, itercrate_2};
///
/// struct EmptyIterator {
///     it: itercrate_2::Empty<()>,
/// }
///
/// // expected `itercrate_2::Empty<()>`, found `itercrate_1::Empty<()>`
/// let _ = EmptyIterator { it: iter_1(()) };
/// ```
///
/// > The Rust compiler does not assume that just because two types have the same
/// > fields, they are the same. … imagine that `itercrate` 2.0 added a
/// > `#[derive(Copy)]` for `Empty` . Now, the type suddenly has different move
/// > semantics depending on whether you are using 1.0 or 2.0!
///
/// The 1.0 value is consumed by a by-value function, so it cannot be used again:
///
/// ```compile_fail,E0382
/// use rust_for_rustaceans::designing_interfaces::constrained::reexports::iter_1;
///
/// fn by_value(_: rust_for_rustaceans::designing_interfaces::constrained::reexports::itercrate_1::Empty<()>) {}
///
/// let one = iter_1(());
/// by_value(one);
/// let _also = one;   // use of moved value: `one`
/// ```
pub fn listing_3_7_two_major_versions_are_two_types() {
    // Same layout, same field, same name.
    assert_eq!(
        size_of::<itercrate_1::Empty<()>>(),
        size_of::<itercrate_2::Empty<()>>()
    );

    // Different types, and the compiler says so in their names.
    assert_ne!(
        type_name::<itercrate_1::Empty<()>>(),
        type_name::<itercrate_2::Empty<()>>()
    );

    // And the difference that is not about layout: 2.0 is `Copy`.
    let two = iter_2(());
    let also_two = two;
    assert_eq!(two, also_two); // `two` is still usable

    let one = iter_1(());
    assert_eq!(one, itercrate_1::Empty::new(())); // one use of the 1.0 value
}

// ---------------------------------------------------------------------------
// Mitigation 1: the newtype wrapper
// ---------------------------------------------------------------------------

/// The newtype mitigation: our own type, wrapping the dependency's privately, so
/// the dependency can be swapped without the user noticing.
///
/// The field is private, which is what keeps the wrapper's identity ours:
///
/// ```compile_fail,E0616
/// use rust_for_rustaceans::designing_interfaces::constrained::reexports::EmptyIter;
///
/// let iter = EmptyIter::empty(());
/// // field `inner` of struct `EmptyIter` is private
/// let _ = iter.inner;
/// ```
pub struct EmptyIter<T> {
    inner: itercrate_1::Empty<T>,
}

impl<T> EmptyIter<T> {
    pub fn empty(marker: T) -> Self {
        Self {
            inner: itercrate_1::Empty::new(marker),
        }
    }

    /// The marker the wrapped iterator holds — the one part of the dependency's
    /// type this wrapper chooses to expose.
    pub fn marker(&self) -> &T {
        &self.inner.marker
    }
}

impl<T> Iterator for EmptyIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        None
    }
}

/// The wrapper is nameable by the user and is not the dependency's type at all,
/// so a major bump inside it is invisible to every caller.
pub fn a_newtype_keeps_the_dependency_out_of_the_contract() {
    let iter: EmptyIter<()> = EmptyIter::empty(());

    assert_eq!(iter.marker(), &());
    assert_eq!(iter.count(), 0);
}

// ---------------------------------------------------------------------------
// Mitigation 2: impl Trait promises less
// ---------------------------------------------------------------------------

/// > In many cases, you can avoid the newtype wrapper altogether by using `impl
/// > Trait` to provide only the very minimal contract to the caller. By promising
/// > less, you make fewer changes breaking.
///
/// The caller sees `Iterator<Item = u64>` and nothing else — no dependency type,
/// no name to break:
///
/// ```
/// use rust_for_rustaceans::designing_interfaces::constrained::reexports::iter_minimal;
///
/// fn count(it: impl Iterator<Item = u64>) -> usize {
///     it.count()
/// }
///
/// assert_eq!(count(iter_minimal()), 0);
/// ```
pub fn iter_minimal() -> impl Iterator<Item = u64> {
    std::iter::empty()
}

/// The opaque return type cannot be named by the caller even if it wanted to,
/// which is the point: there is no name in the contract to change.
pub fn impl_trait_promises_less_so_fewer_changes_break() {
    fn count(it: impl Iterator<Item = u64>) -> usize {
        it.count()
    }

    assert_eq!(count(iter_minimal()), 0);
}

// ---------------------------------------------------------------------------
// The semver trick
// ---------------------------------------------------------------------------

/// > The idea is simple: if some type `T` stays the same across a breaking change
/// > … then after releasing 2.0, you can release a new 1.0 _minor_ version that
/// > depends on 2.0 and replaces `T` with a reexport of `T` from 2.0.
///
/// This module is that 1.x release: [`Empty`](itercrate_1::Empty) in the 1.0
/// namespace becomes the 2.0 type.
pub mod compat_1_x {
    /// The opt-in that makes one type stand for two major versions.
    pub use super::itercrate_2::Empty;
}

/// > By doing this, you're ensuring that there is in fact only a single type `T`
/// > across both major versions.
///
/// The user's 1.x dependency now hands back a value the user's 2.x-typed struct
/// accepts — because it *is* that type. The old `itercrate_1::Empty` is
/// untouched: only explicitly opted-in types are reconciled.
pub fn the_semver_trick_makes_one_type_stand_for_two() {
    fn takes_2_0(_: itercrate_2::Empty<()>) {}

    // A value from the 1.x compatibility surface is a 2.0 value.
    takes_2_0(compat_1_x::Empty::new(()));

    assert_eq!(
        type_name::<compat_1_x::Empty<()>>(),
        type_name::<itercrate_2::Empty<()>>()
    );

    // The type that did not opt in is still distinct.
    assert_ne!(
        type_name::<compat_1_x::Empty<()>>(),
        type_name::<itercrate_1::Empty<()>>()
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_3_7_two_major_versions_are_two_types_runs() {
        listing_3_7_two_major_versions_are_two_types();
    }

    #[test]
    fn a_newtype_keeps_the_dependency_out_of_the_contract_runs() {
        a_newtype_keeps_the_dependency_out_of_the_contract();
    }

    #[test]
    fn impl_trait_promises_less_so_fewer_changes_break_runs() {
        impl_trait_promises_less_so_fewer_changes_break();
    }

    #[test]
    fn the_semver_trick_makes_one_type_stand_for_two_runs() {
        the_semver_trick_makes_one_type_stand_for_two();
    }
}
