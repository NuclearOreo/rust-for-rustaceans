//! # Generic Arguments
//!
//! > One obvious requirement your interface must place on users is what types
//! > they must provide to your code. If your function explicitly takes a `Foo` ,
//! > the user must own and give you a `Foo` . There is no way around it.
//!
//! > In most cases it pays off to use generics rather than concrete types, to
//! > allow the caller to pass any type that conforms to what your function
//! > actually needs, rather than only a particular type.
//!
//! > A good rule of thumb is to make an argument generic if you can think of
//! > other types a user might reasonably and frequently want to use instead of
//! > the concrete type you started with.
//!
//! Three functions sum integers, with three widths of requirement:
//!
//! | Function | Requirement | Call sites it accepts |
//! | --- | --- | --- |
//! | [`sum_concrete`] | exactly `&Vec<usize>` | a `Vec<usize>` reference, nothing else |
//! | [`sum_slice`] | a `&[usize]` | `Vec`, array, or slice — via deref coercion |
//! | [`sum_as_ref`] | `impl AsRef<[usize]>` | all of the above, plus owned values and other people's types |
//!
//! The second half of the subsection is about *not* making the generic choice
//! blindly. Two costs are demonstrated here: relaxing an argument after the fact
//! can break the caller's type inference (`E0283`, the book's `foo(&iter.
//! collect())` example), and moving the choice to dynamic dispatch takes a
//! decision away from the caller.
//!
//! > For arguments that you take by reference anyway … you can easily replace
//! > your generic argument with one that uses dynamic dispatch. For instance,
//! > instead of `impl AsRef<str>` , you could take `&dyn AsRef<str>` .
//!
//! > with generics, the caller can always choose dynamic dispatch themselves by
//! > passing in a trait object. The reverse is not true: if you take a trait
//! > object, that is what the caller must provide.

// ---------------------------------------------------------------------------
// Three widths of requirement for the same job
// ---------------------------------------------------------------------------

/// The narrowest contract: it names `Vec`, so the caller must own one.
///
/// A slice is not a `Vec`, and the compiler will not invent one:
///
/// ```compile_fail,E0308
/// use rust_for_rustaceans::designing_interfaces::flexible::generic_arguments::sum_concrete;
///
/// let total = sum_concrete(&[1, 2, 3]);   // expected `&Vec<usize>`, found `&[{integer}; 3]`
/// ```
#[allow(clippy::ptr_arg)] // the narrow signature *is* the demo; clippy's fix deletes it
pub fn sum_concrete(values: &Vec<usize>) -> usize {
    values.iter().sum()
}

/// One step wider: the function never needed a `Vec`, only a slice of integers.
/// Deref coercion makes every `&Vec<usize>` a `&[usize]` at the call site.
pub fn sum_slice(values: &[usize]) -> usize {
    values.iter().sum()
}

/// The widest useful contract: anything that can produce a `&[usize]` — owned,
/// borrowed, array, or a type this crate has never seen.
pub fn sum_as_ref(values: impl AsRef<[usize]>) -> usize {
    values.as_ref().iter().sum()
}

/// The same requirement through a trait object, which is what the book suggests
/// when the bound is simple and the call is not hot. The caller loses the
/// option of monomorphizing.
pub fn sum_dyn(values: &dyn AsRef<[usize]>) -> usize {
    values.as_ref().iter().sum()
}

/// A caller-defined type that is not a `Vec`, not a slice, and not an array.
pub struct Weights(Vec<usize>);

