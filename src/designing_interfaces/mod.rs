//! Chapter 3 — *Designing Interfaces*.
//!
//! > Every project, no matter how large or small, has an API. In fact, it
//! > usually has several. … As your codebase grows in size and complexity,
//! > you'll find it worthwhile to invest some thought and care into how you
//! > design even the internal APIs to make the experience of using and
//! > maintaining the code over time as pleasant as possible.
//!
//! > These essentially boil down to four principles: your interfaces should be
//! > _unsurprising_ , _flexible_ , _obvious_ , and _constrained_ .
//!
//! The chapter's first two principles are covered so far:
//!
//! | Section | Book subsections | What it pins down |
//! | --- | --- | --- |
//! | [`unsurprising`] | *Naming Practices*, *Common Traits for Types*, *Ergonomic Trait Implementations*, *Wrapper Types*, *Deref and Inherent Methods* | names that mean what they say, the traits users assume exist, and the impls that keep `&T` and `Box<T>` from surprising them |
//! | [`flexible`] | *Generic Arguments*, *Object Safety*, *Borrowed vs. Owned*, *Fallible and Blocking Destructors* | the requirements and promises an interface binds itself to, and which changes to them are additive |
//!
//! The remaining sections — *Obvious* and *Constrained* — are not started yet.
//!
//! # What "unsurprising" means in code
//!
//! *Unsurprising* is unusual in that most of its advice is about *conventions*
//! rather than about things the compiler can be made to reject. The demos
//! therefore take the convention as a hypothesis and check its observable
//! consequence: `iter` works through `&self`, `into_inner` consumes its
//! receiver, a `…Error` type survives `?` into `Box<dyn Error>`, a wrapper's
//! inner methods are reachable through `.`, and so on. Where the *absence* of a
//! convention is what matters, the demonstration is a `compile_fail` doctest
//! naming the error code the user would hit.
//!
//! # What "flexible" means in code
//!
//! *Flexible* is about the contract itself, and a contract is a statement about
//! which programs keep compiling. A relaxed requirement or a new promise is an
//! addition, so it is demonstrated by a call site the narrower signature would
//! have rejected; a tightened requirement or a withdrawn promise is a
//! subtraction, so it is a `compile_fail` doctest with the caller's error code
//! (`E0283`, `E0599`, `E0509`, …).

pub mod flexible;
pub mod unsurprising;
