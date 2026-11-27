//! # Constrained — Trait Implementations
//!
//! > Since we do not know what implementations downstream code may have added,
//! > adding a blanket implementation of an existing trait is generally a breaking
//! > change. The same holds true for implementing a foreign trait for an existing
//! > type, or an existing trait for a foreign type …
//!
//! > Removing a trait implementation is a breaking change, but implementing
//! > traits for a _new_ type is never a problem, since no crate can have
//! > implementations that conflict with that type.
//!
//! > Perhaps counterintuitively, you also want to be careful about implementing
//! > _any_ trait for an existing type. … If you add `impl Foo1 for Unit` to
//! > `crate1` without marking it a breaking change, the downstream code will
//! > suddenly stop compiling since the call to `foo` is now ambiguous.
//!
//! > Most changes to existing traits are also breaking changes … Adding a new
//! > method with a default implementation is fine though, since existing
//! > implementations will continue to apply.
//!
//! > … as interface authors, we have a tool available to us that lets us skirt
//! > some of these rules: _sealed traits_. A sealed trait is one that can be used
//! > only, and not implemented, by other crates.
//!
//! The five changes and their verification:
//!
//! | Change | User code | Code |
//! | --- | --- | --- |
//! | new `impl` for an existing type (Listing 3-5) | a call that was unambiguous | `E0034` |
//! | new blanket `impl` | a downstream `impl` for a local type | `E0119` |
//! | new required method on an open trait | the downstream `impl` | `E0046` |
//! | new *default* method on an open trait | nothing — but a new method on a *sealed* trait is even freer | — |
//! | new `impl` for a foreign type in a sealed trait | nothing | `E0277` for outsiders |

use std::fmt::{Debug, Display};

// ---------------------------------------------------------------------------
// Listing 3-5: a new impl can make a downstream call ambiguous
// ---------------------------------------------------------------------------

/// crate1's 1.0 trait. It exists in 1.0 and is *not* implemented for [`Unit`].
pub trait Foo1 {
    fn foo(&self) -> &'static str;
}

/// crate1's 2.0 addition: `impl Foo1 for Unit`.
///
/// The trait was visible in 1.0, so crate2 may have written `impl Foo2 for Unit`
/// and called `Unit.foo()`. Importing `Foo1` — which is what `use crate1::{Unit,
/// Foo1}` does — makes that call ambiguous:
///
/// ```compile_fail,E0034
/// use rust_for_rustaceans::designing_interfaces::constrained::trait_implementations::{
///     Foo1, Unit,
/// };
///
/// trait Foo2 {
///     fn foo(&self) -> &'static str;
/// }
///
/// impl Foo2 for Unit {
///     fn foo(&self) -> &'static str {
///         "crate2"
///     }
/// }
///
/// // multiple applicable items in scope
/// let _ = Unit.foo();
/// ```
pub struct Unit;

impl Foo1 for Unit {
    fn foo(&self) -> &'static str {
        "crate1"
    }
}

/// The same downstream code against a type crate1 does *not* implement [`Foo1`]
/// for: one candidate, so the call is fine. That is Listing 3-5 before the 2.0
/// release.
///
/// ```
/// use rust_for_rustaceans::designing_interfaces::constrained::trait_implementations::UnitBefore;
///
/// trait Foo2 {
///     fn foo(&self) -> &'static str;
/// }
///
/// impl Foo2 for UnitBefore {
///     fn foo(&self) -> &'static str {
///         "crate2"
///     }
/// }
///
/// assert_eq!(UnitBefore.foo(), "crate2");
/// ```
pub struct UnitBefore;

/// crate2's trait, kept here so the "before" half can be shown in one crate.
pub trait Foo2 {
    fn foo(&self) -> &'static str;
}

impl Foo2 for UnitBefore {
    fn foo(&self) -> &'static str {
        "crate2"
    }
}

