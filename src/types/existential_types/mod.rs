//! # Existential Types
//!
//! > In Rust you very rarely have to specify the types of variables you declare
//! > in the body of a function or the types of generic arguments to methods that
//! > you call. This is because of _type inference_, where the compiler decides
//! > what type to use based on what type the code the type appears in evaluates
//! > to.
//!
//! > To handle situations like this, Rust supports _existential types_. Chances
//! > are, you have already seen existential types in action. All functions marked
//! > as `async fn` or with a return type of `impl Trait` have an existential
//! > return type: the signature does not give the true type of the return value,
//! > just a hint that the function returns _some_ type that implements some set
//! > of traits that the caller can rely on.
//!
//! The section is one page long, but it makes four separable claims — inference
//! is limited to specific places, some types cannot be named, `impl Trait` means
//! different things in different positions, and the whole thing is a zero-cost
//! way to hide an implementation. Each gets a file:
//!
//! | Module | Book claim | What it pins down |
//! | --- | --- | --- |
//! | [`inference_and_naming`] | inference is for locals and closures; items must name types | `_` is rejected at item level; closures are distinct, unnameable types |
//! | [`return_position_impl_trait`] | `impl Trait` / `async fn` return an existential type | the caller sees only the named traits; auto-traits propagate |
//! | [`argument_position_impl_trait`] | argument-position `impl Trait` is an unnamed generic parameter | independent parameters, no turbofish |
//! | [`associated_type_position`] | `type IntoIter = impl Iterator<…>` avoids a helper type | that exact recipe is still nightly-only; the stable workarounds |
//!
//! # The one caveat to budget for
//!
//! The book's `IntoIterator` recipe — naming an associated type as
//! `impl Iterator<Item = Self::Item>` — is *not* available on stable today: it is
//! the `type_alias_impl_trait` feature, rejected as `E0658`.
//! [`associated_type_position`] demonstrates the rejection and then shows what
//! stable Rust can hide instead, so the section records both the promise and its
//! current status.

pub mod argument_position_impl_trait;
pub mod associated_type_position;
pub mod inference_and_naming;
pub mod return_position_impl_trait;
