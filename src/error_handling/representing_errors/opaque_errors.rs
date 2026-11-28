//! # Representing Errors — Opaque Errors
//!
//! > If the decoding fails, the user needs to be able to figure out how to
//! > resolve the issue, and so must understand the cause. But is it important
//! > whether the cause is the `size` field in the image header being invalid, or
//! > the compression algorithm failing to decompress a block? Probably not—the
//! > application can't meaningfully recover from either situation, even if it
//! > knows the exact cause. In cases like this, you as the library author may
//! > instead want to provide a single, opaque error type.
//!
//! > It could just be a type with all private fields that exposes only limited
//! > methods for displaying and introspecting the error, or it could be a
//! > severely type-erased error type like
//! > `Box<dyn Error + Send + Sync + 'static>`
//!
//! > if the error has some interesting facets to it, such as a line number or a
//! > status code, you may want to expose that through a concrete but opaque type
//! > instead.
//!
//! The two ends of that spectrum are [`DecodeError`] and [`BoxError`]:
//!
//! | Module item | Opacity | What the caller can do |
//! | --- | --- | --- |
//! | [`DecodeError`] | concrete, private fields | read [`DecodeError::line`] / [`DecodeError::status`], print it, bubble it up |
//! | [`BoxError`] | erased | print it, bubble it up, `downcast_ref` back to a concrete type |
//!
//! The two errors that are *not* here are the two rejections on their docs:
//! `Box<dyn Error>` does not implement `Error`, so it cannot be used where a
//! generic `E: Error` is expected (`E0277`), and an error that is not `'static`
//! cannot be downcast at all (`E0521`) — which is the reason the book insists on
//! the `'static` bound.

use std::any::{Any, TypeId};
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// A concrete but opaque error
// ---------------------------------------------------------------------------

/// The really fine-grained causes of a decode failure.
///
/// > You might internally represent more fine-grained error states, but there is
/// > no need to expose those to the users of the library.
///
/// Private, so no caller can name it, match on it, or add an arm for it. The
/// rejection is a doctest on [`DecodeError`].
#[derive(Debug)]
enum Cause {
    /// The `IMG1` header is missing or wrong.
    Header,
    /// The `width height` line is missing, malformed, or zero-sized.
    Size,
    /// The pixel data is shorter than the size line promised.
    Truncated,
    /// A compression block would not decompress; the payload is the column.
    Block(u32),
}

impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header => f.write_str("the header is not a valid IMG1 header"),
            Self::Size => f.write_str("the size field is invalid"),
            Self::Truncated => f.write_str("the pixel data ends before the declared size"),
            Self::Block(column) => write!(f, "compression block at column {column} is corrupt"),
        }
    }
}

/// A concrete error whose fields are private and whose facets are readable.
///
/// > if the error has some interesting facets to it, such as a line number or a
/// > status code, you may want to expose that through a concrete but opaque type
/// > instead.
///
/// No variant, no cause, and no constructor is visible: the caller can read the
/// line and status, print the error, and propagate it. Everything else is the
/// library's business.
///
/// The private `cause` field is what makes the causes unreachable. From another
/// crate, a struct literal is refused because every field here is private:
///
/// ```compile_fail,E0451
/// use rust_for_rustaceans::error_handling::representing_errors::opaque_errors::DecodeError;
///
/// let error = DecodeError {
///     line: 1,
///     status: 400,
///     cause: todo!(),
/// };
/// ```
///
/// And the cause type itself cannot even be named, so no `match` can enumerate
/// it:
///
/// ```compile_fail,E0603
/// use rust_for_rustaceans::error_handling::representing_errors::opaque_errors::Cause;
///
/// let cause = Cause::Size;
/// ```
#[derive(Debug)]
pub struct DecodeError {
    line: usize,
    status: u16,
    cause: Cause,
}

impl DecodeError {
    /// Line number of the first thing that could not be decoded (1-based).
    pub fn line(&self) -> usize {
        self.line
    }

