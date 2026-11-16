//! # Trait Bounds
//!
//! > trait bounds do not have to be of the form `T: Trait` where `T` is some type
//! > your implementation or type is generic over. The bounds can be arbitrary
//! > type restrictions and do not even need to include generic parameters, types
//! > of arguments, or local types.
//!
//! > you may find that you need to write a bound that talks about references to a
//! > type. … you want the bound to say "this reference implements this trait for
//! > any lifetime." This type of bound is known as a _higher-ranked trait bound_.
//!
//! The subsections covered: arbitrary `where` clauses, `#[derive]`'s hidden
//! bound, bounds on associated types, higher-ranked bounds, and Listing 2-6.

use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::{self, Formatter};
use std::io;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Bounds that name no generic parameter
// ---------------------------------------------------------------------------

/// > You can write a trait bound like `where String: Clone`, even though
/// > `String: Clone` is always true and contains no local types.
pub fn bounds_need_not_mention_a_generic_parameter() {
    fn render<T: Debug>(value: &T) -> String
    where
        String: Clone, // always true; still checked, still a bound
    {
        let rendered = format!("{value:?}");
        rendered.clone()
    }

    assert_eq!(render(&1u8), "1");
    assert_eq!(render(&"hi"), "\"hi\"");
}

/// An error type of ours, used to show a bound whose generic parameter appears
/// only on the *right*.
pub struct MyError<T>(pub T);

/// > `impl<T> From<MyError<T>> for std::io::Error` is a *covered*
/// > implementation: the trait (`From`) and the target (`io::Error`) are both
/// > foreign, but the local `MyError<T>` appears as a type argument.
impl<T: Display> From<MyError<T>> for io::Error {
    fn from(error: MyError<T>) -> io::Error {
        io::Error::other(format!("{}", error.0))
    }
}

/// > You can also write `where io::Error: From<MyError<T>>`; your generic type
/// > parameters do not need to appear only on the left-hand side.
pub fn a_bound_may_put_the_generic_parameter_on_the_right() {
    fn fail_with<T: Display>(value: T) -> io::Error
    where
        io::Error: From<MyError<T>>,
    {
        MyError(value).into()
    }

    assert_eq!(fail_with("boom").to_string(), "boom");
    assert_eq!(fail_with(42).to_string(), "42");
}

/// > if your method wants to construct a `HashMap<K, V, S>` … instead of writing
/// > the bounds out like `where T: Hash + Eq, S: BuildHasher + Default`, you
/// > could write `where HashMap<T, usize, S>: FromIterator`. This saves you from
/// > looking up the exact bounds requirements for the methods you end up using.
///
/// Both functions below do the same job; the first states the *requirement*, the
/// second restates the standard library's *implementation* bounds. The first
/// keeps working if `HashMap`'s bounds ever change.
pub fn a_bound_on_a_whole_foreign_type_replaces_inner_bounds() {
    fn collect_pairs<K, V, S>(pairs: impl IntoIterator<Item = (K, V)>) -> HashMap<K, V, S>
    where
        HashMap<K, V, S>: FromIterator<(K, V)>,
    {
        pairs.into_iter().collect()
    }

    fn collect_pairs_explicit<K, V, S>(pairs: impl IntoIterator<Item = (K, V)>) -> HashMap<K, V, S>
    where
        K: std::hash::Hash + Eq,
        S: std::hash::BuildHasher + Default,
    {
        pairs.into_iter().collect()
    }

    let concise: HashMap<&str, usize> = collect_pairs([("a", 1), ("b", 2)]);
    let explicit: HashMap<&str, usize> = collect_pairs_explicit([("a", 1), ("b", 2)]);

    assert_eq!(concise, explicit);
    assert_eq!(concise["b"], 2);
}

// ---------------------------------------------------------------------------
// The hidden bound in #[derive]
// ---------------------------------------------------------------------------

