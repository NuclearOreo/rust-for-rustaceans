---
name: rust-for-rustaceans-nav
description: Re-orient in this Rust for Rustaceans study repo after context loss. Covers where chapters and sections live, the conventions every demo must follow, how to verify work, and the traps that have already caused silent failures. Use when starting or resuming work in this repo, when unsure which module a book section belongs to, before adding a chapter or section, or when the user says "continue" without other context.
---

# Navigating this repo

A study sandbox for **Jon Gjengset's *Rust for Rustaceans***. Each book chapter
becomes a module; each section of a chapter becomes a file of demos backed by
tests. The goal is *verified understanding*, not a reimplementation of the book.

## Restore context (do this first)

```sh
sed -n '10,35p' README.md                              # directory layout
sed -n '/## Chapter Progress/,/^## Notes/p' README.md   # what is done, what is next
find src -name '*.rs' | sort                           # module tree
cargo test                                             # is it healthy?
```

`README.md` is the source of truth for layout and progress — it is maintained
alongside the code and is what a fresh session should trust. `src/lib.rs` and
each `mod.rs` give the actual tree.

Then read the `//!` module docs of the section you are working on: they quote the
book and state which listings are demonstrated.

## The one structural rule

**Directory = chapter title in `snake_case`. Never `chapter_N`.**

```text
src/foundation/     Ch. 1 Foundations        ✅ complete
src/types/          Ch. 2 Types              ← next
src/designing_interfaces/   Ch. 3
src/error_handling/         Ch. 4
src/project_structure/      Ch. 5
src/testing/                Ch. 6
src/macros/                 Ch. 7
src/asynchronous_programming/  Ch. 8
src/unsafe_code/            Ch. 9  (renamed: `unsafe` is a keyword)
src/concurrency/            Ch. 10
src/foreign_function_interfaces/  Ch. 11
src/rust_without_the_standard_library/  Ch. 12
src/rust_ecosystem/         Ch. 13
```

Inside a chapter, one file per `####` section of the book. A section covering
several distinct topics gets several files — `foundation/borrowing/` has six.

## Current state

- `foundation/memory/` — terminology, variables, regions
- `foundation/ownership/` — move_and_copy, dropping
- `foundation/borrowing/` — shared, mutable, interior_mutability, lifetimes,
  generic_lifetimes, variance
- `types/types_in_memory/` — alignment, layout, complex_types,
  dynamically_sized_types

Chapter 1 is complete. Chapter 2's first section, *Types in Memory*, is complete.
**Next is Chapter 2, *Traits and Trait Bounds*** → `types/traits_and_trait_bounds/`.

The shape is uniform: `src/<chapter>/<section>/<topic>.rs`. `src/types/` is the
chapter, and each `####` section of the book (`Types in Memory`, `Traits and
Trait Bounds`, `Existential Types`) is a directory under it — the same way
`foundation/` holds `memory/`, `ownership/` and `borrowing/`.

## Anatomy of a section module

```rust
//! # <Section title>
//!
//! Quoted claims from the book, then what this file pins down.

use std::...;

/// <Which listing this is, and the claim it proves.>
pub fn listing_1_7_short_name() {
    // assert the observable consequence
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_1_7_short_name_runs() { listing_1_7_short_name(); }
}
```

Every demo is a `pub fn` that asserts internally, plus exactly one `#[test]`
that calls it. File naming follows the topic, not the listing number.

## Conventions (non-negotiable)

1. **Everything on the path is `pub`** — chapter module, section module, demo
   fns. Rustdoc only collects doctests from public items, so a private path means
   the `compile_fail` programs are silently never run.
2. **Runnable demos for accepted programs; `compile_fail,EXXXX` doctests for
   rejected ones.** The book's commented-out lines are the rejected programs —
   they belong in doctests so the compiler's refusal is verified, not described.
3. **When the book's claim is two-sided, include both halves.** "Two lifetimes
   are needed" means a runnable `MutStr<'a,'b>` *and* a doctest for the
   one-lifetime collapse. Same for `Drop` making a drop count as a use.
4. **Always name the error code**: `compile_fail,E0507`, not `compile_fail`.
   Without a code, any compile error satisfies the test — including a typo.
5. **Quote the book** in `//!`/`///` with quotation marks, and cite listing
   numbers. Do not paraphrase away the author's phrasing.
6. **No global test state.** For drop-order or timing demos use a per-test log
   (`DropLog`/`Tracked` in `foundation/ownership/dropping.rs`) or an `Arc` count.
7. **Update `README.md`** — layout tree and progress table — in the same change
   that adds or completes a section.

## Verification loop

```sh
cargo test                  # two "test result: ok" lines: unit tests, then doctests
cargo clippy --all-targets  # must print zero warnings
cargo doc --no-deps         # must print zero warnings (broken intra-doc links)
```

All three clean is the definition of done. Useful filters:

```sh
cargo test borrowing::         # one section
cargo test foundation::        # one chapter
cargo test listing_1_3         # substring match on a test name
cargo test --doc 2>&1 | grep "compile fail"   # list every rejected program
```

## Traps that have already caused silent failures