impl AsRef<[usize]> for Weights {
    fn as_ref(&self) -> &[usize] {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// What the caller gains by the relaxation
// ---------------------------------------------------------------------------

/// > In most cases it pays off to use generics rather than concrete types, to
/// > allow the caller to pass any type that conforms to what your function
/// > actually needs, rather than only a particular type.
///
/// The three functions agree on the answer for every input all three accept.
/// What changes is the set of inputs: `sum_concrete` takes one shape, `sum_slice`
/// takes every contiguous run of integers, and `sum_as_ref` also takes owned
/// values and other crates' types.
pub fn the_wider_the_requirement_the_wider_the_call_site() {
    let owned = vec![1, 2, 3];
    let array = [1, 2, 3];

    // The concrete signature accepts what it names, and nothing else.
    assert_eq!(sum_concrete(&owned), 6);

    // A `&Vec<usize>` deref-coerces to a slice, and so does a `&[usize; 3]`.
    assert_eq!(sum_slice(&owned), 6);
    assert_eq!(sum_slice(&array[..]), 6);

    // The generic signature accepts all of them, and by value as well.
    assert_eq!(sum_as_ref(&owned), 6);
    assert_eq!(sum_as_ref(&array[..]), 6);
    assert_eq!(sum_as_ref(owned.clone()), 6);
    assert_eq!(sum_as_ref(array), 6);
    assert_eq!(sum_as_ref(Weights(vec![1, 2, 3])), 6);
}

// ---------------------------------------------------------------------------
// Who chooses dynamic dispatch
// ---------------------------------------------------------------------------

/// > For arguments that you take by reference anyway … you can easily replace
/// > your generic argument with one that uses dynamic dispatch.
///
/// > And finally, remember that with generics, the caller can always choose
/// > dynamic dispatch themselves by passing in a trait object. The reverse is not
/// > true: if you take a trait object, that is what the caller must provide.
///
/// Both directions are visible at the call site. Passing `&values as &dyn
/// AsRef<[usize]>` to the *generic* function makes the caller pay for the vtable
/// hop; passing `&values` to [`sum_dyn`] is the only option that function has.
/// Note also that the dynamic version works only because the bound is a single
/// trait: a trait object cannot be built for a combined bound.
///
/// ```compile_fail,E0225
/// use std::hash::Hash;
///
/// fn takes_bound(_: &(dyn Hash + Eq)) {}   // only auto traits can be used as
///                                          // additional traits in a trait object
/// ```
pub fn the_caller_not_the_callee_chooses_dynamic_dispatch() {
    let values = vec![4, 5, 6];

    // The generic function, monomorphized for a concrete type...
    assert_eq!(sum_as_ref(&values), 15);

    // ...or asked for at run time, at the caller's discretion.
    let erased: &dyn AsRef<[usize]> = &values;
    assert_eq!(sum_as_ref(erased), 15);

    // The trait-object function accepts a value directly — unsize coercion
    // happens at the call site — but never monomorphizes.
    assert_eq!(sum_dyn(&values), 15);
    assert_eq!(sum_dyn(&Weights(values)), 15);
}

// ---------------------------------------------------------------------------
// The cost of relaxing an argument after the fact
// ---------------------------------------------------------------------------

/// > It may be tempting to start your interfaces off with concrete types and
/// > then turn them generic over time. This can work, but keep in mind that such
/// > changes are not necessarily backward compatible.
///
/// > Consider what happens if the caller invokes `foo` with `foo(&iter.
/// > collect())` . In the original version, the compiler could determine that it
/// > should collect into a `Vec` , but now it just knows that it needs to collect
/// > into some type that implements `AsRef<[usize]>` . And there could be
/// > multiple such types, so with this change, the caller's code will no longer
/// > compile!
///
/// The caller's code is unchanged; only the signature moved. Collecting into a
/// `Vec` was the old parameter type's job, and the new parameter type does not
/// do that job:
///
/// ```compile_fail,E0283
/// use rust_for_rustaceans::designing_interfaces::flexible::generic_arguments::sum_as_ref;
///
/// let n = 4;
/// // cannot infer type of the type parameter `B` declared on `collect`
/// let total = sum_as_ref(&(0..n).collect());
/// ```
///
/// The fix is on the caller's side — say which type to collect into:
///
/// ```
/// use rust_for_rustaceans::designing_interfaces::flexible::generic_arguments::{sum_as_ref, sum_concrete};
///
/// let n = 4;
/// assert_eq!(sum_concrete(&(0..n).collect()), 6); // inference had only one option
/// assert_eq!(sum_as_ref(&(0..n).collect::<Vec<_>>()), 6); // now it has to be told
/// ```
pub fn genericizing_late_breaks_the_callers_inference() {
    let n = 4;

    // The concrete signature is what made the bare `collect()` unambiguous.
    assert_eq!(sum_concrete(&(0..n).collect()), 6);

    // Against the generic signature the caller must spell the element type out;
    // `Vec<_>` restores the old behaviour, at the caller's expense.
    assert_eq!(sum_as_ref((0..n).collect::<Vec<_>>()), 6);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_wider_the_requirement_the_wider_the_call_site_runs() {
        the_wider_the_requirement_the_wider_the_call_site();
    }

    #[test]
    fn the_caller_not_the_callee_chooses_dynamic_dispatch_runs() {
        the_caller_not_the_callee_chooses_dynamic_dispatch();
    }

    #[test]
    fn genericizing_late_breaks_the_callers_inference_runs() {
        genericizing_late_breaks_the_callers_inference();
    }
}
