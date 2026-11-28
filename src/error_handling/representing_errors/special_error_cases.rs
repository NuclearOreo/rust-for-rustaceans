//! # Representing Errors — Special Error Cases
//!
//! > Some functions are fallible but cannot return any meaningful error if they
//! > fail. Conceptually, these functions have a return type of `Result<T, ()>` .
//! > In some codebases, you may see this represented as `Option<T>` instead.
//! > While both are legitimate choices for the return type for such a function,
//! > they convey different semantic meanings, and you should usually avoid
//! > "simplifying" a `Result<T, ()>` to `Option<T>` .
//!
//! > **NOTE** _You should also keep in mind that `()` does not implement the_
//! > _`Error` trait. This means that it cannot be type-erased into_
//! > _`Box<dyn Error>` and can be a bit of a pain to use with `?` . For this_
//! > _reason, it is often better to define your own unit struct type, implement_
//! > _`Error` for it, and use that as the error instead of `()` in these cases._
//!
//! > Some functions, like those that start a continuously running server loop,
//! > only ever return errors; unless an error occurs, they run forever. … For
//! > functions like these, Rust provides the _never type_ , written with the `!`
//! > syntax.
//!
//! > One last curious error case is the error type `std::thread::Result` .
//!
//! The subsection's four cases, each with its own rejection:
//!
//! | Case | Demo | Rejection |
//! | --- | --- | --- |
//! | `Result<T, ()>` vs `Option<T>` | a retry loop vs an ordinary "not found" | an `Option` is not a `Result` (`E0308`) |
//! | `()` is not an `Error` | a unit struct that *is* one survives `?` | `Box::<dyn Error>::from(())` (`E0277`) |
//! | the never type | `Result<T, !>` cannot carry `Err`, `Result<!, E>` cannot carry `Ok` | `Err(())` in `Result<T, !>` (`E0308`) |
//! | `std::thread::Result` | a panic payload is a `dyn Any`, not a `dyn Error` | `?` on a join error (`E0277`) |

use std::any::Any;
use std::error::Error;
use std::fmt;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};

// ---------------------------------------------------------------------------
// `Result<T, ()>` versus `Option<T>`
// ---------------------------------------------------------------------------

/// Looking somebody up: "no such user" is an ordinary answer, not a failure.
pub fn find_user(name: &str) -> Option<u32> {
    match name {
        "ada" => Some(1),
        _ => None,
    }
}

/// Connecting to a service: failure is exceptional and may be worth retrying.
///
/// The `()` is the point — there is nothing to say beyond "it did not work".
///
/// # Errors
///
/// Returns `Err(())` for the first three attempts.
// `Result<T, ()>` is the book's subject here, not an oversight.
#[allow(clippy::result_unit_err)]
pub fn connect(attempt: u8) -> Result<u32, ()> {
    if attempt < 3 { Err(()) } else { Ok(7) }
}

// ---------------------------------------------------------------------------
// An error that can be erased, unlike `()`
// ---------------------------------------------------------------------------

/// The book's replacement for `()`: a unit struct that implements `Error`.
#[derive(Debug, PartialEq, Eq)]
pub struct NotConnected;

impl fmt::Display for NotConnected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the service is not connected")
    }
}

impl Error for NotConnected {}

/// The same failure as [`connect`], with an error that can be erased.
///
/// # Errors
///
/// Always returns [`NotConnected`].
pub fn open_connection() -> Result<u32, NotConnected> {
    Err(NotConnected)
}

/// > it cannot be type-erased into `Box<dyn Error>` and can be a bit of a pain
/// > to use with `?`
///
/// The unit type is the one error `Box` will not accept:
///
/// ```compile_fail,E0277
/// use std::error::Error;
///
/// let boxed: Box<dyn Error> = Box::from(());
/// ```
pub fn unit_is_not_an_error_but_a_unit_struct_can_be() {
    // `()` cannot be boxed, so a `?` on `connect` would not compile in a
    // function returning `Box<dyn Error>`.
    //
    // `NotConnected` can: it implements `Error`, `Display`, `Debug`, and is
    // trivially `Send + Sync + 'static`.
    fn serve() -> Result<u32, Box<dyn Error>> {
        let port = open_connection()?;
        Ok(port)
    }

    let error = serve().unwrap_err();
    assert_eq!(error.to_string(), "the service is not connected");
    assert_eq!(error.downcast_ref::<NotConnected>(), Some(&NotConnected));
}