    /// A status code in the shape an HTTP client would report.
    pub fn status(&self) -> u16 {
        self.status
    }

    fn new(line: usize, status: u16, cause: Cause) -> Self {
        Self {
            line,
            status,
            cause,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to decode image at line {}: {}",
            self.line, self.cause
        )
    }
}

impl Error for DecodeError {}

/// A decoded image. Small and made-up, but enough to be a real success case.
#[derive(Debug, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<char>,
}

fn parse_size(line: &str) -> Option<(usize, usize)> {
    let mut dimensions = line.split(' ');
    let width = dimensions.next()?.parse().ok()?;
    let height = dimensions.next()?.parse().ok()?;
    if dimensions.next().is_some() || width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

/// Decodes `IMG1`, a size line, and one text row per pixel row.
///
/// The rules are deliberately simple; the point is that all four failures leave
/// as one type with two readable facets.
///
/// # Errors
///
/// Returns [`DecodeError`] when the header is missing or wrong, the size line
/// does not parse to two non-zero numbers, a row has the wrong width, the data
/// ends early, or a row contains an `X`.
pub fn decode(bytes: &[u8]) -> Result<Image, DecodeError> {
    let text = std::str::from_utf8(bytes).map_err(|_| DecodeError::new(1, 415, Cause::Header))?;
    let mut lines = text.lines();

    match lines.next() {
        Some("IMG1") => {}
        _ => return Err(DecodeError::new(1, 415, Cause::Header)),
    }

    let (width, height) =
        lines
            .next()
            .and_then(parse_size)
            .ok_or(DecodeError::new(2, 400, Cause::Size))?;

    let expected = width
        .checked_mul(height)
        .ok_or(DecodeError::new(2, 400, Cause::Size))?;
    if expected > text.len() {
        return Err(DecodeError::new(2, 400, Cause::Truncated));
    }

    let mut pixels = Vec::with_capacity(expected);
    for (index, row) in lines.enumerate() {
        let line = index + 3;
        if row.len() != width {
            return Err(DecodeError::new(line, 400, Cause::Truncated));
        }
        if let Some(column) = row.find('X') {
            return Err(DecodeError::new(line, 422, Cause::Block(column as u32)));
        }
        pixels.extend(row.chars());
    }

    if pixels.len() != expected {
        return Err(DecodeError::new(2, 400, Cause::Truncated));
    }

    Ok(Image {
        width,
        height,
        pixels,
    })
}

// ---------------------------------------------------------------------------
// A fully erased error
// ---------------------------------------------------------------------------

/// > it could be a severely type-erased error type like
/// > `Box<dyn Error + Send + Sync + 'static>`, which reveals nothing more than
/// > the fact that it is an error
///
/// > **NOTE** _While `Box<dyn Error + ...>` is an attractive type-erased error
/// > type, it counterintuitively does not itself implement `Error`. Therefore,
/// > consider adding your own `BoxError` type for type erasure in libraries that_
/// > does _implement `Error`._
///
/// The note's rejection, spelled as a generic bound:
///
/// ```compile_fail,E0277
/// use std::error::Error;
///
/// fn needs_error<E: Error>(_: E) {}
///
/// let boxed: Box<dyn Error> = "boom".into();
/// needs_error(boxed);   // `Box<dyn Error>: Error` is not satisfied
/// ```
///
/// `Box<dyn Error + Send + Sync>` is not an exception: the `Error` impl for
/// `Box<E>` requires `E: Sized`, and `dyn Error` is not.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// The custom error for a config file that has no port in it.
#[derive(Debug, PartialEq, Eq)]
pub struct MissingPort;

impl fmt::Display for MissingPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the configuration does not name a port")
    }
}

impl Error for MissingPort {}

/// The I/O half of loading a config.
pub fn read_config(path: &str) -> Result<&'static str, io::Error> {
    match path {
        "config.toml" => Ok("8080"),
        "empty.toml" => Ok(""),
        "broken.toml" => Ok("http"),
        _ => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no such configuration file",
        )),
    }
}

