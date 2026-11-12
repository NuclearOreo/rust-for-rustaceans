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
│   └── chapter_1/
│       ├── mod.rs          # `mod boiler;`
│       └── boiler.rs       # scratch file: one fn + one unit test
├── book/
│   ├── Rust_for_Rustaceans.md    # local copy of the book text (for reference/grepping)
│   └── Rust_for_Rustaceans.pdf
└── target/                 # build artifacts (gitignored)
```

`src/chapter_1/boiler.rs` is the template shape every new exercise follows: a small function plus a `#[cfg(test)]` module asserting on it. Use it as the starting point for new files.

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
cargo test running_print          # run a single test by name
cargo test chapter_1::            # run everything under one chapter module
cargo run              # no-op: this is a library crate, there is no main()
cargo clippy --all-targets        # lints (if clippy is installed)
cargo doc --open       # browse generated docs for the crate
```

## Adding a Chapter

1. Create the module directory and file:

   ```sh
   mkdir -p src/chapter_2
   cp src/chapter_1/boiler.rs src/chapter_2/basics.rs
   ```

2. Wire it up:

   ```rust
   // src/chapter_2/mod.rs
   mod basics;
   ```

   ```rust
   // src/lib.rs
   mod chapter_1;
   mod chapter_2;
   ```

3. Rename the copied function and test in `basics.rs` away from `addition` / `running_print`, then `cargo test`.

Conventions used here:

- One file per subtopic, one `#[cfg(test)] mod test` per file — tests live beside the code they exercise.
- Modules are private by default. Add `pub` only when you actually want to try the item from another module or an integration test in `tests/`.
- `#[allow(dead_code)]` is fine while experimenting; remove it once the file's contents are used.
- Prefer `cargo test` over `cargo run`: every claim in the book should be something you can assert on.

## Chapter Progress

| # | Chapter | Module | Status |
| --- | --- | --- | --- |
| 1 | Foundations | `src/chapter_1/` | ⏳ scaffold only (`boiler.rs`) |
| 2 | Types | — | not started |
| 3 | Designing Interfaces | — | not started |
| 4 | Error Handling | — | not started |
| 5 | Project Structure | — | not started |
| 6 | Testing | — | not started |
| 7 | Macros | — | not started |
| 8 | Asynchronous Programming | — | not started |
| 9 | Unsafe Code | — | not started |
| 10 | Concurrency (and Parallelism) | — | not started |
| 11 | Foreign Function Interfaces | — | not started |
| 12 | Rust Without the Standard Library | — | not started |
| 13 | The Rust Ecosystem | — | not started |

Update the status column as you go: ⏳ in progress, ✅ done, ⏭️ skipped.

## Notes

- `book/` is committed so the reference text is available offline and greppable, e.g. `grep -n "Lifetime Variance" book/Rust_for_Rustaceans.md`. It's the author's copyrighted work — for personal study only, no redistribution.
- Chapters are largely independent. Chapters 1 and 2 are the foundation for everything else, so resist skipping them even if they feel familiar.
- Async (8), unsafe (9), and `no_std` (12) will eventually want extra dependencies (`tokio`, `miri`, `cross`). Add them to `Cargo.toml` when you get there rather than now.