/// > If you add `impl Foo1 for Unit` to `crate1` without marking it a breaking
/// > change, the downstream code will suddenly stop compiling since the call to
/// > `foo` is now ambiguous.
///
/// Inside crate1 the two calls are each unambiguous — `Foo1` applies to [`Unit`],
/// `Foo2` to [`UnitBefore`] — which is exactly why the author can ship the impl
/// without noticing. Only the downstream crate that put `Foo2` on `Unit` sees
/// `E0034`.
pub fn listing_3_5_a_new_impl_can_make_a_downstream_call_ambiguous() {
    assert_eq!(Unit.foo(), "crate1");
    assert_eq!(UnitBefore.foo(), "crate2");
}

// ---------------------------------------------------------------------------
// A blanket implementation reaches types that never asked
// ---------------------------------------------------------------------------

/// A derived trait: implemented for everything that is [`Debug`].
pub trait Describe {
    fn describe(&self) -> String {
        "a debuggable value".to_owned()
    }
}

impl<T: Debug> Describe for T {}

/// A fresh type. Someone might have wanted to write `impl Describe for Local`
/// themselves — which is why the blanket impl above is the breaking change:
///
/// ```compile_fail,E0119
/// use rust_for_rustaceans::designing_interfaces::constrained::trait_implementations::Describe;
///
/// #[derive(Debug)]
/// struct MyType;
///
/// // conflicting implementations of trait `Describe` for type `MyType`
/// impl Describe for MyType {}
/// ```
#[derive(Debug)]
pub struct Local;

/// > implementing traits for a _new_ type is never a problem, since no crate can
/// > have implementations that conflict with that type.
///
/// [`Local`] gets [`Describe`] without implementing it, and [`Fresh`] can take
/// whatever impls the author wants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fresh;

/// The blanket impl's promise, seen from the type that did not ask for it.
pub fn a_blanket_impl_reaches_types_that_never_asked() {
    assert_eq!(Local.describe(), "a debuggable value");

    // A new type has no conflicting impls to worry about, so this is free.
    assert_eq!(Fresh.clone(), Fresh);
    assert_eq!(format!("{:?}{:?}", Local, Fresh), "LocalFresh".to_owned());
}

// ---------------------------------------------------------------------------
// Adding a method: with a default, or without
// ---------------------------------------------------------------------------

/// An open trait with a method added after 1.0 — with a default body, so the
/// downstream implementations keep compiling:
///
/// ```
/// use rust_for_rustaceans::designing_interfaces::constrained::trait_implementations::Listener;
///
/// struct Mine;
///
/// // only the original method: still a complete implementation
/// impl Listener for Mine {
///     fn on_event(&self, event: &str) -> String {
///         event.to_uppercase()
///     }
/// }
///
/// assert_eq!(Mine.on_event("x"), "X");
/// assert_eq!(Mine.on_error(), "no handler");
/// ```
///
/// > Adding a new method with a default implementation is fine though, since
/// > existing implementations will continue to apply.
pub trait Listener {
    fn on_event(&self, event: &str) -> String;

    fn on_error(&self) -> String {
        "no handler".to_owned()
    }
}

/// The version without a default: existing downstream impls are complete no
/// longer.
///
/// ```compile_fail,E0046
/// use rust_for_rustaceans::designing_interfaces::constrained::trait_implementations::Strict;
///
/// struct Mine;
///
/// // not all trait items implemented, missing: `second`
/// impl Strict for Mine {
///     fn first(&self) -> u8 {
///         1
///     }
/// }
/// ```
pub trait Strict {
    fn first(&self) -> u8;
    fn second(&self) -> u8;
}

/// One in-crate implementation of each, so the two traits are more than type
/// declarations.
pub fn default_methods_are_additive_required_ones_are_not() {
    struct Both;

    impl Listener for Both {
        fn on_event(&self, event: &str) -> String {
            event.to_owned()
        }
    }

    impl Strict for Both {
        fn first(&self) -> u8 {
            1
        }

        fn second(&self) -> u8 {
            2
        }
    }

    assert_eq!(Both.on_event("event"), "event");
    assert_eq!(Both.on_error(), "no handler");
    assert_eq!((Both.first(), Both.second()), (1, 2));
}