/// > If you write a function whose return type is `Box<dyn Error + ...>` , then
/// > you can use `?` across different error types inside that function, on all
/// > sorts of different errors, and they will all be turned into that one common
/// > error type.
///
/// Three sources — `io::Error`, `ParseIntError`, and [`MissingPort`] — meet in
/// one return type.
///
/// # Errors
///
/// Returns whatever `read_config` fails with, [`MissingPort`] for an empty
/// value, or the `ParseIntError` from parsing the rest.
pub fn load_port(path: &str) -> Result<u16, BoxError> {
    let text = read_config(path)?;
    if text.is_empty() {
        return Err(MissingPort.into());
    }
    let port = text.parse()?;
    Ok(port)
}

// ---------------------------------------------------------------------------
// What the caller can still recover
// ---------------------------------------------------------------------------

/// A request that fails differently depending on how many times it is attempted.
///
/// # Errors
///
/// `WouldBlock` on the first attempt, [`MissingPort`] on the second.
pub fn fetch(attempt: u8) -> Result<u8, BoxError> {
    match attempt {
        0 => Err(io::Error::from(io::ErrorKind::WouldBlock).into()),
        1 => Err(MissingPort.into()),
        _ => Ok(42),
    }
}

/// The book's example: act on a `WouldBlock`, and on nothing else.
///
/// > the user may want to take a particular action if the error they received was
/// > a `std::io::Error` of kind `std::io::ErrorKind::WouldBlock` , but they would
/// > not take that same action in any other case
pub fn retry_decision(error: &(dyn Error + 'static)) -> &'static str {
    match error.downcast_ref::<io::Error>() {
        Some(error) if error.kind() == io::ErrorKind::WouldBlock => "retry",
        _ => "give up",
    }
}

// ---------------------------------------------------------------------------
// Size: why errors live behind a pointer
// ---------------------------------------------------------------------------

/// A deliberately bulky concrete error.
#[derive(Debug)]
pub struct LargeError {
    pub context: [u8; 64],
}

impl fmt::Display for LargeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a wide error record")
    }
}

impl Error for LargeError {}

// ---------------------------------------------------------------------------
// Demos
// ---------------------------------------------------------------------------

/// > the application can't meaningfully recover from either situation, even if
/// > it knows the exact cause
///
/// Every failure leaves `decode` as the same type with the same two accessors,
/// and the private cause is visible only in the `Display` text.
pub fn an_opaque_error_exposes_facets_but_not_causes() {
    let image = decode(b"IMG1\n2 2\n.#\n#.").expect("a well-formed 2x2 image decodes");
    assert_eq!(
        Image {
            width: 2,
            height: 2,
            pixels: vec!['.', '#', '#', '.'],
        },
        image
    );

    // Bad magic: line 1, status 415.
    let error = decode(b"PNG\n2 2\n..\n..").unwrap_err();
    assert_eq!((error.line(), error.status()), (1, 415));
    assert_eq!(
        error.to_string(),
        "failed to decode image at line 1: the header is not a valid IMG1 header"
    );

    // Bad size: line 2, status 400.
    let error = decode(b"IMG1\nzero 2\n..\n..").unwrap_err();
    assert_eq!((error.line(), error.status()), (2, 400));
    assert_eq!(
        error.to_string(),
        "failed to decode image at line 2: the size field is invalid"
    );

    // A corrupt block: the line of the offending row, status 422.
    let error = decode(b"IMG1\n2 2\n..\n.X").unwrap_err();
    assert_eq!((error.line(), error.status()), (4, 422));
    assert_eq!(
        error.to_string(),
        "failed to decode image at line 4: compression block at column 1 is corrupt"
    );

    // Truncated data: reported against the size line, status 400.
    let error = decode(b"IMG1\n2 2\n..\n..\n..\n..\n@@").unwrap_err();
    assert_eq!((error.line(), error.status()), (2, 400));

    // All four are the same type, and it is `Error + Send + Sync + 'static`,
    // so `?` can erase it into the open-ended boxed error.
    fn summarize(bytes: &[u8]) -> Result<usize, BoxError> {
        let image = decode(bytes)?;
        Ok(image.width * image.height)
    }
    assert_eq!(summarize(b"IMG1\n2 2\n..\n..").ok(), Some(4));
    let boxed = summarize(b"PNG\n").unwrap_err();
    let error = boxed
        .downcast_ref::<DecodeError>()
        .expect("the erased error is still a DecodeError");
    assert_eq!((error.line(), error.status()), (1, 415));
}

