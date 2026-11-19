//! # Unsurprising
//!
//! > The Principle of Least Surprise, otherwise known as the Law of Least
//! > Astonishment, comes up a lot in software engineering, and it holds true
//! > for Rust interfaces as well. Where possible, your interfaces should be
//! > intuitive enough that if the user has to guess, they usually guess
//! > correctly.
//!
//! > The core idea here is to stick close to things the user is likely to
//! > already know so that they don't have to relearn concepts in a different
//! > way than they're used to.
//!
//! > Here, we'll look at how you can use naming, common traits, and ergonomic
//! > trait tricks to help the user out.
//!
//! Those three, plus the wrapper-type tricks that follow, become the files here:
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`naming`] | *Naming Practices* | `iter` borrows, `into_inner` consumes, `…Error` survives `?` |
//! | [`common_traits`] | *Common Traits for Types* | `Debug`, `Send`/`Sync`/`Unpin`, `Clone`/`Default`, the comparison traits — and why `Copy` is left out |
//! | [`ergonomic_trait_implementations`] | *Ergonomic Trait Implementations* | blanket impls for `&T`, `&mut T`, `Box<T>`, and `IntoIterator` for borrowed types |
//! | [`wrapper_types`] | *Wrapper Types* | `Deref`, `AsRef`, `From`/`Into`, and what `Borrow` is actually for |
//! | [`deref_and_inherent_methods`] | *Deref and Inherent Methods* | why a transparent wrapper should not have inherent methods |
//!
//! # The testable half of a style guide
//!
//! Naming and "expected traits" are conventions, not rules the compiler can
//! enforce, so each demo works from the consequence a user would notice. The
//! payoff of implementing `Debug` is a `{:?}` that prints; the payoff of
//! implementing `Error` is that `?` converts the type into `Box<dyn Error>`;
//! the cost of forgetting a blanket impl is a `E0277` on `foo(&bar)`. That last
//! kind — code that *should* compile but does not — is the exact surprise the
//! section is about, and it is verified as a `compile_fail` doctest.

pub mod common_traits;
pub mod deref_and_inherent_methods;
pub mod ergonomic_trait_implementations;
pub mod naming;
pub mod wrapper_types;