// ---------------------------------------------------------------------------
// Listing 3-6: sealing a trait
// ---------------------------------------------------------------------------

/// The private module Listing 3-6 depends on: a `pub` trait nobody else can
/// reach, because the module holding it is private.
pub mod sealed {
    /// > The trick is to add a private, empty trait as a supertrait of the trait
    /// > you wish to seal.
    pub trait Sealed {}
}

/// > A sealed trait is one that can be used only, and not implemented, by other
/// > crates. This immediately makes a number of breaking changes nonbreaking.
///
/// This trait is **sealed**: it can be used but not implemented outside this
/// crate. A downstream `impl` cannot satisfy the private supertrait:
///
/// ```compile_fail,E0277
/// use rust_for_rustaceans::designing_interfaces::constrained::trait_implementations::CanUseCannotImplement;
///
/// struct Mine;
///
/// // the trait bound `Mine: sealed::Sealed` is not satisfied
/// impl CanUseCannotImplement for Mine {
///     fn render(&self) -> String {
///         String::new()
///     }
///
///     fn shout(&self) -> String {
///         String::new()
///     }
/// }
/// ```
pub trait CanUseCannotImplement: sealed::Sealed {
    fn render(&self) -> String;

    /// > you can add a new method to a sealed trait, since you know there are no
    /// > implementations outside of the current crate to consider.
    ///
    /// No default body: every implementation is in this crate, so adding the
    /// method is still not a breaking change.
    fn shout(&self) -> String;

    /// A method added with a default, which would be safe on any trait.
    fn quietly(&self) -> String {
        self.render().to_lowercase()
    }
}

/// The bound the seal accepts: any type that can be displayed.
pub trait Renderable: Display {}

impl Renderable for u8 {}

impl Renderable for &str {}

/// > The sealed trait requires the underlying type to implement `Sealed` , so only
/// > the types that we explicitly allow are able to ultimately implement the trait.
///
/// The blanket impl is also how a sealed trait gets implementations for *foreign*
/// types — safe, because the foreign crate cannot have a conflicting impl of a
/// trait it cannot name.
impl<T> sealed::Sealed for T where T: Renderable {}

impl<T> CanUseCannotImplement for T
where
    T: Renderable,
{
    fn render(&self) -> String {
        self.to_string()
    }

    fn shout(&self) -> String {
        self.render().to_uppercase()
    }
}

/// > Sealed traits are most commonly used for _derived_ traits—traits that provide
/// > blanket implementations for types that implement particular other traits.
///
/// Both halves of the Listing 3-6 trick, visible: `u8` and `&str` never implement
/// `sealed::Sealed` by hand, and the trait's three methods work on both.
pub fn listing_3_6_a_sealed_trait_can_gain_methods_and_impls() {
    assert_eq!(7u8.render(), "7");
    assert_eq!(7u8.shout(), "7");
    assert_eq!(7u8.quietly(), "7");

    assert_eq!("Hello".render(), "Hello");
    assert_eq!("Hello".shout(), "HELLO");
    assert_eq!("Hello".quietly(), "hello");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_3_5_a_new_impl_can_make_a_downstream_call_ambiguous_runs() {
        listing_3_5_a_new_impl_can_make_a_downstream_call_ambiguous();
    }

    #[test]
    fn a_blanket_impl_reaches_types_that_never_asked_runs() {
        a_blanket_impl_reaches_types_that_never_asked();
    }

    #[test]
    fn default_methods_are_additive_required_ones_are_not_runs() {
        default_methods_are_additive_required_ones_are_not();
    }

    #[test]
    fn listing_3_6_a_sealed_trait_can_gain_methods_and_impls_runs() {
        listing_3_6_a_sealed_trait_can_gain_methods_and_impls();
    }
}
