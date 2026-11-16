//! # `impl Trait` in Associated-Type Position
//!
//! > Existential types come in handy particularly when you implement traits that
//! > have associated types. For example, imagine you're implementing the
//! > `IntoIterator` trait. It has an associated type `IntoIter` that holds the
//! > type of the iterator that the type in question can be turned into. With
//! > existential types, you do not need to define a separate iterator type to use
//! > for `IntoIter`. Instead, you can give the associated type as
//! > `impl Iterator<Item = Self::Item>` and just write an expression inside the
//! > `fn into_iter(self)` that evaluates to an `Iterator`, such as by using maps
//! > and filters over some existing iterator type.
//!
//! # Status: not on stable yet
//!
//! This is the one recommendation in the section that stable Rust does not
//! implement. `type IntoIter = impl Iterator<…>` is the `type_alias_impl_trait`
//! feature; without `#![feature(type_alias_impl_trait)]` it is rejected as
//! `E0658`. The recipe is demonstrated below exactly as the book writes it, as a
//! rejection, and then the three workarounds that stable Rust does have are
//! compared — because "hide the concrete type" is available today in every
//! position *except* an associated type.

use std::any::type_name_of_val;
use std::mem::size_of;
use std::mem::size_of_val;

/// The collection being made iterable.
pub struct Wrapper(pub Vec<u8>);

// ---------------------------------------------------------------------------
// The book's recipe, on stable
// ---------------------------------------------------------------------------

/// > you can give the associated type as `impl Iterator<Item = Self::Item>` and
/// > just write an expression inside the `fn into_iter(self)` that evaluates to
/// > an `Iterator`
///
/// ```compile_fail,E0658
/// struct Wrapper(Vec<u8>);
///
/// impl IntoIterator for Wrapper {
///     type Item = u8;
///     type IntoIter = impl Iterator<Item = u8>;   // `impl Trait` in associated
///                                                 // types is unstable
///     fn into_iter(self) -> Self::IntoIter {
///         self.0.into_iter().map(|byte| byte * 2)
///     }
/// }
/// ```
///
/// The gate is `#![feature(type_alias_impl_trait)]`, so the rest of this file is
/// about what to write instead.
pub fn the_recipe_requires_nightly_today() {
    // The named workaround, used for the rest of the file.
    let doubled: Vec<u8> = NamedIter(Wrapper(vec![1, 2, 3])).into_iter().collect();
    assert_eq!(doubled, vec![2, 4, 6]);
}

// ---------------------------------------------------------------------------
// Workaround 1: a named helper type
// ---------------------------------------------------------------------------

/// The iterator the named workaround has to expose. It exists only because
/// `IntoIter` must name a type.
pub struct Doubled(std::vec::IntoIter<u8>);

impl Iterator for Doubled {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.0.next().map(|byte| byte * 2)
    }
}

/// `IntoIterator` implemented the only way stable allows: with the helper named.
pub struct NamedIter(pub Wrapper);

impl IntoIterator for NamedIter {
    type Item = u8;
    type IntoIter = Doubled;

    fn into_iter(self) -> Doubled {
        Doubled(self.0.0.into_iter())
    }
}

/// Naming the helper costs nothing at run time — `Doubled` is a newtype over the
/// iterator it delegates to, with no indirection — but it does put a type in the
/// public API that exists purely as an implementation detail.
pub fn the_named_helper_costs_nothing_but_is_exported() {
    let doubled = NamedIter(Wrapper(vec![1, 2, 3])).into_iter();

    // A newtype over `vec::IntoIter<u8>`: same size, same machine code.
    assert_eq!(size_of_val(&doubled), size_of::<std::vec::IntoIter<u8>>());
    assert_eq!(doubled.collect::<Vec<u8>>(), vec![2, 4, 6]);
}

// ---------------------------------------------------------------------------
// Workaround 2: box the iterator
// ---------------------------------------------------------------------------

/// The erased alternative: `IntoIter` names a trait object instead of a helper
/// struct, so no helper type escapes — at the price of an allocation and a
/// virtual call per item.
pub struct BoxedIter(pub Wrapper);

impl IntoIterator for BoxedIter {
    type Item = u8;
    type IntoIter = Box<dyn Iterator<Item = u8>>;

    fn into_iter(self) -> Box<dyn Iterator<Item = u8>> {
        Box::new(self.0.0.into_iter().map(|byte| byte * 2))
    }
}

pub fn boxing_hides_the_type_at_a_runtime_cost() {
    let boxed = BoxedIter(Wrapper(vec![1, 2, 3])).into_iter();

    // A wide pointer plus a heap allocation, instead of the iterator itself.
    assert_eq!(size_of_val(&boxed), 2 * size_of::<usize>());
    assert_eq!(boxed.collect::<Vec<u8>>(), vec![2, 4, 6]);
}

// ---------------------------------------------------------------------------
// Workaround 3: `impl Trait` on an inherent method
// ---------------------------------------------------------------------------

impl Wrapper {
    /// Stable `impl Trait` in return position: the concrete
    /// `Map<slice::Iter<u8>, {closure}>` never appears in the signature, because
    /// this is an inherent method rather than a trait implementation.
    pub fn doubled(&self) -> impl Iterator<Item = u8> + '_ {
        self.0.iter().map(|byte| byte * 2)
    }
}

/// The third option is the closest to the book's intent: zero-cost, no exported
/// helper type, an opaque return type — just not available where a *trait* fixes
/// the return type. Methods you define yourself have no such constraint.
pub fn an_inherent_method_can_hide_its_type_on_stable() {
    let wrapper = Wrapper(vec![1, 2, 3]);

    assert_eq!(wrapper.doubled().collect::<Vec<u8>>(), vec![2, 4, 6]);

    // Observable at run time, invisible in the signature.
    let name = type_name_of_val(&wrapper.doubled());
    assert!(name.contains("Map"), "unexpected name: {name}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_recipe_requires_nightly_today_runs() {
        the_recipe_requires_nightly_today();
    }

    #[test]
    fn the_named_helper_costs_nothing_but_is_exported_runs() {
        the_named_helper_costs_nothing_but_is_exported();
    }

    #[test]
    fn boxing_hides_the_type_at_a_runtime_cost_runs() {
        boxing_hides_the_type_at_a_runtime_cost();
    }

    #[test]
    fn an_inherent_method_can_hide_its_type_on_stable_runs() {
        an_inherent_method_can_hide_its_type_on_stable();
    }
}