/// > you can use `?` across different error types inside that function … and
/// > they will all be turned into that one common error type
pub fn a_boxed_error_composes_three_sources() {
    // The happy path.
    assert_eq!(load_port("config.toml").ok(), Some(8080));

    // I/O: `io::Error`, and `?` boxes it.
    let error = load_port("absent.toml").unwrap_err();
    assert_eq!(
        error.downcast_ref::<io::Error>().map(io::Error::kind),
        Some(io::ErrorKind::NotFound)
    );

    // Parsing: `std::num::ParseIntError`.
    let error = load_port("broken.toml").unwrap_err();
    assert!(error.downcast_ref::<std::num::ParseIntError>().is_some());

    // The library's own error.
    let error = load_port("empty.toml").unwrap_err();
    assert_eq!(error.downcast_ref::<MissingPort>(), Some(&MissingPort));

    // The three are distinct: an `io::Error` is not a `MissingPort`, and so on.
    let io_error = load_port("absent.toml").unwrap_err();
    assert!(io_error.downcast_ref::<MissingPort>().is_none());
    assert!(io_error.downcast_ref::<std::num::ParseIntError>().is_none());
}

/// > downcasting allows a user to turn a `dyn Error` into a concrete underlying
/// > error type when that `dyn Error` was originally of that type
///
/// > The `downcast_ref` method returns an `Option` , which tells the user
/// > whether or not the downcast succeeded.
pub fn downcasting_lets_the_caller_act_on_one_specific_error() {
    let would_block = fetch(0).unwrap_err();
    assert_eq!(retry_decision(&*would_block), "retry");

    // The same opaque type, a different concrete error: no retry.
    let missing = fetch(1).unwrap_err();
    assert_eq!(retry_decision(&*missing), "give up");

    assert_eq!(fetch(2).ok(), Some(42));
}

/// > `downcast_ref` works only if the argument is `'static` . If we return an
/// > opaque `Error` that's not `'static` , we take away the user's ability to do
/// > this kind of error introspection
///
/// The compiler refuses to downcast behind a borrowed trait object at all. The
/// error is not "the downcast returned `None`"; it is that the method cannot be
/// called:
///
/// ```compile_fail,E0521
/// use std::error::Error;
///
/// fn inspect<'a>(error: Box<dyn Error + 'a>) -> bool {
///     error.downcast_ref::<std::io::Error>().is_some()
/// }
/// ```
pub fn only_static_errors_can_be_downcast() {
    // The `'static` version the alias insists on, for contrast.
    let boxed: BoxError = io::Error::from(io::ErrorKind::WouldBlock).into();
    assert!(boxed.downcast_ref::<io::Error>().is_some());
}

