//! # Compilation and Dispatch
//!
//! > When you write a type or function that is generic over `T`, you're really
//! > telling the compiler to make a copy of that type or function for each type
//! > `T`.
//!
//! > This process of going from a generic type to many non-generic types is
//! > called _monomorphization_, and it's part of the reason generic Rust code
//! > usually performs just as well as non-generic code.
//!
//! > The alternative to static dispatch is _dynamic dispatch_, which enables
//! > code to call a trait method on a generic type without knowing what that
//! > type is.
//!
//! The book illustrates both with `impl String { fn contains(&self, p: impl
//! Pattern) }`. `String` cannot be extended from outside `std`, so the demos use
//! a free function and a `Pattern` trait of our own; the shape is identical.
//!
//! Monomorphization is not taken on faith here. Two instantiations of a generic
//! function are two different functions with two different addresses, and
//! `type_name` reports the substituted type — both are observable from inside a
//! running test.

use std::mem::size_of;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// The example trait
// ---------------------------------------------------------------------------

/// The book's `Pattern`: something that can be looked for inside a string.
pub trait Pattern {
    fn is_contained_in(&self, haystack: &str) -> bool;
}

impl Pattern for char {
    fn is_contained_in(&self, haystack: &str) -> bool {
        haystack.contains(*self)
    }
}

impl Pattern for &str {
    fn is_contained_in(&self, haystack: &str) -> bool {
        haystack.contains(*self)
    }
}

// ---------------------------------------------------------------------------
// Static dispatch
// ---------------------------------------------------------------------------

/// Listing 2-2: a generic method using static dispatch.
///
/// > A copy of this method is made for every distinct pattern type (recall that
/// > `impl Trait` is shorthand for `<T: Trait>`). We need a different copy of the
/// > function body for each `impl Pattern` type because we need to know the
/// > address of the `is_contained_in` function to call it.
///
/// `contains` below is exactly `impl Pattern` in argument position: two
/// monomorphizations, one for `char` and one for `&str`, each with the address of
/// that type's `is_contained_in` baked in.
pub fn listing_2_2_static_dispatch() {
    fn contains(haystack: &str, pattern: impl Pattern) -> bool {
        pattern.is_contained_in(haystack)
    }

    assert!(contains("hello", 'e')); // monomorphized for `char`
    assert!(contains("hello", "ell")); // monomorphized for `&str`
    assert!(!contains("hello", 'z'));
    assert!(!contains("hello", "elll"));
}

/// > This is referred to as _static dispatch_, since for any given copy of the
/// > method, the address we are "dispatching to" is known statically.
///
/// Two instantiations are two functions. Their addresses differ, and each knows
/// its own concrete type — that is what "one copy per `T`" means.
pub fn every_instantiation_is_its_own_function() {
    fn helper<T: Default>() -> usize {
        // The body differs per `T`, so the copies cannot be merged.
        size_of::<T>()
    }

    fn name_of<T>(_: &T) -> &'static str {
        std::any::type_name::<T>()
    }

    assert_eq!(name_of(&0u8), "u8");
    assert_eq!(name_of(&0u16), "u16");

    let byte_copy = helper::<u8> as fn() -> usize;
    let word_copy = helper::<u16> as fn() -> usize;

    // Distinct function items, distinct addresses. (In an optimised build the
    // compiler is allowed to merge *identical* functions; these are not
    // identical, which is why the claim survives optimisation.)
    assert_ne!(byte_copy as usize, word_copy as usize);

    assert_eq!(byte_copy(), 1);
    assert_eq!(word_copy(), 2);
}

