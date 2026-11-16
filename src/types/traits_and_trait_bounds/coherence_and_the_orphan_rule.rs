//! # Coherence and the Orphan Rule
//!
//! > These rules exist to preserve the coherence property: for any given type
//! > and method, there is only ever one correct choice for which implementation
//! > of the method to use for that type.
//!
//! > Simply stated, the orphan rule says that you can implement a trait for a
//! > type only if the trait _or_ the type is local to your crate. So, you can
//! > implement `Debug` for your own type, and you can implement `MyNeatTrait`
//! > for `bool`, but you cannot implement `Debug` for `bool`.
//!
//! # Why these demos are doctests
//!
//! Inside this crate, everything the crate defines is local, so no code here can
//! violate the orphan rule — which makes the rule untestable *from* the crate.
//! The rule is about crate boundaries, so the demos cross one: a doctest is
//! compiled as its own crate, with `rust_for_rustaceans` as a dependency. In
//! that context [`ForeignTrait`] is foreign, `std`'s `Vec` and `From` are
//! foreign, and a local `MyType` is local — exactly the situation the listings
//! describe.
//!
//! The runnable demos below are the two *allowed* halves, checked by the fact
//! that they compile: our trait on a foreign type, and a foreign trait on our
//! type.

use std::fmt;

// ---------------------------------------------------------------------------
// The four kinds of implementation
// ---------------------------------------------------------------------------

/// Our own trait, so `impl Label for bool` is allowed: the *trait* is local.
pub trait Label {
    fn label(&self) -> &'static str;
}

/// A second trait of ours, used for a blanket implementation.
pub trait Shout {
    fn shout(&self) -> String;
}

/// Our own type, so `impl fmt::Display for LocalType` is allowed: the *type* is
/// local.
pub struct LocalType(pub u32);

/// A collection used to show why `&` is a *fundamental* type.
pub struct Bag(pub Vec<u8>);

/// > a given `impl<P1..=Pn> ForeignTrait<T1..=Tn> for T0` is allowed only if at
/// > least one `Ti` is a local type and no `T` before the first such `Ti` is one
/// > of the generic types `P1..=Pn`.
///
/// This trait exists only to be foreign in a doctest — it has two type
/// parameters so that Listings 2-4 and 2-5 can be reproduced exactly,
/// including the ordering subtlety in the book's note.
pub trait ForeignTrait<A, B> {
    fn probe(&self) -> (&'static str, &'static str);
}

// The *trait* is local, so this is allowed even though `bool` is foreign.
impl Label for bool {
    fn label(&self) -> &'static str {
        if *self { "true" } else { "false" }
    }
}

// The *type* is local, so this is allowed even though `Display` is foreign.
impl fmt::Display for LocalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LocalType({})", self.0)
    }
}

impl Label for LocalType {
    fn label(&self) -> &'static str {
        "local"
    }
}

/// > Only the crate that defines a trait is allowed to write a blanket
/// > implementation
impl<T: Label + ?Sized> Shout for T {
    fn shout(&self) -> String {
        self.label().to_uppercase()
    }
}