/// > The answer lies in the compiler-supported type `std::any::TypeId` , which
/// > allows you to get a unique identifier for any type.
///
/// > `Any` has a blanket implementation of `impl Any for T` , and in that
/// > implementation, its `type_id` returns the same.
///
/// `Error::type_id` is a hidden, unstable provided method, so the mechanism is
/// observed through `Any` — which is the trait `std::thread::Result` erases to.
///
/// The trap: `erased.type_id()` on a `Box<dyn Any>` is `Any for Box<dyn Any>`,
/// the id of the *box*, not of the value inside it. The trait object has to be
/// reached explicitly through the deref, which is also why `downcast_ref` is a
/// method of `dyn Any` rather than of `Box<dyn Any>`.
pub fn type_id_is_what_makes_a_downcast_safe() {
    let mut erased: Box<dyn Any> = Box::new(7u8);
    let other: Box<dyn Any> = Box::new(7u16);

    // Every `Box<dyn Any>` reports the same id: the box is the type here. The
    // UFCS spells out which `Any` impl is answering — clippy's `type_id_on_box`
    // is this same trap, and the method-call spelling is what it warns about.
    let box_id = <Box<dyn Any> as Any>::type_id(&erased);
    assert_eq!(box_id, TypeId::of::<Box<dyn Any>>());
    assert_eq!(box_id, <Box<dyn Any> as Any>::type_id(&other));

    // Through the deref, `Any` forwards to the vtable's `type_id`, whose `Self`
    // is the concrete type inside. Note that `&erased` would not do: a `&Box<T>`
    // coerces to `&dyn Any` with `T = Box<dyn Any>`, the box type again.
    assert_eq!((*erased).type_id(), TypeId::of::<u8>());
    assert_ne!((*erased).type_id(), TypeId::of::<u16>());
    assert_ne!((*erased).type_id(), (*other).type_id());

    // Which is exactly what the downcast compares.
    assert_eq!(erased.downcast_ref::<u8>(), Some(&7));
    assert!(erased.downcast_ref::<u16>().is_none());
    assert_eq!(other.downcast_ref::<u16>(), Some(&7));

    // The same slot, a different real type underneath.
    erased = Box::new(String::from("seven"));
    assert_eq!((*erased).type_id(), TypeId::of::<String>());
    assert!(erased.downcast_ref::<u8>().is_none());
    assert_eq!(
        erased.downcast_ref::<String>().map(String::as_str),
        Some("seven")
    );
}

/// > errors are often placed behind a pointer type, such as a `Box` or an `Arc` .
/// > This way, they're unlikely to add much to the size of the overall `Result`
/// > type they're contained within.
pub fn boxing_keeps_the_result_small_and_arcs_share() {
    let inline = size_of::<Result<u32, LargeError>>();
    let boxed = size_of::<Result<u32, BoxError>>();
    assert!(
        inline >= 64,
        "the inline error is as wide as its payload: {inline}"
    );
    assert_eq!(boxed, size_of::<BoxError>());
    assert!(boxed < inline);

    // `Arc` is the other pointer the note names: cloning it shares the error.
    let shared: Arc<dyn Error + Send + Sync> = Arc::new(LargeError { context: [0; 64] });
    let also = Arc::clone(&shared);
    assert_eq!(Arc::strong_count(&shared), 2);
    assert!(also.downcast_ref::<LargeError>().is_some());
    assert_eq!(
        size_of::<Result<u32, Arc<dyn Error + Send + Sync>>>(),
        size_of::<Arc<dyn Error + Send + Sync>>()
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_opaque_error_exposes_facets_but_not_causes_runs() {
        an_opaque_error_exposes_facets_but_not_causes();
    }

    #[test]
    fn a_boxed_error_composes_three_sources_runs() {
        a_boxed_error_composes_three_sources();
    }

    #[test]
    fn downcasting_lets_the_caller_act_on_one_specific_error_runs() {
        downcasting_lets_the_caller_act_on_one_specific_error();
    }

    #[test]
    fn only_static_errors_can_be_downcast_runs() {
        only_static_errors_can_be_downcast();
    }

    #[test]
    fn type_id_is_what_makes_a_downcast_safe_runs() {
        type_id_is_what_makes_a_downcast_safe();
    }

    #[test]
    fn boxing_keeps_the_result_small_and_arcs_share_runs() {
        boxing_keeps_the_result_small_and_arcs_share();
    }
}
