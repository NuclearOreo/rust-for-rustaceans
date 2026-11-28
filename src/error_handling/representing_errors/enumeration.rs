//! # Representing Errors — Enumeration
//!
//! > You have two main options for representing errors: enumeration and erasure.
//! > That is, you can either have your error type _enumerate_ the possible error
//! > conditions so that the caller can distinguish them, or you can just provide
//! > the caller with a single, _opaque_ error.
//!
//! > Here, the user will likely want to know whether it was the input stream or
//! > the output stream that failed. For example, in a web server, if an error
//! > occurs on the input stream while streaming a file to a client, it might be
//! > because a disk was ejected, whereas if the output stream errors, maybe the
//! > client just disconnected. The latter may be an error the server should
//! > ignore … whereas the former may require that the whole server be shut down!
//!
//! Listing 4-1 is [`CopyError`], and [`copy`] is the function that produces it.
//! The book's four requirements for an error type are each pinned to something
//! observable:
//!
//! | Requirement | What the demo checks |
//! | --- | --- |
//! | `Error`, with a useful `source` | the inner `std::io::Error` comes back out, and the chain has exactly one link |
//! | `Display` and `Debug` | `Display` is one lowercase line, no trailing punctuation; `Debug` carries the variant and the I/O kind |
//! | `Send` + `Sync` | [`CopyError`] fits `io::Error::new`, which requires them, and survives the round trip |
//! | `'static` | same assertion — no lifetime parameter is even possible |
//!
//! The last demo, [`CarelessCopyError`], is the cautionary half: `?` performs
//! `From<IoError>` conversion, but a `From` impl sees the *error*, never the
//! call site, so it cannot know which stream failed. Enumeration only
//! distinguishes what you deliberately put into the type.

use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};

// ---------------------------------------------------------------------------
// Listing 4-1
// ---------------------------------------------------------------------------

/// Listing 4-1: an enumerated error type.
///
/// > Each variant also includes the error that was encountered to provide the
/// > caller with as much information about went wrong as possible.
#[derive(Debug)]
pub enum CopyError {
    /// Reading from the input stream failed.
    In(io::Error),
    /// Writing to the output stream failed.
    Out(io::Error),
}

/// > your implementation of `Display` should give a one-line description of what
/// > went wrong that can easily be folded into other error messages. The display
/// > format should be lowercase and without trailing punctuation
impl fmt::Display for CopyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::In(error) => write!(f, "failed to read from the input stream: {error}"),
            Self::Out(error) => write!(f, "failed to write to the output stream: {error}"),
        }
    }
}

/// > The main method of interest is `Error::source` , which provides a mechanism
/// > to find the underlying cause of an error.
///
/// > we match on `self` and extract and return the inner `std::io::Error`
impl Error for CopyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::In(error) | Self::Out(error) => Some(error),
        }
    }
}

/// Copies every byte from `input` into `output`, stopping at the first failure.
///
/// > it's entirely possible for either stream to fail, at which point the copy
/// > has to stop and return an error to the user
///
/// The `map_err` calls are the whole point of enumeration: the variant is chosen
/// by the *call site*, because only the call site knows which stream it was.
pub fn copy(mut input: impl Read, mut output: impl Write) -> Result<u64, CopyError> {
    let mut buffer = [0u8; 64];
    let mut copied = 0;

    loop {
        let read = input.read(&mut buffer).map_err(CopyError::In)?;
        if read == 0 {
            return Ok(copied);
        }
        output.write_all(&buffer[..read]).map_err(CopyError::Out)?;
        copied += read as u64;
    }
}

/// > in a web server … the latter may be an error the server should ignore …
/// > whereas the former may require that the whole server be shut down!
pub fn should_shut_down(error: &CopyError) -> bool {
    match error {
        CopyError::In(_) => true,
        CopyError::Out(_) => false,
    }
}

// ---------------------------------------------------------------------------
// The streams used by the demos
// ---------------------------------------------------------------------------

/// A reader that always fails, standing in for the ejected disk.
struct EjectedDisk;

impl Read for EjectedDisk {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "the disk was ejected",
        ))
    }
}

/// A writer that always fails, standing in for the disconnected client.
struct DisconnectedClient;

