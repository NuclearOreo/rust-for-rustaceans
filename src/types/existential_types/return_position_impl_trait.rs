//! # `impl Trait` in Return Position
//!
//! > All functions marked as `async fn` or with a return type of `impl Trait`
//! > have an existential return type: the signature does not give the true type
//! > of the return value, just a hint that the function returns _some_ type that
//! > implements some set of traits that the caller can rely on. And crucially,
//! > the caller can only rely on the return type implementing those traits, and
//! > nothing else.
//!
//! > This behavior is what gives existential types their name: we are asserting
//! > that there exists some concrete type that matches the signature, and we
//! > leave it up to the compiler to find what that type is.
//!
//! Three consequences are checked here: the caller really is limited to the
//! named traits, `async fn` returns such a type, and the erasure costs nothing —
//! no `Box`, no vtable, no change to callers when the implementation changes.

use std::any::type_name_of_val;
use std::future::Future;
use std::mem::size_of;
use std::mem::size_of_val;
use std::pin::pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

// ---------------------------------------------------------------------------
// The signature promises only its bounds
// ---------------------------------------------------------------------------

/// The returned type is a `Map<Range<u8>, {closure}>`, but the signature says
/// only "some `Iterator` of `u8`". Callers get to rely on `Iterator`, and on
/// nothing else.
pub fn evens_up_to(limit: u8) -> impl Iterator<Item = u8> {
    (0..limit).map(|value| value * 2)
}

/// > the caller can only rely on the return type implementing those traits, and
/// > nothing else.
///
/// The concrete type *does* have an `as_slice` method — it is a
/// `vec::IntoIter` — but the caller cannot see that, so it cannot call it:
///
/// ```compile_fail,E0599
/// fn numbers() -> impl Iterator<Item = u8> {
///     vec![1u8, 2].into_iter()
/// }
///
/// let numbers = numbers();
/// let slice = numbers.as_slice();   // no method named `as_slice` found for
///                                   // opaque type `impl Iterator<Item = u8>`
/// ```
pub fn the_caller_sees_only_the_declared_traits() {
    let evens: Vec<u8> = evens_up_to(4).collect();
    assert_eq!(evens, vec![0, 2, 4, 6]);

    // Everything `Iterator` offers is available…
    let count = evens_up_to(4).filter(|value| *value > 0).count();
    assert_eq!(count, 3);

    // …and the concrete type is still observable at run time, for diagnostics.
    let name = type_name_of_val(&evens_up_to(4));
    assert!(name.contains("Map"), "unexpected name: {name}");
}

// ---------------------------------------------------------------------------
// async fn
// ---------------------------------------------------------------------------

/// An `async fn`: the signature says `u8`, but the value returned is an
/// anonymous future whose type nobody can write down.
pub async fn seven() -> u8 {
    7
}

/// Enough of an executor to run a future that is ready on its first poll — which
/// is all this section needs. `Waker::noop()` means "nothing to wake", and
/// `pin!` keeps the future in the caller's frame. Chapter 8 builds the real
/// thing.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());

    for _ in 0..1_000 {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => continue,
        }
    }

    panic!("future did not complete in 1000 polls");
}

/// > if you return … an async block from a trait method, its type does not have
/// > a name that you can type into your code.
///
/// ```compile_fail,E0121
/// async fn seven() -> u8 { 7 }
///
/// fn named() -> _ {
///     seven()     // the placeholder `_` is not allowed within types on item
/// }               // signatures for return types
/// ```
pub fn an_async_fn_returns_an_existential_future() {
    // It is a real value implementing `Future`…
    fn assert_future<F: Future<Output = u8>>(_future: F) {}

    assert_future(seven());

    // …and it can be driven to completion.
    assert_eq!(block_on(seven()), 7);

    // The same value spelled existentially, without `async fn`. Clippy would
    // rewrite this into the `async fn` above; the two spellings side by side are
    // the point.
    #[allow(clippy::manual_async_fn)]
    fn seven_again() -> impl Future<Output = u8> {
        async { 7 }
    }

    assert_eq!(block_on(seven_again()), 7);
}