// ---------------------------------------------------------------------------
// The never type
// ---------------------------------------------------------------------------

/// A function whose `Err` side can never be produced.
///
/// > If you write a function that returns `Result<T, !>` , you will be unable to
/// > ever return `Err` , since the only way to do so is to enter code that will
/// > never return.
///
/// # Errors
///
/// Never.
pub fn never_fails(text: &str) -> Result<u32, !> {
    Ok(text.len() as u32)
}

/// > Some functions, like those that start a continuously running server loop,
/// > only ever return errors; unless an error occurs, they run forever.
///
/// The `Ok` side is `!`, so this function cannot return success at all.
///
/// # Errors
///
/// Returns an error when there are no connections left to serve.
pub fn serve_forever(connections: usize) -> Result<!, io::Error> {
    if connections == 0 {
        return Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            "no connections left to serve",
        ));
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

/// A function that never returns normally: the other way to make a `!`.
pub fn abort_with(message: &'static str) -> ! {
    panic!("{message}")
}

// ---------------------------------------------------------------------------
// `std::thread::Result`
// ---------------------------------------------------------------------------

/// The value a panicking thread can carry: any type at all.
#[derive(Debug)]
pub struct Explosion {
    pub code: u32,
}

/// Joins a thread that is expected to panic and returns its payload.
fn panic_payload<F: FnOnce() + Send + 'static>(body: F) -> Box<dyn Any + Send + 'static> {
    std::thread::spawn(body)
        .join()
        .expect_err("the spawned thread was supposed to panic")
}

// ---------------------------------------------------------------------------
// Demos
// ---------------------------------------------------------------------------

/// > An `Err(())` indicates that an operation failed and should be retried,
/// > reported, or otherwise handled exceptionally. `None` , on the other hand,
/// > conveys only that the function has nothing to return
///
/// The same "no value" shape leads to two different caller behaviours: the
/// `Option` is an answer, the `Result` is a retry loop.
pub fn result_unit_and_option_mean_different_things() {
    // An ordinary miss: nothing to handle, and nothing to retry.
    let missing = find_user("grace");
    assert_eq!(missing, None);
    assert_eq!(find_user("ada"), Some(1));

    // An exceptional failure: retry until it works, then stop.
    let mut attempt = 0;
    let port = loop {
        match connect(attempt) {
            Ok(port) => break port,
            Err(()) => attempt += 1,
        }
    };
    assert_eq!((attempt, port), (3, 7));

    // `Result` is `#[must_use]`, `Option` is not: discarding the lookup is
    // silent, and discarding the connection attempt warns. The warning (a lint,
    // so it has no error code) is quoted rather than compiled:
    //
    // ```text
    // warning: unused `Result` that must be used
    //  --> src/main.rs:2:5
    //   |
    // 2 |     connect(3);
    //   |     ^^^^^^^^^^^
    //   |
    //   = note: `#[warn(unused_must_use)]` on by default
    // ```
    let _ignored = find_user("nobody");
    #[allow(unused_must_use)]
    connect(3);
}

/// > they convey different semantic meanings, and you should usually avoid
/// > "simplifying" a `Result<T, ()>` to `Option<T>`
///
/// The two are not interchangeable even when the success types agree:
///
/// ```compile_fail,E0308
/// fn find_user(name: &str) -> Option<u32> {
///     if name == "ada" { Some(1) } else { None }
/// }
///
/// let found: Result<u32, ()> = find_user("ada");
/// ```
pub fn an_option_is_not_a_result_even_with_the_same_success_type() {
    let as_option: Option<u32> = find_user("ada");
    let as_result: Result<u32, ()> = connect(3);
    assert_eq!(as_option, Some(1));
    assert_eq!(as_result, Ok(7));
}

