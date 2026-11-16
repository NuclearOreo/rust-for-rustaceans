//! # Type Inference and Unnameable Types
//!
//! > The compiler will usually infer types only for variables and for the
//! > arguments (and return types) of closures; top-level definitions like
//! > functions, types, traits, and trait implementation blocks all require that
//! > you explicitly name all types.
//!
//! > However, it's not always easy, or even possible, to fully name a type! For
//! > example, if you return a closure from a function … its type does not have a
//! > name that you can type into your code.
//!
//! Two boundaries: inference stops at item signatures, and some types have no
//! spelling at all. Both are demonstrated by the code that *fails* to compile.

use std::any::type_name_of_val;

/// Where inference is allowed: locals, closure arguments taken from context, and
/// the type parameter of `collect`.
#[allow(clippy::vec_init_then_push)] // creating then pushing *is* the demo: the type comes from `push`
pub fn inference_fills_in_locals_and_closure_arguments() {
    // A local's type comes from how it is used, not from an annotation.
    let mut numbers = Vec::new();
    numbers.push(1u8); // this is what makes `numbers` a `Vec<u8>`
    assert_eq!(numbers, vec![1u8]);

    // The closure's parameter type is inferred from `map`'s own signature: there
    // is no way to write this closure's type, and no need to write its input.
    let doubled: Vec<i32> = vec![1, 2, 3].into_iter().map(|value| value * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6]);

    // `collect` is the canonical "the answer comes from the annotation" case.
    let collected: Vec<u8> = (1..=3).collect();
    assert_eq!(collected, vec![1, 2, 3]);
}

/// > top-level definitions like functions, types, traits, and trait
/// > implementation blocks all require that you explicitly name all types.
///
/// The `_` placeholder that works so well in a `let` is refused in a signature:
///
/// ```compile_fail,E0121
/// fn make() -> _ {
///     1u8     // the placeholder `_` is not allowed within types on item
/// }           // signatures for return types
/// ```
///
/// > but the primary one is that type inference is much easier when you have at
/// > least some known points to start the inference from.
pub fn item_signatures_must_name_their_types() {
    // Every item below states its types; that is the price of being a known
    // point for callers.
    struct Pair {
        left: u8,
        right: u16,
    }

    fn make() -> Pair {
        Pair { left: 1, right: 2 }
    }

    let made: Pair = make();
    assert_eq!(u16::from(made.left) + made.right, 3);
}

/// A function that returns a closure. The signature can name the *traits* — `Fn(i32)
/// -> i32` — but never the closure's own type.
pub fn add_one() -> impl Fn(i32) -> i32 {
    |value| value + 1
}

/// > its type does not have a name that you can type into your code
///
/// Two closures are two distinct types even when their bodies are identical, so
/// they cannot share a binding. That is what "no name" costs: there is nothing to
/// coerce one to.
///
/// ```compile_fail,E0308
/// let mut operation = |value: i32| value + 1;
/// operation = |value: i32| value + 2;   // mismatched types: different closures
/// ```
pub fn every_closure_is_a_type_of_its_own() {
    let add_one = add_one();
    assert_eq!(add_one(41), 42);

    // Both of these are `{{closure}}` types, distinct and unnameable; the only
    // way to hold two of them together is to erase the type or make them a
    // trait object.
    let first = |value: i32| value + 1;
    let second = |value: i32| value + 2;

    assert_ne!(first(1), second(1));

    // A `type_name` exists for diagnostics, but it is not a type you can write
    // in source — note the `{{closure}}` placeholder.
    let name = type_name_of_val(&first);
    assert!(name.contains("closure"), "unexpected name: {name}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inference_fills_in_locals_and_closure_arguments_runs() {
        inference_fills_in_locals_and_closure_arguments();
    }

    #[test]
    fn item_signatures_must_name_their_types_runs() {
        item_signatures_must_name_their_types();
    }

    #[test]
    fn every_closure_is_a_type_of_its_own_runs() {
        every_closure_is_a_type_of_its_own();
    }
}