1. **A doctest written as `//` inside a function body is a plain comment.**
   Rustdoc never sees it, so the test does not exist and nothing fails. Doc
   comments must sit above an item. Symptom: the doctest count did not increase.
   Always cross-check with `cargo test --doc 2>&1 | grep "compile fail"`.
2. **"Illegal" snippets that actually compile**, which makes the doctest fail:
   - `let a = &mut x; let b = &mut x;` — fine if `a` is never used; NLL ends it
     immediately. Use `*a += 1` after `*b` to force the overlap.
   - `let was = *s;` — legal when `s: Box<i32>`; only `&mut Box<i32>` is `E0507`.
   - `let x: usize; ptr::addr_of!(x)` — `E0381`; Rust will not even hand out the
     address of an uninitialized local. Use `MaybeUninit`.
3. **Deref depth.** With `let x = Box::new(42); let r = &x;`, `*r` is a `Box`,
   so comparisons need `**r`. This fails as `E0308`, not as a subtle bug.
4. **Clippy suggestions that would destroy the demo.** Fix at the source when
   possible; when the lint's fix removes the point (e.g. `replace_box` replacing
   a move with a write-through, `needless_late_init` reordering the drop demo),
   put `#[allow(clippy::the_lint)]` directly above the fn with a `//` comment
   explaining why. Existing precedents: `replace_box`, `needless_late_init`,
   `ptr_arg`, `let_and_return`.
5. **`let mut slot = T::new(); slot = other;`** triggers `unused_assignments`
   because the first value is never read — and it also proves nothing. Read the
   old value first, then show the drop (see
   `borrowing::mutable::overwriting_drops_the_old_value_immediately`, which
   observes it with `Arc::strong_count`).
6. **Layout demos: the "same" struct is not the same struct.**
   - A plain struct is `repr(Rust)` and reorders fields, so
     `struct Bar { a: u8, b: u32 }` really is 8 bytes but has `b` at offset 0.
     Want declarable padding? Say `#[repr(C)]`.
   - `offset_of!` is `E0793` on a `#[repr(packed)]` field (it takes a reference
     internally), so packed types can only be asserted on size and alignment.
   - `println!("{}", packed.field)` is *also* `E0793`, because format arguments
     are taken by reference. Copy the field into a local first.
   - Clippy's `repr_packed_without_abi` wants `#[repr(C, packed)]`; that keeps
     the zero padding and alignment of 1, so qualify it and note the difference
     instead of reaching for `#[allow]`.
7. **`std::ptr::metadata` is unstable** (`ptr_metadata`, `E0658`). To show that a
   wide pointer carries a length or a vtable, use `size_of_val`, which reads the
   metadata for you — and never `size_of_val(&handle)` on a `&dyn T`, which is a
   `&&dyn T` (`clippy::size_of_ref`) measuring the handle instead of the value.

## Error codes already in use

| Code | Used for |
| --- | --- |
| `E0184` | `Copy` + `Drop` on one type |
| `E0204` | `derive(Copy)` with a non-`Copy` field |
| `E0277` | missing trait bound (`assert_copy::<String>()`); also a `Sized` requirement (`size_of::<[u8]>()`, a non-last unsized field, `*boxed_slice`) |
| `E0308` | mismatched types (variance direction, fn-pointer coercion) |
| `E0381` | use/borrow of a possibly-uninitialized binding |
| `E0382` | use of moved value |
| `E0499` | two `&mut` at once |
| `E0502` | shared and mutable borrow overlap |
| `E0506` | assign to a place while it is borrowed |
| `E0507` | move out of a `&mut` |
| `E0515` | return a reference to a temporary / local |
| `E0594` | write through a `&` |
| `E0597` | "does not live long enough" |
| `E0605` | cast `&T` as `&mut T` |
| `E0690` | `repr(transparent)` on a struct with two non-trivial fields |
| `E0793` | reference to a field of a `repr(packed)` struct |

Verify a new code with a throwaway file before trusting it:

```sh
printf '%s' 'fn f(s: &mut Box<i32>) { let _was = *s; }' > /tmp/x.rs
rustc --edition 2024 --crate-type lib --emit=metadata -o /tmp/x.rmeta /tmp/x.rs
```

## Finding the book text

`book/Rust_for_Rustaceans.md` is the full book, committed for offline grep.

```sh
grep -n "^# \*\*"    book/Rust_for_Rustaceans.md   # chapters: "# **2 TYPES**"
grep -n "^#### \*\*" book/Rust_for_Rustaceans.md   # sections: "#### **Ownership**"
grep -n "Lifetime Variance" book/Rust_for_Rustaceans.md   # by title
```

- The table of contents near the top (roughly lines 15–170) repeats every title;
  the later hit is the body.
- The file is PDF-extracted: running heads get spliced into paragraphs ("pS As
  you dive in...") and listings are split across inline-code lines. Read for
  meaning; do not trust its formatting.
- Line numbers shift. Grep by title, never by remembered offset.

## Do not

- Add dependencies while working on chapters 1–7; there are none today. `tokio`,
  `miri`, and `cross` arrive with chapters 8, 9, and 12.
- Rename a directory to `chapter_N`, or restructure an existing section module.
- Use `cargo run` — this is a library crate with no `main`.
- Mark a section done in the progress table while `cargo test` or `cargo clippy`
  is failing, or while a `compile_fail` doctest is inert.