/// > Often, much of the code in a generic method is not type-dependent. …
/// > One pattern you can use for cases like this is to declare a non-generic
/// > helper function inside the generic method that performs the shared
/// > operations. This leaves only the type-dependent code for the compiler to
/// > copy-paste for you while allowing the helper function to be shared.
///
/// The observable difference between the two kinds of helper: the non-generic
/// one has *one* address no matter which instantiation calls it, while a generic
/// helper would have one address per instantiation (as in
/// [`every_instantiation_is_its_own_function`]).
pub fn a_non_generic_inner_helper_is_shared_by_every_instantiation() {
    /// Type-independent work: one copy of this code, ever.
    fn fold_bytes(bytes: &[u8]) -> u64 {
        bytes.iter().map(|byte| u64::from(*byte)).sum()
    }

    fn record<T: Into<u64>>(value: T) -> (u64, usize) {
        // Type-dependent part: only this conversion needs `T`.
        let as_u64: u64 = value.into();

        // Type-independent part: the same function for every `T`.
        let work: fn(&[u8]) -> u64 = fold_bytes;
        (work(&as_u64.to_ne_bytes()), work as usize)
    }

    let (byte_total, byte_address) = record(1u8);
    let (word_total, word_address) = record(2u16);

    assert_eq!(byte_total, 1);
    assert_eq!(word_total, 2);

    // One `fold_bytes`, shared by both monomorphizations of `record`.
    assert_eq!(byte_address, word_address);
}

// ---------------------------------------------------------------------------
// Dynamic dispatch
// ---------------------------------------------------------------------------

/// Listing 2-3: a generic method using dynamic dispatch.
///
/// > If you replace `impl Pattern` with `&dyn Pattern`, you tell the caller that
/// > they must give _two_ pieces of information for this argument: the address of
/// > the pattern _and_ the address of the `is_contained_in` method.
///
/// One copy of this function exists for all pattern types. The answers match the
/// static version exactly, because both end up calling the same implementation.
pub fn listing_2_3_dynamic_dispatch() {
    fn contains(haystack: &str, pattern: &dyn Pattern) -> bool {
        pattern.is_contained_in(haystack)
    }

    assert!(contains("hello", &'e'));
    assert!(contains("hello", &"ell"));
    assert!(!contains("hello", &'z'));
}

/// > In practice, the caller gives us a pointer to a chunk of memory called a
/// > virtual method table, or _vtable_, that holds the address of the
/// > implementation of _all_ the trait's methods for the type in question.
///
/// The vtable pointer is the wide pointer's second word — which is why a
/// `&dyn Pattern` costs two words, and why it works from behind any owning
/// pointer.
pub fn dynamic_dispatch_carries_a_vtable_pointer() {
    let char_pattern = 'e';
    let str_pattern = "ell";

    let boxed: Box<dyn Pattern> = Box::new('e');
    let shared: Arc<dyn Pattern> = Arc::new("ell");

    // > You can use any type that is able to hold a wide pointer for dynamic
    // > dispatch, such as `&mut`, `Box`, and `Arc`.
    assert_eq!(size_of::<&dyn Pattern>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<&mut dyn Pattern>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<Box<dyn Pattern>>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<Arc<dyn Pattern>>(), 2 * size_of::<usize>());

    // Different concrete types, different vtables, one shared call site.
    assert!(char_pattern.is_contained_in("hello"));
    assert!(str_pattern.is_contained_in("hello"));
    assert!(boxed.is_contained_in("hello"));
    assert!(shared.is_contained_in("hello"));
}

/// > when we opted in to dynamic dispatch using the `dyn` keyword, we had to
/// > place an `&` in front of it. The reason is that we no longer know at compile
/// > time the size of the pattern type that the caller passes in … In other
/// > words, `dyn Trait` is `!Sized`.
///
/// ```compile_fail,E0277
/// trait Pattern {
///     fn is_contained_in(&self, haystack: &str) -> bool;
/// }
///
/// let pattern: dyn Pattern;   // the size for values of type `dyn Pattern`
///                             // cannot be known at compilation time
/// ```
pub fn dyn_is_unsized_so_it_needs_a_pointer() {
    let value = 'e';

    // Behind a reference it is sized: the reference is the two-word handle.
    let object: &dyn Pattern = &value;
    assert_eq!(size_of::<&dyn Pattern>(), 2 * size_of::<usize>());

    // And the vtable lets `size_of_val` answer for the unsized value behind it,
    // even though `size_of::<dyn Pattern>` cannot be written.
    assert_eq!(std::mem::size_of_val(object), size_of::<char>());

    assert!(object.is_contained_in("hello"));
}