/// > With just the orphan rule, this implementation would not be permitted since
/// > it implements a foreign trait for a foreign type — `IntoIterator` and `&`
/// > both come from the standard library.
///
/// `&` is a *fundamental* type, so it is erased before the check and `Bag` is
/// what remains:
///
/// > For the purposes of the orphan rule, fundamental types may as well not
/// > exist — they are effectively erased before the orphan rule is checked in
/// > order to allow you to, for example, implement `IntoIterator for &MyType`.
impl<'a> IntoIterator for &'a Bag {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// > The simplest example of this is when you want to write something like
/// > `impl From<MyType> for Vec<i32>`. Here, the `From` trait is foreign, as is
/// > the `Vec` type, yet there is no danger of violating coherence.
impl From<LocalType> for Vec<i32> {
    fn from(value: LocalType) -> Vec<i32> {
        vec![value.0 as i32]
    }
}

// ---------------------------------------------------------------------------
// The allowed halves, demonstrated by compiling
// ---------------------------------------------------------------------------

/// > you can implement `MyNeatTrait` for `bool`
pub fn your_own_trait_for_a_foreign_type_is_allowed() {
    assert_eq!(true.label(), "true");
    assert_eq!(false.label(), "false");
}

/// > you can implement `Debug` for your own type
pub fn a_foreign_trait_for_your_own_type_is_allowed() {
    assert_eq!(LocalType(7).to_string(), "LocalType(7)");
}

/// A blanket implementation is "not limited to just one particular type but
/// instead applies to a wide range of types" — here, everything that carries a
/// [`Label`], foreign or local.
pub fn a_blanket_implementation_covers_every_qualifying_type() {
    assert_eq!(true.shout(), "TRUE");
    assert_eq!(LocalType(1).shout(), "LOCAL");
}

/// > For the purposes of the orphan rule, fundamental types may as well not
/// > exist
pub fn fundamental_types_are_erased_before_the_check() {
    let bag = Bag(vec![1, 2, 3]);

    let mut total = 0u8;
    for byte in &bag {
        total += *byte;
    }

    assert_eq!(total, 6);
}

/// > The simplest example of this is when you want to write something like
/// > `impl From<MyType> for Vec<i32>`
pub fn covered_implementations_are_allowed() {
    let converted: Vec<i32> = Vec::from(LocalType(5));

    assert_eq!(converted, vec![5]);
}

// ---------------------------------------------------------------------------
// Listing 2-4: the allowed shapes
// ---------------------------------------------------------------------------

/// Listing 2-4: valid implementations of foreign traits for foreign types.
///
/// > So, all the implementations in Listing 2-4 are valid.
///
/// ```rust
/// use rust_for_rustaceans::types::traits_and_trait_bounds::coherence_and_the_orphan_rule::ForeignTrait;
///
/// struct MyType;
/// struct MyTypeOf<T>(T);
///
/// // Local type as the target of the foreign trait.
/// impl<T> From<T> for MyTypeOf<T> {
///     fn from(value: T) -> Self { MyTypeOf(value) }
/// }
///
/// // Local type inside the foreign type — `impl<T> From<MyType> for Vec<T>`.
/// impl<T> From<MyType> for Vec<T> {
///     fn from(_: MyType) -> Vec<T> { Vec::new() }
/// }
///
/// // Local type in argument position, *before* the bare `T`:
/// // `impl<T> ForeignTrait<MyType, T> for Vec<T>`.
/// impl<T> ForeignTrait<MyType, T> for Vec<T> {
///     fn probe(&self) -> (&'static str, &'static str) { ("Vec", "MyType") }
/// }
///
/// assert_eq!(MyTypeOf::from(1u8).0, 1);
/// assert_eq!(Vec::<u8>::from(MyType).len(), 0);
/// assert_eq!(Vec::<u8>::new().probe(), ("Vec", "MyType"));
/// ```
pub fn listing_2_4_covered_implementations_are_valid() {
    // The in-crate counterpart: `From<LocalType> for Vec<i32>`, above.
    assert_eq!(Vec::<i32>::from(LocalType(5)), vec![5]);
}

/// Listing 2-4's first shape is orphan-rule-legal but collides with the standard
/// library's reflexive `impl<T> From<T> for T` at `T = MyType`, so it no longer
/// compiles for a different reason. Worth knowing: "coherence" is about there
/// being one candidate, and the orphan rule is only one of the ways a second
/// candidate can appear.
///
/// ```compile_fail,E0119
/// struct MyType;
///
/// impl<T> From<T> for MyType {
///     fn from(_: T) -> MyType { MyType }
/// }
/// // conflicting implementations of trait `From<MyType>` for type `MyType`
/// ```
pub fn listing_2_4_first_shape_now_conflicts_with_std() {
    // The legal stand-in is a *generic* local target, which cannot overlap.
    struct MyTypeOf<T>(T);

    impl<T> From<T> for MyTypeOf<T> {
        fn from(value: T) -> MyTypeOf<T> {
            MyTypeOf(value)
        }
    }

    assert_eq!(MyTypeOf::from(9u16).0, 9);
}

// ---------------------------------------------------------------------------
// Listing 2-5: the rejected shapes
// ---------------------------------------------------------------------------

/// Listing 2-5: invalid implementations of foreign traits for foreign types.
///
/// > However, the implementations in Listing 2-5 are invalid.
///
/// A bare type parameter as the target — `impl<T> ForeignTrait for T`:
///
/// ```compile_fail,E0210
/// impl<T> std::fmt::Display for T {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "x")
///     }
/// }
/// ```
///
/// The reflexive `From`, `impl<T> From<T> for T`:
///
/// ```compile_fail,E0210
/// impl<T> From<T> for T {
///     fn from(value: T) -> T { value }
/// }
/// ```
///
/// Moving *out* of the foreign type, `impl<T> From<Vec<T>> for T`:
///
/// ```compile_fail,E0210
/// impl<T> From<Vec<T>> for T {
///     fn from(_: Vec<T>) -> T { unimplemented!() }
/// }
/// ```
///
/// `impl<T> From<T> for Vec<T>` — the target is foreign and no local type
/// appears anywhere:
///
/// ```compile_fail,E0210
/// impl<T> From<T> for Vec<T> {
///     fn from(_: T) -> Vec<T> { Vec::new() }
/// }
/// ```
///
/// And the ordering rule from the book's note: a local type that appears *after*
/// a bare type parameter does not rescue the implementation.
/// `impl<T> ForeignTrait<T, MyType> for Vec<T>` is invalid even though
/// `impl<T> ForeignTrait<MyType, T> for Vec<T>` is fine.
///
/// ```compile_fail,E0210
/// use rust_for_rustaceans::types::traits_and_trait_bounds::coherence_and_the_orphan_rule::ForeignTrait;
///
/// struct MyType;
///
/// impl<T> ForeignTrait<T, MyType> for Vec<T> {
///     fn probe(&self) -> (&'static str, &'static str) { ("Vec", "MyType") }
/// }
/// ```
pub fn listing_2_5_uncovered_implementations_are_rejected() {
    // There is nothing to run: the point of this demo is that the five blocks
    // above do not compile. The line below is the same *bound* the rejections
    // are about, used the way one legitimately can be.
    fn every_pair_is_usable<A, B, T: ForeignTrait<A, B>>(
        value: &T,
    ) -> (&'static str, &'static str) {
        value.probe()
    }

    // In *this* crate `ForeignTrait` is local, so a local implementation is
    // unremarkable here — which is exactly why the rule needs a doctest.
    struct Local;

    impl ForeignTrait<Local, u8> for Vec<Local> {
        fn probe(&self) -> (&'static str, &'static str) {
            ("Vec", "Local")
        }
    }

    assert_eq!(every_pair_is_usable(&Vec::<Local>::new()), ("Vec", "Local"));
}

// ---------------------------------------------------------------------------
// The motivating case
// ---------------------------------------------------------------------------

/// > consider what would happen if I could write my own implementation of the
/// > `Display` trait for the `bool` type from the standard library. Now, for any
/// > code that tries to print a `bool` value and includes my crate, the compiler
/// > won't know whether to pick the implementation I wrote or the one from the
/// > standard library.
///
/// ```compile_fail,E0117
/// impl std::fmt::Debug for bool {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "bool")
///     }
/// }
/// // only traits defined in the current crate can be implemented for primitive types
/// ```
pub fn coherence_is_why_this_is_refused() {
    // What the rule buys: `print` below has exactly one implementation to call,
    // chosen by the type alone.
    fn print(value: bool) -> String {
        format!("{value}")
    }

    assert_eq!(print(true), "true");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn your_own_trait_for_a_foreign_type_is_allowed_runs() {
        your_own_trait_for_a_foreign_type_is_allowed();
    }

    #[test]
    fn a_foreign_trait_for_your_own_type_is_allowed_runs() {
        a_foreign_trait_for_your_own_type_is_allowed();
    }

    #[test]
    fn a_blanket_implementation_covers_every_qualifying_type_runs() {
        a_blanket_implementation_covers_every_qualifying_type();
    }

    #[test]
    fn fundamental_types_are_erased_before_the_check_runs() {
        fundamental_types_are_erased_before_the_check();
    }

    #[test]
    fn covered_implementations_are_allowed_runs() {
        covered_implementations_are_allowed();
    }

    #[test]
    fn listing_2_4_covered_implementations_are_valid_runs() {
        listing_2_4_covered_implementations_are_valid();
    }

    #[test]
    fn listing_2_4_first_shape_now_conflicts_with_std_runs() {
        listing_2_4_first_shape_now_conflicts_with_std();
    }

    #[test]
    fn listing_2_5_uncovered_implementations_are_rejected_runs() {
        listing_2_5_uncovered_implementations_are_rejected();
    }

    #[test]
    fn coherence_is_why_this_is_refused_runs() {
        coherence_is_why_this_is_refused();
    }
}
