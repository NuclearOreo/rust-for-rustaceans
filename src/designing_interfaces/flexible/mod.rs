//! # Flexible
//!
//! > Every piece of code you write includes, implicitly or explicitly, a
//! > contract. The contract consists of a set of requirements and a set of
//! > promises. The requirements are restrictions on how the code can be used,
//! > while the promises are guarantees about how the code can be used.
//!
//! > A good rule of thumb is to avoid imposing unnecessary restrictions and to
//! > only make promises you can keep. Adding restrictions or removing promises
//! > usually requires a major semantic version change and is likely to break
//! > code elsewhere. Relaxing restrictions or giving additional promises, on the
//! > other hand, is usually backward compatible.
//!
//! > In Rust, restrictions usually come in the form of trait bounds and
//! > argument types, and promises come in the form of trait implementations and
//! > return types.
//!
//! The section's five subsections become the files here:
//!
//! | Module | Book subsection | What it pins down |
//! | --- | --- | --- |
//! | [`contracts`] | intro, Listing 3-1 | one job, three contracts: owned in/out, `&str` → `Cow`, `impl AsRef<str>` → `impl AsRef<str>` |
//! | [`generic_arguments`] | *Generic Arguments* | relaxing a concrete argument, what that costs type inference, and who gets to pick dynamic dispatch |
//! | [`object_safety`] | *Object Safety*, *Drop Trait Objects* | object safety as an unwritten part of the contract, and the `drop` entry in every vtable |
//! | [`borrowed_vs_owned`] | *Borrowed vs. Owned* | `Cow`, caller-controlled allocation, and `Copy` types that are not cheap |
//! | [`fallible_destructors`] | *Fallible and Blocking Destructors* | an explicit destructor beside a best-effort `Drop`, with the book's three workarounds |
//!
//! # Why a contract can be tested
//!
//! A contract is a statement about which programs keep compiling. So the
//! evidence splits cleanly along the section's own axis: a *relaxed* requirement
//! or an *added* promise is an addition, and it shows up as code that compiles
//! (a runnable demo with a call site the narrower signature would have
//! rejected); a *tightened* requirement or a *removed* promise is a subtraction,
//! and it shows up as a `compile_fail` doctest naming the error the caller
//! would hit. `E0283` (an argument relaxed too late broke the caller's
//! inference) and `E0599` (the return value is only `impl AsRef<str>`) are both
//! that second kind, and both are errors a major version bump is supposed to
//! prevent.

pub mod borrowed_vs_owned;
pub mod contracts;
pub mod fallible_destructors;
pub mod generic_arguments;
pub mod object_safety;