// ---------------------------------------------------------------------------
// Object safety
// ---------------------------------------------------------------------------

/// > Most traits can be turned into trait objects, but not all.
///
/// `Clone` returns `Self`, and a vtable has no room for a return type that is
/// only known at run time:
///
/// ```compile_fail,E0038
/// let object: &dyn Clone;   // the trait `Clone` is not dyn compatible
/// ```
///
/// `Extend::extend` is generic over the iterator, so there is no single address
/// to put in the vtable:
///
/// ```compile_fail,E0038
/// let object: &dyn Extend<i32>;
/// ```
///
/// > To be object-safe, none of a trait's methods can be generic or use the
/// > `Self` type.
///
/// ```compile_fail,E0038
/// trait Generic {
///     fn apply<X>(&self, value: X);
/// }
///
/// let object: &dyn Generic;
/// ```
///
/// > Furthermore, the trait cannot have any static methods (that is, methods
/// > whose first argument does not dereference to `Self`)
///
/// ```compile_fail,E0038
/// trait Factory {
///     fn create() -> Self;
/// }
///
/// let object: &dyn Factory;
/// ```
///
/// The positive half: a trait whose methods all take `&self` and never mention
/// `Self` in a return type is perfectly usable behind `dyn`.
pub fn not_every_trait_is_object_safe() {
    let object: &dyn Pattern = &'e';

    assert!(object.is_contained_in("hello"));

    // The vtable entries are the only thing that varies; the call site does not.
    let mut through_dyn: Box<dyn Pattern> = Box::new("ell");
    assert!(through_dyn.is_contained_in("hello"));
    through_dyn = Box::new('x');
    assert!(!through_dyn.is_contained_in("hello"));
}

/// > When reading about trait objects, you may see mentions of the trait bound
/// > `Self: Sized`. … Methods with a `where Self: Sized` bound are exempted when
/// > checking if a trait is object-safe.
///
/// The exemption is what lets a trait mix object-safe methods with generic ones:
/// `get` is in the vtable, `collect_all` is not (it could not be), and the trait
/// is still a legal `dyn` type.
///
/// Calling `collect_all` on a `&dyn Store` is refused — with a diagnostic that
/// carries no error code ("the `collect_all` method cannot be invoked on a trait
/// object"), which is why that half of the claim is prose rather than a doctest.
pub trait Store {
    fn get(&self) -> u8;

    fn collect_all<T>(&self, items: Vec<T>) -> usize
    where
        Self: Sized,
    {
        items.len() + usize::from(self.get())
    }
}

pub struct ByteStore(pub u8);

impl Store for ByteStore {
    fn get(&self) -> u8 {
        self.0
    }
}

pub fn self_sized_exempts_a_method_from_object_safety() {
    let store = ByteStore(1);

    // The sized path still has the generic method.
    assert_eq!(store.collect_all(vec![1, 2, 3]), 4);

    // And the trait is object-safe because that method is exempted.
    let object: &dyn Store = &store;
    assert_eq!(object.get(), 1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_2_2_static_dispatch_runs() {
        listing_2_2_static_dispatch();
    }

    #[test]
    fn every_instantiation_is_its_own_function_runs() {
        every_instantiation_is_its_own_function();
    }

    #[test]
    fn a_non_generic_inner_helper_is_shared_by_every_instantiation_runs() {
        a_non_generic_inner_helper_is_shared_by_every_instantiation();
    }

    #[test]
    fn listing_2_3_dynamic_dispatch_runs() {
        listing_2_3_dynamic_dispatch();
    }

    #[test]
    fn dynamic_dispatch_carries_a_vtable_pointer_runs() {
        dynamic_dispatch_carries_a_vtable_pointer();
    }

    #[test]
    fn dyn_is_unsized_so_it_needs_a_pointer_runs() {
        dyn_is_unsized_so_it_needs_a_pointer();
    }

    #[test]
    fn not_every_trait_is_object_safe_runs() {
        not_every_trait_is_object_safe();
    }

    #[test]
    fn self_sized_exempts_a_method_from_object_safety_runs() {
        self_sized_exempts_a_method_from_object_safety();
    }
}
