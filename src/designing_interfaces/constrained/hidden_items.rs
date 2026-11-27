//! # Constrained — Hiding Items from Documentation
//!
//! > The `#[doc(hidden)]` attribute lets you hide a public item from your
//! > documentation without making it inaccessible to code that happens to know it
//! > is there. This is often used to expose methods and types that are needed by
//! > macros, but not by user code.
//!
//! > How such hidden items interact with your interface contract is a matter of
//! > some debate. In general, items marked as `#[doc(hidden)]` are only considered
//! > part of your contract insofar as their public effects; for example, if user
//! > code may end up containing a hidden type, then whether that type is `Send` or
//! > not is part of the contract, whereas its name is not.
//!
//! > Hidden inherent methods and hidden trait methods on sealed traits are not
//! > generally part of your interface contract, though you should make sure to
//! > state this clearly in the documentation for those methods.
//!
//! > And yes, hidden items should still be documented!
//!
//! Two hidden items, one for each use the paragraph names:
//!
//! | Item | Hidden from | Still part of the contract |
//! | --- | --- | --- |
//! | [`__increment`] | the docs | what the macro that calls it does |
//! | [`HiddenIter`] | the docs | its `Iterator` impl *and* its auto-traits; not its name |
//!
//! The verification is the mirror image of the rest of the section: instead of a
//! `compile_fail` for something the user cannot do, this file has runnable
//! doctests that do the thing the attribute was supposed to discourage — name the
//! hidden items — and pass anyway.

// ---------------------------------------------------------------------------
// A helper a macro expands to
// ---------------------------------------------------------------------------

/// The helper `increment!` expands to.
///
/// > This is often used to expose methods and types that are needed by macros,
/// > but not by user code.
///
/// A user sees `increment!(41)` and never the path below; an error message is
/// where the name can still surface, which is why the item is documented.
#[doc(hidden)]
pub fn __increment(value: u64) -> u64 {
    value + 1
}

/// A macro that expands to the hidden helper.
#[macro_export]
macro_rules! increment {
    ($value:expr) => {
        $crate::designing_interfaces::constrained::hidden_items::__increment($value)
    };
}

// ---------------------------------------------------------------------------
// A type a macro can hand back
// ---------------------------------------------------------------------------

/// A hidden type that user code can end up holding.
///
/// > if user code may end up containing a hidden type, then whether that type is
/// > `Send` or not is part of the contract, whereas its name is not
///
/// The name being out of the contract does not make it unreachable — it is public:
///
/// ```
/// use rust_for_rustaceans::designing_interfaces::constrained::auto_traits::is_normal;
/// use rust_for_rustaceans::designing_interfaces::constrained::hidden_items::{
///     hidden_iter, HiddenIter,
/// };
///
/// // The name works, and so does the type.
/// let iter: HiddenIter = hidden_iter();
/// assert_eq!(iter.count(), 2);
///
/// // The auto-trait promise is part of the contract even though the name is not.
/// is_normal::<HiddenIter>();
/// ```
///
/// Nor does it make the *methods* part of the contract: [`HiddenIter::remaining`]
/// is hidden too, documented so that whoever meets it knows what it does, and
/// still not something to rely on.
#[doc(hidden)]
pub struct HiddenIter {
    remaining: u8,
}

impl HiddenIter {
    /// How many items are left. Hidden, documented, and not part of the contract.
    #[doc(hidden)]
    pub fn remaining(&self) -> u8 {
        self.remaining
    }
}

impl Iterator for HiddenIter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        Some(self.remaining)
    }
}

/// Returns a [`HiddenIter`] by value, so a caller can hold one without the
/// function's signature promising anything about its name.
pub fn hidden_iter() -> HiddenIter {
    HiddenIter { remaining: 2 }
}

// ---------------------------------------------------------------------------
// What "hidden" changes, and what it does not
// ---------------------------------------------------------------------------

/// The macro is the user-facing half, and it works without anyone knowing the
/// helper exists — which is the reason to hide it in the first place.
pub fn a_macro_can_use_a_hidden_helper() {
    assert_eq!(crate::increment!(41), 42);
    assert_eq!(crate::increment!(crate::increment!(40)), 42);
}

/// The type is the contract half: nameable, usable, `Send`-checked, and with a
/// hidden inherent method that is deliberately outside what a version bump has to
/// preserve.
pub fn a_hidden_type_is_still_nameable_and_still_counts_by_its_effects() {
    let mut iter = hidden_iter();
    assert_eq!(iter.remaining(), 2);

    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.remaining(), 1);
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), None);

    crate::designing_interfaces::constrained::auto_traits::is_normal::<HiddenIter>();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_macro_can_use_a_hidden_helper_runs() {
        a_macro_can_use_a_hidden_helper();
    }

    #[test]
    fn a_hidden_type_is_still_nameable_and_still_counts_by_its_effects_runs() {
        a_hidden_type_is_still_nameable_and_still_counts_by_its_effects();
    }
}