impl Write for DisconnectedClient {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::ConnectionReset,
            "the client disconnected",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// What the enumeration buys the caller
// ---------------------------------------------------------------------------

/// > the user will likely want to know whether it was the input stream or the
/// > output stream that failed
///
/// The same `io::Error` on two different streams becomes two different variants,
/// and the caller's response differs with the variant rather than with the
/// error's own kind.
pub fn listing_4_1_the_caller_can_tell_which_side_failed() {
    // The happy path: `&[u8]` is a `Read`, and `Vec<u8>` is a `Write`.
    let mut output = Vec::new();
    let copied = copy(&b"hello"[..], &mut output).expect("in-memory copy cannot fail");
    assert_eq!(copied, 5);
    assert_eq!(output, b"hello");

    // An input failure.
    let error = copy(EjectedDisk, &mut output).expect_err("the reader always fails");
    assert!(matches!(error, CopyError::In(_)));
    assert_eq!(
        error.to_string(),
        "failed to read from the input stream: the disk was ejected"
    );

    // An output failure.
    let error = copy(&b"hello"[..], DisconnectedClient).expect_err("the writer always fails");
    assert!(matches!(error, CopyError::Out(_)));
    assert_eq!(
        error.to_string(),
        "failed to write to the output stream: the client disconnected"
    );

    // Which is exactly the distinction that decides "shut down or keep serving".
    let doomed = copy(EjectedDisk, &mut Vec::new()).unwrap_err();
    let survivable = copy(&b"hello"[..], DisconnectedClient).unwrap_err();
    assert!(should_shut_down(&doomed));
    assert!(!should_shut_down(&survivable));
}

/// > First, your error type should implement the `std::error::Error` trait,
/// > which provides callers with common methods for introspecting error types.
///
/// > `Debug` should provide a more descriptive error including auxiliary
/// > information that may be useful in tracking down the cause of the error
pub fn an_error_exposes_its_cause_and_prints_two_ways() {
    let error = copy(EjectedDisk, &mut Vec::new()).unwrap_err();

    // `Display`: one lowercase line, no trailing punctuation.
    let display = error.to_string();
    assert!(display.starts_with("failed to read"));
    assert!(!display.ends_with('.'));
    assert!(display.chars().next().is_some_and(char::is_lowercase));

    // `Debug`: the variant and the I/O kind, which `derive` is usually enough for.
    let debug = format!("{error:?}");
    assert!(debug.contains("In"), "{debug}");
    assert!(debug.contains("BrokenPipe"), "{debug}");
    assert!(debug.contains("the disk was ejected"), "{debug}");

    // `source`: the inner error, and nothing below it.
    let source = error.source().expect("the inner io::Error is the cause");
    assert_eq!(source.to_string(), "the disk was ejected");
    assert!(
        source.downcast_ref::<io::Error>().is_some(),
        "the cause is the io::Error itself"
    );
    assert!(
        source.source().is_none(),
        "the chain is exactly one link deep"
    );
}

/// > Third, your type should, if possible, implement both `Send` and `Sync` so
/// > that users are able to share the error across thread boundaries.
///
/// > Error types that implement `Send` and `Sync` are also much easier to use
/// > with the very common `std::io::Error` type
///
/// The assertion is a compile-time one; the `io::Error` round trip is the
/// observable payoff.
pub fn an_error_is_send_sync_and_static() {
    /// The book's fourth requirement, and the three bounds `io::Error` demands.
    fn assert_error<E: Error + Send + Sync + 'static>() {}

    assert_error::<CopyError>();

    // `io::Error::other` requires `Into<Box<dyn Error + Send + Sync>>`, so this
    // line would not compile if any of the three bounds were missing.
    let wrapped = io::Error::other(copy(EjectedDisk, &mut Vec::new()).unwrap_err());
    let recovered = wrapped
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<CopyError>())
        .expect("the wrapped error is still a CopyError");
    assert!(matches!(recovered, CopyError::In(_)));
}

// ---------------------------------------------------------------------------
// The cautionary half: `From` cannot see the call site
// ---------------------------------------------------------------------------

/// An error type that lets `?` convert automatically, and therefore loses the
/// one distinction the enum was supposed to carry.
#[derive(Debug, PartialEq, Eq)]
pub enum CarelessCopyError {
    In,
    Out,
}

/// > `?` performs type conversion through the `From` trait
///
/// A `From<io::Error>` impl is handed the error and nothing else, so it has to
/// pick a variant without knowing which stream produced it.
impl From<io::Error> for CarelessCopyError {
    fn from(_: io::Error) -> Self {
        Self::In
    }
}

/// The same copy loop, with `?` converting instead of `map_err` labelling.
pub fn careless_copy(
    mut input: impl Read,
    mut output: impl Write,
) -> Result<u64, CarelessCopyError> {
    let mut buffer = [0u8; 64];
    let mut copied = 0;

    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            return Ok(copied);
        }
        output.write_all(&buffer[..read])?;
        copied += read as u64;
    }
}

/// > you can either have your error type _enumerate_ the possible error
/// > conditions so that the caller can distinguish them
///
/// Enumeration is a property of the *type*, not of `?`: this type has both
/// variants, and the conversion still reports the wrong one.
pub fn conversion_alone_cannot_say_which_stream_failed() {
    let error = careless_copy(&b"hello"[..], DisconnectedClient).unwrap_err();
    assert_eq!(
        error,
        CarelessCopyError::In,
        "the writer failed, but `From` chose `In`"
    );

    // The deliberate version, for contrast, gets it right.
    let error = copy(&b"hello"[..], DisconnectedClient).unwrap_err();
    assert!(matches!(error, CopyError::Out(_)));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_4_1_the_caller_can_tell_which_side_failed_runs() {
        listing_4_1_the_caller_can_tell_which_side_failed();
    }

    #[test]
    fn an_error_exposes_its_cause_and_prints_two_ways_runs() {
        an_error_exposes_its_cause_and_prints_two_ways();
    }

    #[test]
    fn an_error_is_send_sync_and_static_runs() {
        an_error_is_send_sync_and_static();
    }

    #[test]
    fn conversion_alone_cannot_say_which_stream_failed_runs() {
        conversion_alone_cannot_say_which_stream_failed();
    }
}
