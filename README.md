# Rust for Rustaceans — Study Sandbox

A hands-on workspace for working through **[*Rust for Rustaceans*](https://rust-for-rustaceans.com) by Jon Gjengset**.

The book is a sequel to *The Rust Programming Language*: it assumes you can already write Rust and focuses on the parts that separate working code from well-designed code — memory models, the trait system, API design, error handling, macros, async, unsafe, FFI, `no_std`, and the wider ecosystem.

This repo is not a reimplementation of the book. It's a scratchpad where each chapter gets a module, and each concept gets a small, tested code sample. Keep the book open, type the examples yourself, break them, fix them.

## Repository Layout

```text
.
├── Cargo.toml              # library crate, edition 2024, no dependencies
├── src/
│   ├── lib.rs              # module tree root
│   └── foundation/         # Ch. 1 — Foundations
│       ├── mod.rs
│       ├── memory/         # "Talking About Memory"
│       │   ├── mod.rs
│       │   ├── terminology.rs   # values, places, variables, pointers
│       │   ├── variables.rs     # high-level (flows) vs low-level (slots) models
│       │   └── regions.rs       # stack, heap, static memory
│       ├── ownership/      # "Ownership"
│       │   ├── mod.rs
│       │   ├── move_and_copy.rs # Listing 1-3, Copy vs move
│       │   └── dropping.rs      # recursive drop, drop order
│       └── borrowing/      # "Borrowing and Lifetimes"
│           ├── mod.rs
│           ├── shared.rs        # &T: shared, Copy, immutable
│           ├── mutable.rs       # &mut T: exclusive, must leave a value
│           ├── interior_mutability.rs   # mutating through a &
│           ├── lifetimes.rs     # the borrow checker; lifetimes as regions
│           ├── generic_lifetimes.rs     # references inside your own types
│           └── variance.rs      # covariant / invariant / contravariant
├── book/
│   ├── Rust_for_Rustaceans.md    # local copy of the book text (for reference/grepping)
│   └── Rust_for_Rustaceans.pdf
└── target/                 # build artifacts (gitignored)
```

**Directories are named for the chapter, not its number.** `foundation/`, not `chapter_1/`; `types/`, not `chapter_2/`. The chapter number lives in the progress table below, where it is actually useful for ordering, and nowhere in the module path — so `use` statements and test filters read as the topic they are about. Chapter titles that are Rust keywords are adjusted: *Unsafe Code* becomes `unsafe_code/`, since `unsafe` cannot be an identifier.

Sections of the book become subdirectories, one file per subsection, with the module docs quoting the book and every claim backed by a test. A demo function is a small function plus a `#[cfg(test)]` module that calls it. Illegal programs (the ones the compiler rejects) are written as `compile_fail` doctests, so `cargo test` verifies that `rustc` still rejects them with the expected error code — `E0382` for use after move, `E0184` for `Copy` plus `Drop`, and so on.

## Prerequisites

Install a Rust toolchain via [rustup](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The crate uses **edition 2024**, so it needs a reasonably recent stable toolchain (verified against `rustc 1.99.1`). Check with:

```sh
rustc --version
```

## Commands

```sh
cargo check            # fast type-check of the whole crate
cargo test             # run every exercise's unit tests
cargo test listing_1_3            # run a single test by name (substring match)
cargo test ownership::            # run everything under one section
cargo test foundation::           # run everything under one chapter
cargo run              # no-op: this is a library crate, there is no main()
cargo clippy --all-targets        # lints (if clippy is installed)
cargo doc --open       # browse generated docs for the crate
```

## Adding a Chapter

Name the directory for the chapter title in `snake_case` — `types`, not `chapter_2` — and name each file inside it for the section it covers.

1. Create the module directory and file:

   ```sh
   mkdir -p src/types
   cp src/foundation/ownership/move_and_copy.rs src/types/layout.rs
   ```

2. Wire it up:

   ```rust
   // src/types/mod.rs
   pub mod layout;
   ```

   ```rust
   // src/lib.rs
   pub mod foundation;
   pub mod types;
   ```

3. Rename the copied function and its test away from the ones you copied, then run `cargo test`.

Conventions used here:

- One directory per chapter, named after the chapter title; one file per section of that chapter.
- One `#[cfg(test)] mod test` per file — tests live beside the code they exercise.
- Chapter and section modules are `pub`, and so are the demo functions. This is deliberate, not exposure for its own sake: `rustdoc` only collects `compile_fail` doctests from public items, so the rejected programs are verified only if the path to them is public.
- `#[allow(dead_code)]` is fine while experimenting; remove it once the file's contents are used.
- Prefer `cargo test` over `cargo run`: every claim in the book should be something you can assert on.

## Chapter Progress

| # | Chapter | Directory | Status |
| --- | --- | --- | --- |
| 1 | Foundations | `src/foundation/` | ✅ *Talking About Memory* · *Ownership* · *Borrowing and Lifetimes* |
| 2 | Types | `src/types/` | not started |
| 3 | Designing Interfaces | `src/designing_interfaces/` | not started |
| 4 | Error Handling | `src/error_handling/` | not started |
| 5 | Project Structure | `src/project_structure/` | not started |
| 6 | Testing | `src/testing/` | not started |
| 7 | Macros | `src/macros/` | not started |
| 8 | Asynchronous Programming | `src/asynchronous_programming/` | not started |
| 9 | Unsafe Code | `src/unsafe_code/` | not started |
| 10 | Concurrency (and Parallelism) | `src/concurrency/` | not started |
| 11 | Foreign Function Interfaces | `src/foreign_function_interfaces/` | not started |
| 12 | Rust Without the Standard Library | `src/rust_without_the_standard_library/` | not started |
| 13 | The Rust Ecosystem | `src/rust_ecosystem/` | not started |

Directories are listed as they will be named when created; only `foundation/` exists so far. Update the status column as you go: ⏳ in progress, ✅ done, ⏭️ skipped.

## Notes

- `book/` is committed so the reference text is available offline and greppable, e.g. `grep -n "Lifetime Variance" book/Rust_for_Rustaceans.md`. It's the author's copyrighted work — for personal study only, no redistribution.
- Chapters are largely independent. *Foundations* and *Types* underpin everything else, so resist skipping them even if they feel familiar.
- *Asynchronous Programming*, *Unsafe Code*, and *Rust Without the Standard Library* will eventually want extra dependencies (`tokio`, `miri`, `cross`). Add them to `Cargo.toml` when you get there rather than now.
