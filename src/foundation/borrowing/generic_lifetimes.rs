//! # Generic Lifetimes
//!
//! "Occasionally you need to store references within your own types. Those
//! references need to have a lifetime so that the borrow checker can check their
//! validity when they are used in the various methods on that type."
//!
//! The book flags two subtleties that only show up once your own types get
//! involved:
//!
//! 1. **Implementing `Drop` makes dropping count as a use** of every generic
//!    lifetime and type parameter. "Essentially, when an instance of your type is
//!    dropped, the borrow checker will check that it's still legal to use any of
//!    your type's generic lifetimes before dropping it." Without `Drop`, dropping
//!    is not a use at all.
//! 2. **Multiple lifetime parameters are usually unnecessary** — a type generic
//!    over one lifetime will take the shorter of the lifetimes it is given. You
//!    need several only when methods must return references tied to one specific
//!    borrow.
//!
//! Listing 1-10 is the standard-bearer for the second point, so it is implemented
//! here for real rather than left as `todo!()`.

// ---------------------------------------------------------------------------
// Subtlety 1: `Drop` turns dropping into a use
// ---------------------------------------------------------------------------

/// Holds a reference but does not implement `Drop`, so dropping it *is not a
/// use* of `'a`. The borrow only has to last as long as the value is actually
/// used, which is why this compiles even though `x` is declared after the holder.
pub struct NoDrop<'a> {
    pub reference: &'a i32,
}

/// Identical shape, but with a `Drop` impl — and now dropping it *does* count as
/// a use of `'a`. The borrow must stay alive until the value is dropped, so the
/// same declaration order that was fine above is now rejected.
///
/// ```compile_fail,E0597
/// struct WithDrop<'a>(&'a i32);
/// impl Drop for WithDrop<'_> {
///     fn drop(&mut self) {}
/// }
///
/// let holder;
/// let x = 5;
/// holder = WithDrop(&x);   // `x` does not live long enough
/// ```
///
/// The reason is defensive: the checker cannot see inside `drop`, so it must
/// assume your `Drop` code might touch the reference.
pub struct WithDrop<'a> {
    pub reference: &'a i32,
}

impl Drop for WithDrop<'_> {
    fn drop(&mut self) {
        // A real implementation might well read `self.reference` here.
        let _ = self.reference;
    }
}

// The late initialization is the demonstration, not an accident: `holder` is
// declared before `x` so that `x` is dropped first, which is only legal because
// `NoDrop` makes dropping a non-use.
#[allow(clippy::needless_late_init)]
pub fn without_drop_dropping_is_not_a_use() {
    let holder;
    let x = 5;

    holder = NoDrop { reference: &x };
    assert_eq!(*holder.reference, 5);
    // `holder` is never used again, so the borrow of `x` ends here even though
    // `holder` is dropped after `x`.
}

pub fn with_drop_the_borrow_must_outlive_the_drop() {
    let x = 5; // declared first, so it outlives `holder`
    let holder = WithDrop { reference: &x };

    assert_eq!(*holder.reference, 5);
    // `holder` is dropped before `x` here, which is what the `Drop` rule demands.
}

// ---------------------------------------------------------------------------
// Subtlety 2: when you actually need more than one lifetime
// ---------------------------------------------------------------------------

/// Listing 1-10: *A type that needs to be generic over multiple lifetimes.*
///
/// The iterator borrows a `document` (`'s`) and a `delimiter` (`'p`), but yields
/// slices **of the document**. `Item = &'s str` says exactly that: the yielded
/// strings are tied to the document, not to the delimiter.
///
/// With a single lifetime, the yielded slices would be tied to *both*, and
/// [`str_before`] below would be impossible to write — its delimiter is a local
/// `String` that dies at the end of the function.
pub struct StrSplit<'s, 'p> {
    delimiter: &'p str,
    document: &'s str,
}

impl<'s, 'p> StrSplit<'s, 'p> {
    pub fn new(delimiter: &'p str, document: &'s str) -> Self {
        Self {
            delimiter,
            document,
        }
    }
}

impl<'s, 'p> Iterator for StrSplit<'s, 'p> {
    type Item = &'s str;

    fn next(&mut self) -> Option<&'s str> {
        // Copy the `&'s str` out, so slicing it yields `&'s str` rather than a
        // borrow of `self`.
        let document: &'s str = self.document;
        let delimiter = self.delimiter;

        if document.is_empty() {
            return None;
        }

        // Deliberately simplified: an empty delimiter would match forever, which
        // the book's exercise is meant to address.
        match document.find(delimiter) {
            Some(index) => {
                self.document = &document[index + delimiter.len()..];
                Some(&document[..index])
            }
            None => {
                self.document = "";
                Some(document)
            }
        }
    }
}

/// The function the book uses to justify two lifetime parameters. It returns a
/// slice of `s`, while the delimiter it builds is a local temporary — legal only
/// because `Item` is tied to `'s` alone.
///
/// With one lifetime for both fields, the return value would borrow from the
/// temporary `String`, and the function could not compile:
///
/// ```compile_fail,E0515
/// struct StrSplitSingle<'a> {
///     delimiter: &'a str,
///     document: &'a str,
/// }
/// impl<'a> Iterator for StrSplitSingle<'a> {
///     type Item = &'a str;
///     fn next(&mut self) -> Option<&'a str> { None }
/// }
///
/// fn str_before(s: &str, c: char) -> Option<&str> {
///     // cannot return value referencing temporary value
///     StrSplitSingle { document: s, delimiter: &c.to_string() }.next()
/// }
/// ```
pub fn str_before(document: &str, delimiter: char) -> Option<&str> {
    StrSplit::new(&delimiter.to_string(), document).next()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn without_drop_dropping_is_not_a_use_runs() {
        without_drop_dropping_is_not_a_use();
    }

    #[test]
    fn with_drop_the_borrow_must_outlive_the_drop_runs() {
        with_drop_the_borrow_must_outlive_the_drop();
    }

    #[test]
    fn str_split_yields_parts_of_the_document() {
        let parts: Vec<&str> = StrSplit::new(", ", "a, b, c").collect();
        assert_eq!(parts, ["a", "b", "c"]);
    }

    #[test]
    fn str_before_returns_a_slice_of_its_argument() {
        assert_eq!(str_before("hello world", ' '), Some("hello"));

        // No delimiter found: this simplified iterator yields the whole document
        // rather than reporting the end of iteration.
        assert_eq!(str_before("nodelimiter", ' '), Some("nodelimiter"));
    }
}
