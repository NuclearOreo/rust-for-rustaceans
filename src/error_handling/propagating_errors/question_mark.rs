//! # Propagating Errors — `?` as unwrap-or-return
//!
//! > Rust's `?` operator acts as a shorthand for _unwrap or return early_, for
//! > working easily with errors.
//!
//! > First, `?` performs type conversion through the `From` trait. In a function
//! > that returns `Result<T, E>` , you can use `?` on any `Result<T, X>` where
//! > `E: From<X>` .
//!
//! > This is the feature that makes error erasure through `Box<dyn Error>` so
//! > appealing; you can just use `?` everywhere and not worry about the
//! > particular error type, and it will usually "just work."
//!
//! Three mechanical facts, each with a runnable half and, where it has one, a
//! `compile_fail` half:
//!
//! | Fact | Runnable evidence | Rejection |
//! | --- | --- | --- |
//! | it is `return` on `Err` and `unwrap` on `Ok` | a step after the failure never runs | — |
//! | it converts with `From`, one hop per `?` | `ParseIntError` → [`ConfigError`] → `Box<dyn Error>` | `Into` without `From` ([`from_and_into`](super::from_and_into)) |
//! | it follows the *enclosing function's* type | `?` in an `Option` function | an `Option` in a `Result` function and vice versa (`E0277`) |
//!
//! > The `?` operator is already usable in fallible functions, in doctests, and
//! > in `fn main` .
//!
//! Both of the last two are runnable doctests here. A doctest's implicit `main`
//! returns `()` unless the snippet ends in `Ok::<_, E>(())`, which is how a bare
//! `?` gets a fallible wrapper:
//!
//! ```
//! let port: u16 = "8080".parse()?;
//! assert_eq!(port, 8080);
//! # Ok::<(), std::num::ParseIntError>(())
//! ```
//!
//! And a real `fn main` may return a `Result` just as well:
//!
//! ```
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let port: u16 = "8080".parse()?;
//!     assert_eq!(port, 8080);
//!     Ok(())
//! }
//! ```

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// ---------------------------------------------------------------------------
// Unwrap or return early
// ---------------------------------------------------------------------------

/// A job that logs how many steps it reached.
#[derive(Debug, Default)]
pub struct Job {
    calls: Cell<u32>,
}

impl Job {
    /// Records the call, then fails on step 3.
    ///
    /// # Errors
    ///
    /// Fails when `step == 3`.
    pub fn step(&self, step: u32) -> Result<u32, &'static str> {
        self.calls.set(self.calls.get() + 1);
        if step == 3 {
            Err("step three failed")
        } else {
            Ok(step)
        }
    }
}

/// Four steps joined by `?`: the failure returns from `run`.
///
/// # Errors
///
/// The first error any step produces.
pub fn run(job: &Job) -> Result<u32, &'static str> {
    let first = job.step(1)?;
    let second = job.step(2)?;
    let third = job.step(3)?;
    // Unreachable: step 3 returned early.
    let fourth = job.step(4)?;
    Ok(first + second + third + fourth)
}

/// Two steps that both succeed, so every `?` continues on the happy path.
///
/// # Errors
///
/// The first error any step produces.
pub fn sum_first_two(job: &Job) -> Result<u32, &'static str> {
    let first = job.step(1)?;
    let second = job.step(2)?;
    Ok(first + second)
}

/// What `?` desugars to, written out: unwrap `Ok`, `return` the `Err`.
// `question_mark`'s fix is to put the `?` back, which removes the desugaring
// this function exists to show.
#[allow(clippy::question_mark)]
pub fn run_desugared(job: &Job) -> Result<u32, &'static str> {
    let first = match job.step(1) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let second = match job.step(2) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let third = match job.step(3) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let fourth = match job.step(4) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    Ok(first + second + third + fourth)
}

// ---------------------------------------------------------------------------
// One `From` hop per `?`
// ---------------------------------------------------------------------------

/// The error the configuration layer reports.
#[derive(Debug)]
pub struct ConfigError {
    source: ParseIntError,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the configured port is not a number: {}", self.source)
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(source: ParseIntError) -> Self {
        Self { source }
    }
}

/// > you can use `?` on any `Result<T, X>` where `E: From<X>`
///
/// The generic signature is the rule itself, with the bound the operator needs.
///
/// # Errors
///
/// Whatever `value` carries, converted into `E`.
pub fn convert<T, E, X>(value: Result<T, X>) -> Result<T, E>
where
    E: From<X>,
{
    // One `?`, one conversion: `X` becomes `E` here or the value comes out as
    // `Err(E)`.
    Ok(value?)
}