/// > Technically, it isn't strictly true that the caller relies on the return
/// > type and nothing else. The compiler will also propagate auto-traits like
/// > `Send` and `Sync` through `impl Trait` in return position.
///
/// And the propagation goes the other way too: hold an `Rc` across an `.await`
/// and the future stops being `Send`, *without* the signature changing. It cannot
/// be a `compile_fail` doctest, because that diagnostic carries no error code:
///
/// ```text
/// fn borrowed() -> impl Future<Output = usize> {
///     async {
///         let shared = Rc::new(1usize);
///         std::future::ready(()).await;   // `shared` is held across this await
///         Rc::strong_count(&shared)
///     }
/// }
/// fn needs_send<F: Future + Send>(_future: F) {}
/// needs_send(borrowed());
/// ```
///
/// ```text
/// error: future cannot be sent between threads safely
///    = help: within `impl Future<Output = usize>`, the trait `Send` is not
///            implemented for `Rc<usize>`
/// note: future is not `Send` as this value is used across an await
/// ```
pub fn auto_traits_are_propagated_through_impl_trait() {
    fn needs_send<F: Future + Send>(_future: F) {}

    // Auto-traits are not nameable in a bound the way `Iterator` is, so they
    // travel with the concrete type. A future whose body owns only `Send` values
    // therefore satisfies `Send` — the bound below is what proves it.
    fn owned_future() -> impl Future<Output = u8> + Send {
        let value = 7u8; // moved into the future; `u8` is `Send`
        async move { value }
    }

    needs_send(owned_future());
    assert_eq!(block_on(owned_future()), 7);
}

// ---------------------------------------------------------------------------
// Zero-cost type erasure
// ---------------------------------------------------------------------------

/// The same values produced by a differently shaped iterator. Both functions have
/// the same signature; a caller cannot tell them apart, which is what lets an
/// implementation change without breaking downstream code.
pub fn evens_alternate(limit: u8) -> impl Iterator<Item = u8> {
    (0..limit).flat_map(|value| std::iter::once(value * 2))
}

/// > Not only does this simplify the interface, but it also enables you to change
/// > that implementation as you wish without breaking downstream code in the
/// > future.
///
/// The erasure is not the same thing as boxing. `impl Iterator` returns the
/// adapter chain *by value*: here the closure is zero-sized, so the returned
/// iterator is exactly as large as the `Range` it wraps. `Box<dyn Iterator>` is a
/// wide pointer plus an allocation.
pub fn type_erasure_is_zero_cost() {
    // > they allow you to perform zero-cost type erasure
    let inline = evens_up_to(4);
    assert_eq!(size_of_val(&inline), size_of::<std::ops::Range<u8>>());

    let boxed: Box<dyn Iterator<Item = u8>> = Box::new(evens_up_to(4));
    assert_eq!(size_of_val(&boxed), 2 * size_of::<usize>());

    // Same values either way; only the cost of hiding differs.
    assert_eq!(inline.collect::<Vec<u8>>(), boxed.collect::<Vec<u8>>());

    // Two implementations, one signature, identical results for the caller.
    let original: Vec<u8> = evens_up_to(4).collect();
    let swapped: Vec<u8> = evens_alternate(4).collect();
    assert_eq!(original, swapped);
    assert_eq!(original, vec![0, 2, 4, 6]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_caller_sees_only_the_declared_traits_runs() {
        the_caller_sees_only_the_declared_traits();
    }

    #[test]
    fn an_async_fn_returns_an_existential_future_runs() {
        an_async_fn_returns_an_existential_future();
    }

    #[test]
    fn auto_traits_are_propagated_through_impl_trait_runs() {
        auto_traits_are_propagated_through_impl_trait();
    }

    #[test]
    fn type_erasure_is_zero_cost_runs() {
        type_erasure_is_zero_cost();
    }
}
