//! # `Deref` and Inherent Methods
//!
//! > The magic around the dot operator and `Deref` can get confusing and
//! > surprising when there are methods on `T` that take `self` . For example,
//! > given a value `t: T` , it is not clear whether `t.frobnicate()` frobnicates
//! > the `T` or the underlying `U` !
//!
//! > For this reason, types that allow you to transparently call methods on some
//! > inner type that isn't known in advance should avoid inherent methods.
//!
//! > In these cases, favor static methods of the form `fn frobnicate(t: T)` .
//! > That way, `t.frobnicate()` always calls `U::frobnicate` , and
//! > `T::frobnicate(t)` can be used to frobnicate the `T` itself.
//!
//! The rule is easy to state and easy to break, so this file builds the same
//! wrapper twice:
//!
//! * [`BadHandle`] adds an inherent `frobnicate`. The dot operator silently
//!   prefers it, and the inner type's method becomes reachable only by writing
//!   `(*handle).frobnicate()`.
//! * [`GoodHandle`] has no inherent method, only a static `frobnicate(handle)`.
//!   `handle.frobnicate()` is then unambiguously the inner type's, and the
//!   wrapper's operation has its own spelling.
//!
//! `Vec`'s `push` is the counter-example the book gives for why this is safe
//! there: `Vec<T>` derefs to `[T]`, and slices will not grow a `push` method.
//! The danger is only when the inner type is chosen by the user.

use std::ops::Deref;

// ---------------------------------------------------------------------------
// The inner type: user-controlled, so its methods must stay reachable
// ---------------------------------------------------------------------------

/// The "underlying `U`". In a real API this would be the user's type, which is
/// why the wrapper cannot know which methods it will have.
#[derive(Debug)]
pub struct Frobnicator {
    pub label: &'static str,
}

impl Frobnicator {
    pub fn frobnicate(&self) -> &'static str {
        "Frobnicator::frobnicate"
    }
}

// ---------------------------------------------------------------------------
// The anti-pattern
// ---------------------------------------------------------------------------

/// A transparent wrapper that also defines an inherent method with the same
/// name as one of the inner type's.
#[derive(Debug)]
pub struct BadHandle(pub Frobnicator);

impl Deref for BadHandle {
    type Target = Frobnicator;

    fn deref(&self) -> &Frobnicator {
        &self.0
    }
}

impl BadHandle {
    /// The inherent method. Method resolution finds it before it ever considers
    /// the `Deref` target, so it wins the dot.
    pub fn frobnicate(&self) -> &'static str {
        "BadHandle::frobnicate"
    }
}

/// > given a value `t: T` , it is not clear whether `t.frobnicate()` frobnicates
/// > the `T` or the underlying `U` !
///
/// It is not unclear to the compiler — inherent methods win — but it is unclear
/// to the reader, and the inner method is now reachable only by asking for the
/// dereference explicitly.
pub fn an_inherent_method_shadows_the_inner_one() {
    let handle = BadHandle(Frobnicator { label: "bad" });

    assert_eq!(handle.frobnicate(), "BadHandle::frobnicate"); // the T
    assert_eq!((*handle).frobnicate(), "Frobnicator::frobnicate"); // the U
}

// ---------------------------------------------------------------------------
// The book's fix
// ---------------------------------------------------------------------------

/// The same wrapper with no inherent method: only an associated function that
/// takes the whole wrapper by value.
#[derive(Debug)]
pub struct GoodHandle(pub Frobnicator);

impl Deref for GoodHandle {
    type Target = Frobnicator;

    fn deref(&self) -> &Frobnicator {
        &self.0
    }
}

impl GoodHandle {
    /// > favor static methods of the form `fn frobnicate(t: T)` .
    ///
    /// No `self` receiver, so the dot operator never sees it.
    pub fn frobnicate(handle: GoodHandle) -> &'static str {
        let _label = handle.0.label; // the wrapper's own operation
        "GoodHandle::frobnicate"
    }
}

/// > That way, `t.frobnicate()` always calls `U::frobnicate` , and
/// > `T::frobnicate(t)` can be used to frobnicate the `T` itself.
///
/// Both calls now say what they do without a dereference to disambiguate them.
pub fn a_static_method_leaves_the_dot_operator_to_the_inner_type() {
    let handle = GoodHandle(Frobnicator { label: "good" });

    assert_eq!(handle.frobnicate(), "Frobnicator::frobnicate");
    assert_eq!(GoodHandle::frobnicate(handle), "GoodHandle::frobnicate");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_inherent_method_shadows_the_inner_one_runs() {
        an_inherent_method_shadows_the_inner_one();
    }

    #[test]
    fn a_static_method_leaves_the_dot_operator_to_the_inner_type_runs() {
        a_static_method_leaves_the_dot_operator_to_the_inner_type();
    }
}