/// The first hop: `ParseIntError` → [`ConfigError`].
///
/// # Errors
///
/// [`ConfigError`] when the text is not a port number.
pub fn configured(raw: &str) -> Result<u16, ConfigError> {
    let port = raw.parse()?;
    Ok(port)
}

/// The second hop: [`ConfigError`] → `Box<dyn Error>`.
///
/// # Errors
///
/// The configuration failure, erased.
pub fn start(raw: &str) -> Result<u16, Box<dyn Error>> {
    let port = configured(raw)?;
    Ok(port)
}

// ---------------------------------------------------------------------------
// Which function the `?` returns from
// ---------------------------------------------------------------------------

/// A lookup that reports "nothing there" rather than "this failed".
pub fn first_word_len(text: &str) -> Option<usize> {
    // `?` on an `Option` in a function returning `Option`.
    let word = text.split_whitespace().next()?;
    Some(word.len())
}

/// The same `?` in a function whose error type is the unit type.
///
/// # Errors
///
/// `Err(())` when there are fewer than two characters.
// The unit error is the case under study: `?` works with any `E`, including `()`.
#[allow(clippy::result_unit_err)]
pub fn at_least_two(text: &str) -> Result<(char, char), ()> {
    let mut chars = text.chars();
    let first = chars.next().ok_or(())?;
    let second = chars.next().ok_or(())?;
    Ok((first, second))
}

// ---------------------------------------------------------------------------
// Demos
// ---------------------------------------------------------------------------

/// > `?` … acts as a shorthand for _unwrap or return early_
///
/// The observable difference from a plain `unwrap` is that the *caller* gets the
/// error back, and the observable difference from `return` is that only the
/// fallible step short-circuits.
pub fn question_mark_returns_early_and_leaves_the_caller_to_decide() {
    let job = Job::default();
    assert_eq!(run(&job), Err("step three failed"));
    assert_eq!(job.calls.get(), 3, "the step after the failure never ran");

    // The desugaring behaves identically, including the skipped step.
    let job = Job::default();
    assert_eq!(run_desugared(&job), Err("step three failed"));
    assert_eq!(job.calls.get(), 3);

    // Without a failure, every `?` continues on the happy path.
    let job = Job::default();
    assert_eq!(sum_first_two(&job), Ok(3));
    assert_eq!(job.calls.get(), 2);
}

/// > `?` performs type conversion through the `From` trait
///
/// Two hops, two conversions: the inner `?` turns `ParseIntError` into
/// [`ConfigError`], and the outer one turns [`ConfigError`] into the boxed
/// error.
pub fn question_mark_converts_one_step_per_hop() {
    // The bound written out generically.
    let boxed: Result<u8, Box<dyn Error>> = convert("nope".parse());
    assert!(boxed.unwrap_err().downcast_ref::<ParseIntError>().is_some());

    // The concrete chain.
    let error = configured("nope").unwrap_err();
    let source = error.source().expect("ConfigError keeps its ParseIntError");
    assert!(source.downcast_ref::<ParseIntError>().is_some());

    let erased = start("nope").unwrap_err();
    assert!(erased.downcast_ref::<ConfigError>().is_some());

    assert_eq!(start("8080").ok(), Some(8080));
}

/// > you can use `?` on any `Result<T, X>` where `E: From<X>`
///
/// The compile-fail half is the other direction, and the two are not
/// interchangeable: `?` can only produce the error type of the function it is
/// in.
///
/// ```compile_fail,E0277
/// fn first(value: Option<u8>) -> Result<u8, ()> {
///     Ok(value? + 1)   // the `?` operator can only be used on `Result`s
/// }
/// ```
///
/// ```compile_fail,E0277
/// fn first(value: Result<u8, ()>) -> Option<u8> {
///     Some(value? + 1)   // the `?` operator can only be used on `Option`s
/// }
/// ```
pub fn question_mark_follows_the_enclosing_function() {
    assert_eq!(first_word_len("hello world"), Some(5));
    assert_eq!(first_word_len("   "), None);
    assert_eq!(at_least_two("ab"), Ok(('a', 'b')));
    assert_eq!(at_least_two("a"), Err(()));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn question_mark_returns_early_and_leaves_the_caller_to_decide_runs() {
        question_mark_returns_early_and_leaves_the_caller_to_decide();
    }

    #[test]
    fn question_mark_converts_one_step_per_hop_runs() {
        question_mark_converts_one_step_per_hop();
    }

    #[test]
    fn question_mark_follows_the_enclosing_function_runs() {
        question_mark_follows_the_enclosing_function();
    }
}