/// > Many `#[derive(Trait)]` expansions desugar into
/// > `impl Trait for Foo<T> where T: Trait`. This is often what you want, but not
/// > always. … consider what happens if we try to derive `Clone` this way for
/// > `Foo<T>` and `Foo` contains an `Arc<T>`. `Arc` implements `Clone`
/// > _regardless_ of whether `T` implements `Clone`, but due to the derived
/// > bounds, `Foo` will implement `Clone` only if `T` does!
#[derive(Clone, Debug)]
pub struct DerivedShared<T>(pub Arc<T>);

/// The same type, with the bound the derive should have written — none on `T`.
/// `Arc<T>: Clone` holds for every `T`, so this impl needs nothing from `T`.
pub struct Shared<T>(pub Arc<T>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(Arc::clone(&self.0))
    }
}

/// A type that deliberately cannot be cloned.
pub struct NotClone;

/// The derived version asks for a bound it does not need:
///
/// ```compile_fail,E0277
/// use std::sync::Arc;
/// use rust_for_rustaceans::types::traits_and_trait_bounds::trait_bounds::{
///     DerivedShared, NotClone,
/// };
///
/// let shared = DerivedShared(Arc::new(NotClone));
/// let copy = shared.clone();   // `NotClone: Clone` is not satisfied
/// ```
pub fn derive_adds_a_bound_you_did_not_ask_for() {
    let inner = Arc::new(NotClone);
    let shared = Shared(Arc::clone(&inner));

    // Cloning the handle clones the `Arc`, not the `T` — so `NotClone` is fine.
    let copy = shared.clone();

    assert_eq!(Arc::strong_count(&inner), 3); // `inner`, `shared`, `copy`
    let _ = copy;

    // The derived type would need `T: Clone`, so it cannot even be built with a
    // `NotClone` *and* cloned; that is the doctest above.
    let derived = DerivedShared(Arc::new(1u8));
    assert_eq!(*derived.clone().0, 1);
}

// ---------------------------------------------------------------------------
// Bounds on associated types
// ---------------------------------------------------------------------------

/// > consider the iterator method `flatten` … The type it produces, `Flatten`,
/// > is generic over `I` … `Flatten` implements `Iterator` if `I` implements
/// > `Iterator` _and_ the items yielded by `I` themselves implement
/// > `IntoIterator`. To enable you to write bounds like this, Rust lets you
/// > refer to associated types of a type using the syntax `Type::AssocType`.
pub fn bounds_can_name_associated_types() {
    fn flatten_into<I>(outer: I) -> Vec<<I::Item as IntoIterator>::Item>
    where
        I: Iterator,
        I::Item: IntoIterator,
    {
        outer.flatten().collect()
    }

    let nested = vec![vec![1u8, 2], vec![3]];
    assert_eq!(flatten_into(nested.into_iter()), vec![1, 2, 3]);

    // The same function over a different inner collection type: the bound is on
    // `I::Item`, not on `I`.
    let letters: Vec<[char; 2]> = vec![['a', 'b'], ['c', 'd']];
    assert_eq!(flatten_into(letters.into_iter()), vec!['a', 'b', 'c', 'd']);
}

// ---------------------------------------------------------------------------
// Listing 2-6
// ---------------------------------------------------------------------------

/// A collection wrapper, so that Listing 2-6 has something to be generic over.
pub struct AnyIterable<C>(pub C);

