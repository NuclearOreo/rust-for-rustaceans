//! # Obvious
//!
//! > While some users may be familiar with aspects of the implementation that
//! > underpins your interface, they are unlikely to understand all of its rules
//! > and limitations. They won't know that it's never okay to call `foo` after
//! > calling `bar` … Only if the interface makes it clear that something strange
//! > is going on will they reach for the documentation or carefully read type
//! > signatures.
//!
//! > It's therefore critical for you to make it as easy as possible for users to
//! > understand your interface and as hard as possible for them to use it
//! > incorrectly. The two primary techniques at your disposal for this are your
//! > documentation and the type system, so let's look at each of those in turn.
//!
//! > **NOTE** _You can also take advantage of naming to suggest to the user when
//! > there's more to an interface than meets the eye. If a user sees a method
//! > named `dangerous` , chances are they will read its documentation._
//!
//! The section's two subsections, with the type-system half split by technique,
//! become the files here:
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`documentation`] | *Documentation* | `# Errors` / `# Panics` as contracts, module-level end-to-end examples, grouped items with intra-doc links, `#[doc(alias)]`, `#[doc(hidden)]`, and the nightly-only `#[doc(cfg)]` |
//! | [`semantic_typing`] | *Type System Guidance* (semantic typing) | `bool` arguments → two-variant enums, newtypes for units, capability tokens with private constructors, and a `bool` + pointer pair collapsed into one enum |
//! | [`typestates`] | *Type System Guidance* (Listing 3-2) | `PhantomData` stage markers: illegal states unrepresentable, contrasted with the enum whose mistakes are runtime panics |
//! | [`must_use`] | *Type System Guidance* (`#[must_use]`) | what the annotation buys, and that it is a lint rather than a rule of the type system |
//!
//! # How "obvious" becomes testable
//!
//! The two techniques fail differently, so the evidence differs:
//!
//! - A documentation claim is a *behaviour* claim. "# Panics if `input` is
//!   empty" is verifiable — the function panics, with that message — and so is
//!   "# Errors returns `ReaderError::TooMany`". The `#[doc]` half is checked by
//!   the verification loop itself: broken intra-doc links are `cargo doc`
//!   warnings, and the `api` module turns clippy's `missing_errors_doc` and
//!   `missing_panics_doc` into part of `cargo clippy --all-targets`.
//! - A type-system claim is a *compile-time* claim, so it is a `compile_fail`
//!   doctest: calling `accelerate` while grounded is `E0599`, passing the wrong
//!   newtype is `E0308`, and forging a capability token outside the crate is
//!   `E0603`.
//! - `#[must_use]` is the awkward case: it *warns* instead of refusing, and a
//!   warning has no error code. The demo therefore shows the positive half — the
//!   code compiles and runs — and quotes the diagnostic in a `text` block.

pub mod documentation;
pub mod must_use;
pub mod semantic_typing;
pub mod typestates;