/// > And when you pattern match, the compiler knows that any variant that
/// > contains a `!` does not even need to be listed.
///
/// Two exhausted matches with a missing arm each:
///
/// ```compile_fail,E0308
/// fn never_fails() -> Result<u32, !> {
///     Err(())   // expected `!`, found `()`
/// }
/// ```
// The single-arm `match` *is* the demonstration; clippy's suggested `let Ok(..) =`
// would show irrefutability but not that the arm is absent from a match.
#[allow(clippy::infallible_destructuring_match)]
pub fn the_never_type_removes_a_variant_from_the_match() {
    // `Err` is uninhabited, so it needs no arm.
    let value = match never_fails("four") {
        Ok(length) => length,
    };
    assert_eq!(value, 4);

    // `Ok` is uninhabited, so it needs no arm.
    let error = match serve_forever(0) {
        Err(error) => error,
    };
    assert_eq!(error.kind(), io::ErrorKind::AddrInUse);

    // And diverging is how a `!` value is made: the closure never returns.
    let unwound = catch_unwind(AssertUnwindSafe(|| abort_with("fatal")));
    assert!(
        unwound.is_err(),
        "`abort_with` panicked rather than returning"
    );
}

/// > The reason for this curious-looking error type is that the error variant of
/// > `std::thread::Result` is produced only in response to a panic
///
/// > In essence, the error type is "a panic" and the value is "whatever argument
/// > was passed to `panic!` ," which can truly be any type
pub fn a_join_error_is_the_panic_argument() {
    // `std::thread::Result<T>` is exactly this alias; the coercion is the check.
    fn alias<T>(result: std::thread::Result<T>) -> Result<T, Box<dyn Any + Send + 'static>> {
        result
    }

    // A literal panic carries a `&'static str`.
    let payload = alias(std::thread::spawn(|| panic!("literal boom")).join()).unwrap_err();
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"literal boom"));
    assert!(payload.downcast_ref::<String>().is_none());

    // A formatted panic carries an owned `String`.
    let code = 7;
    let payload = panic_payload(move || panic!("formatted {code}"));
    assert_eq!(
        payload.downcast_ref::<String>().map(String::as_str),
        Some("formatted 7")
    );
    assert!(payload.downcast_ref::<&str>().is_none());

    // Debug is available — which is why `join().unwrap()` compiles — but it says
    // nothing at all, and that is the whole message:
    assert_eq!(format!("{payload:?}"), "Any { .. }");

    // And an arbitrary type, via `panic_any`.
    let payload = panic_payload(|| std::panic::panic_any(Explosion { code: 42 }));
    assert_eq!(
        payload.downcast_ref::<Explosion>().map(|e| e.code),
        Some(42)
    );
    assert!(payload.downcast_ref::<&str>().is_none());
}

/// > the error type is "a panic" … which is not much of a guarantee at all
///
/// A join error is a `dyn Any`, so it is not an `Error` and cannot be erased
/// into one:
///
/// ```compile_fail,E0277
/// use std::any::Any;
/// use std::error::Error;
///
/// fn serve() -> Result<u8, Box<dyn Error>> {
///     let value = std::thread::spawn(|| 1u8).join()?;   // `Box<dyn Any + Send>`
///                                                       // is not an `Error`
///     Ok(value)
/// }
/// ```
pub fn a_panic_payload_cannot_be_propagated_as_an_error() {
    let payload: Box<dyn Any + Send> = panic_payload(|| panic!("boom"));

    // The only two moves the type allows: ask what it is, or give up and panic
    // with the uninformative `Any { .. }` message.
    assert!(payload.downcast_ref::<&str>().is_some());
    assert!(payload.downcast_ref::<Box<dyn Error>>().is_none());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn result_unit_and_option_mean_different_things_runs() {
        result_unit_and_option_mean_different_things();
    }

    #[test]
    fn an_option_is_not_a_result_even_with_the_same_success_type_runs() {
        an_option_is_not_a_result_even_with_the_same_success_type();
    }

    #[test]
    fn the_never_type_removes_a_variant_from_the_match_runs() {
        the_never_type_removes_a_variant_from_the_match();
    }

    #[test]
    fn a_join_error_is_the_panic_argument_runs() {
        a_join_error_is_the_panic_argument();
    }

    #[test]
    fn a_panic_payload_cannot_be_propagated_as_an_error_runs() {
        a_panic_payload_cannot_be_propagated_as_an_error();
    }
}