impl<'a, C> IntoIterator for &'a AnyIterable<C>
where
    &'a C: IntoIterator,
{
    type Item = <&'a C as IntoIterator>::Item;
    type IntoIter = <&'a C as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

/// Listing 2-6: an excessively generic implementation of `Debug` for any
/// iterable collection.
///
/// > You could copy-paste this implementation for pretty much any collection
/// > type and it would "just work."
impl<C> Debug for AnyIterable<C>
where
    for<'a> &'a Self: IntoIterator,
    for<'a> <&'a Self as IntoIterator>::Item: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

/// The impl is written once and works for two unrelated collection types,
/// because every requirement is stated as a bound rather than as a concrete
/// type.
pub fn listing_2_6_debug_for_any_iterable() {
    let numbers = AnyIterable(vec![1, 2, 3]);
    let bytes = AnyIterable([9u8, 8]);

    assert_eq!(format!("{numbers:?}"), "[1, 2, 3]");
    assert_eq!(format!("{bytes:?}"), "[9, 8]");
}

// ---------------------------------------------------------------------------
// Higher-ranked trait bounds
// ---------------------------------------------------------------------------

/// > say you want to be generic over a function that takes a reference to a `T`
/// > and returns a reference to _inside_ that `T`. … Using a higher-ranked
/// > lifetime, you can write `F: for<'a> Fn(&'a T) -> &'a U` to say that for
/// > _any_ lifetime `'a`, the bound must hold.
///
/// `twice` calls `f` on two locals that live for two *different*, very short
/// lifetimes. Only a bound that quantifies over all lifetimes can accept that.
pub fn higher_ranked_bounds_quantify_over_every_lifetime() {
    fn twice<F>(f: F) -> (usize, usize)
    where
        F: for<'a> Fn(&'a u8) -> &'a u8,
    {
        let first = 1u8;
        let second = 2u8;

        (
            f(&first) as *const u8 as usize,
            f(&second) as *const u8 as usize,
        )
    }

    /// > The Rust compiler is smart enough that it automatically adds the `for`
    /// > when you write `Fn` bounds with references like this.
    ///
    /// The elided bound below is the same bound as the explicit one above.
    fn twice_elided<F>(f: F) -> (usize, usize)
    where
        F: Fn(&u8) -> &u8,
    {
        let first = 1u8;
        let second = 2u8;

        (
            f(&first) as *const u8 as usize,
            f(&second) as *const u8 as usize,
        )
    }

    let explicit = twice(|value| value);
    let elided = twice_elided(|value| value);

    // Each call saw two distinct locals, i.e. two distinct lifetimes.
    assert_ne!(explicit.0, explicit.1);
    assert_ne!(elided.0, elided.1);
}

/// Pinning the bound to a single named lifetime instead of quantifying over all
/// of them is rejected, because the two locals cannot both live for the caller's
/// `'a`:
///
/// ```compile_fail,E0597
/// fn twice<'a, F>(f: F) -> (usize, usize)
/// where
///     F: Fn(&'a u8) -> &'a u8,     // one lifetime, chosen by the caller
/// {
///     let first = 1u8;
///     let second = 2u8;
///
///     (
///         f(&first) as *const u8 as usize,
///         f(&second) as *const u8 as usize,
///     )
/// }
/// ```
pub fn a_single_lifetime_is_not_a_higher_ranked_bound() {
    // The same program, with the single lifetime pushed out to the caller: it
    // compiles only when the caller owns values that outlive the call.
    fn twice<'a, F>(f: F, first: &'a u8, second: &'a u8) -> (usize, usize)
    where
        F: Fn(&'a u8) -> &'a u8,
    {
        (
            f(first) as *const u8 as usize,
            f(second) as *const u8 as usize,
        )
    }

    let first = 1u8;
    let second = 2u8;
    let addresses = twice(|value| value, &first, &second);

    assert_ne!(addresses.0, addresses.1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bounds_need_not_mention_a_generic_parameter_runs() {
        bounds_need_not_mention_a_generic_parameter();
    }

    #[test]
    fn a_bound_may_put_the_generic_parameter_on_the_right_runs() {
        a_bound_may_put_the_generic_parameter_on_the_right();
    }

    #[test]
    fn a_bound_on_a_whole_foreign_type_replaces_inner_bounds_runs() {
        a_bound_on_a_whole_foreign_type_replaces_inner_bounds();
    }

    #[test]
    fn derive_adds_a_bound_you_did_not_ask_for_runs() {
        derive_adds_a_bound_you_did_not_ask_for();
    }

    #[test]
    fn bounds_can_name_associated_types_runs() {
        bounds_can_name_associated_types();
    }

    #[test]
    fn listing_2_6_debug_for_any_iterable_runs() {
        listing_2_6_debug_for_any_iterable();
    }

    #[test]
    fn higher_ranked_bounds_quantify_over_every_lifetime_runs() {
        higher_ranked_bounds_quantify_over_every_lifetime();
    }

    #[test]
    fn a_single_lifetime_is_not_a_higher_ranked_bound_runs() {
        a_single_lifetime_is_not_a_higher_ranked_bound();
    }
}
