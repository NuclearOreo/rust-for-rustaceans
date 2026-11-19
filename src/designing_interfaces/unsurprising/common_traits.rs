//! # Common Traits for Types
//!
//! > Users in Rust will also make the major assumption that everything in the
//! > interface "just works." They expect to be able to print any type with
//! > `{:?}` and send anything and everything to another thread, and they expect
//! > that every type is `Clone` . Where possible, we should again avoid
//! > surprising the user and eagerly implement most of the standard traits even
//! > if we do not need them immediately.
//!
//! > Because the coherence rules discussed in Chapter 2, the compiler will not
//! > allow users to implement these traits when they need them.
//!
//! The order in which the book ranks them is the order of the files' demos, and
//! each demo ends at the same question: what can the user no longer do if this
//! trait is missing?
//!
//! | Trait(s) | Why the book puts it here | Demo |
//! | --- | --- | --- |
//! | `Debug` | "Nearly every type can, and should, implement `Debug`" | `{:?}` prints, and `assert_eq!` works |
//! | `Send` / `Sync` / `Unpin` | a non-`Send` type cannot cross a thread | the type can be moved and shared |
//! | `Clone` / `Default` | users expect to make more values | `T::default()` and an independent clone |
//! | `PartialEq` / `PartialOrd` / `Hash` / `Eq` / `Ord` | comparison, map keys, sets | `==`, sorting, `HashSet`, `HashMap` |
//! | `Copy` | *deliberately not* in the list | a move stops moving, and adding one field can remove the impl |
//!
//! `serde`'s `Serialize`/`Deserialize` are the book's last recommendation. They
//! arrive behind an optional feature precisely so users who do not need them do
//! not pay for them; this crate has no dependencies until Chapter 8, so that one
//! is recorded here rather than demonstrated.
//!
//! # The trap in "just derive it"
//!
//! > keep in mind that all derived traits automatically add the same bound for
//! > any generic parameters.
//!
//! `#[derive(Debug)] struct Wrapper<T>(T)` prints only for `T: Debug`, even if
//! the type could describe itself without looking at `T`. The second `Debug`
//! demo writes the impl by hand and shows the bound going away.

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Debug
// ---------------------------------------------------------------------------

/// The derived `Debug`: every field, in declaration order.
#[derive(Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// A type where the derived format would leak something, so `Debug` is written
/// by hand with the `debug_` helpers.
pub struct Password(String);

impl Password {
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// The secret is usable, just not printable: checking a candidate reads the
    /// field that `Debug` deliberately does not.
    pub fn matches(&self, candidate: &str) -> bool {
        self.0 == candidate
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Password").field(&"***").finish()
    }
}

/// > First among these standard traits is the `Debug` trait. Nearly every type
/// > can, and should, implement `Debug` , even if it only prints the type's name.
///
/// > Using `# [derive(Debug)]` is often the best way to implement the `Debug`
/// > trait in your interface
pub fn debug_is_the_first_trait_to_implement() {
    let point = Point { x: 1, y: 2 };

    // The derived format, and the reason `assert_eq!` can report a failure.
    assert_eq!(format!("{point:?}"), "Point { x: 1, y: 2 }");
    assert_eq!(point, Point { x: 1, y: 2 });

    // A hand-written impl prints what it chooses, not what it holds.
    let password = Password::new("hunter2");
    assert_eq!(format!("{password:?}"), "Password(\"***\")");
    assert!(!format!("{password:?}").contains("hunter2"));
    assert!(password.matches("hunter2"));
    assert!(!password.matches("hunter3"));
}

// ---------------------------------------------------------------------------
// Debug without the derive's bound
// ---------------------------------------------------------------------------

/// A type whose `Debug` output does not depend on `T`, so it must not require
/// `T: Debug`.
pub struct Opaque<T> {
    pub value: T,
}

impl<T> fmt::Debug for Opaque<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Opaque(..)")
    }
}

/// Deliberately implements nothing: not `Debug`, not `Clone`.
pub struct NotDebug;

/// > keep in mind that all derived traits automatically add the same bound for
/// > any generic parameters.
///
/// The derived version cannot print a `Wrapper<NotDebug>`:
///
/// ```compile_fail,E0277
/// #[derive(Debug)]
/// struct Wrapper<T>(T);
///
/// struct NotDebug;
///
/// let wrapped = Wrapper(NotDebug);
/// println!("{wrapped:?}");   // `NotDebug` doesn't implement `Debug`
/// ```
///
/// The hand-written version can, because it never asks the field to print.
pub fn debug_can_be_written_by_hand_to_avoid_the_derive_bound() {
    let opaque = Opaque { value: NotDebug };

    assert_eq!(format!("{opaque:?}"), "Opaque(..)");

    // The value is still there — it is the *formatting* that does not need it.
    let _: NotDebug = opaque.value;
}

// ---------------------------------------------------------------------------
// Send, Sync, Unpin
// ---------------------------------------------------------------------------

/// An ordinary data type: no interior mutability, no non-atomic reference count.
#[derive(Debug)]
pub struct Record {
    pub id: u64,
    pub label: String,
}

/// The documented exception the book asks for: "If your types cannot implement
/// them, make sure that fact, and the reason why, is well documented!"
///
/// `Rc` has a non-atomic reference count, so sharing one between threads could
/// corrupt the count; and a `&NotThreadSafe` would hand out that `Rc`.
#[derive(Debug)]
pub struct NotThreadSafe {
    pub shared: Rc<u64>,
}

/// > Tied in close second are the Rust auto-traits `Send` and `Sync` (and, to a
/// > lesser extent, `Unpin` ).
///
/// `Send` is "may be moved to another thread", `Sync` is "`&Self` may be shared
/// with another thread", and both are inferred from the fields. `Unpin` says
/// the value can be moved after being pinned, which is the default for every
/// type that does not opt out.
pub fn send_sync_and_unpin_hold_for_ordinary_types() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_unpin<T: Unpin>() {}

    assert_send::<Record>();
    assert_sync::<Record>();
    assert_unpin::<Record>();

    assert_send::<Box<Record>>();
    assert_sync::<std::sync::Arc<Record>>();
    assert_unpin::<Vec<Record>>();
}

/// The same assertions applied to [`NotThreadSafe`] are what the user hits when
/// the opt-out was not documented:
///
/// ```compile_fail,E0277
/// use rust_for_rustaceans::designing_interfaces::unsurprising::common_traits::NotThreadSafe;
///
/// fn assert_send<T: Send>() {}
/// assert_send::<NotThreadSafe>();   // `Rc<u64>` cannot be sent between
///                                   // threads safely
/// ```
///
/// ```compile_fail,E0277
/// use rust_for_rustaceans::designing_interfaces::unsurprising::common_traits::NotThreadSafe;
///
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<NotThreadSafe>();   // `Rc<u64>` cannot be shared between
///                                   // threads safely
/// ```
///
/// `Unpin` is opted out of just as deliberately, with `PhantomPinned`:
///
/// ```compile_fail,E0277
/// fn assert_unpin<T: Unpin>() {}
/// assert_unpin::<std::marker::PhantomPinned>();   // `PhantomPinned` cannot be
///                                                 // unpinned
/// ```
pub fn the_auto_traits_can_be_opted_out_but_document_it() {
    let record = Record {
        id: 1,
        label: "first".to_string(),
    };
    let not_thread_safe = NotThreadSafe { shared: Rc::new(7) };

    // The opt-out is a property of the fields, not an annotation.
    assert_eq!(*not_thread_safe.shared, 7);
    assert_eq!(record.label, "first");
}

// ---------------------------------------------------------------------------
// Clone and Default
// ---------------------------------------------------------------------------

/// Both traits derived: the two "nearly universal" ones.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Buffer {
    pub bytes: Vec<u8>,
    pub read_only: bool,
}

/// > The next set of nearly universal traits you should implement is `Clone` and
/// > `Default` . These traits can be derived or implemented easily and make sense
/// > to implement for most types.
pub fn clone_and_default_are_worth_providing() {
    let empty = Buffer::default();
    assert!(empty.bytes.is_empty());
    assert!(!empty.read_only);

    let mut copy = empty.clone();
    copy.bytes.push(1);

    // The clone owns its own `Vec`: mutating it cannot be seen through the
    // original. That independence is what `Clone` promises.
    assert_ne!(copy, empty);
    assert_eq!(empty.bytes.len(), 0);
    assert_eq!(copy.bytes, vec![1]);
}

// ---------------------------------------------------------------------------
// The comparison traits
// ---------------------------------------------------------------------------

/// `PartialEq` and `Eq`, `PartialOrd` and `Ord`, and `Hash` derived together.
/// Deriving `Eq`/`Ord` is only correct because the fields (`u16`) satisfy those
/// stronger semantics.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

/// > One step further down in the hierarchy of expected traits is the comparison
/// > traits: `PartialEq` , `PartialOrd` , `Hash` , `Eq` , and `Ord` .
///
/// > `Eq` and `Ord` come with additional semantic requirements … you should
/// > implement them _only_ if you're sure those semantics actually apply to your
/// > type.
///
/// `f64` is the standard counter-example: `NaN` is not equal to itself, so
/// `PartialEq` holds and `Eq` cannot.
///
/// ```compile_fail,E0277
/// fn needs_ord<T: Ord>() {}
///
/// needs_ord::<f64>();   // the trait bound `f64: Ord` is not satisfied
/// ```
///
/// A map key needs `Eq` *and* `Hash`, not just `PartialEq`:
///
/// ```compile_fail,E0277
/// use std::collections::HashMap;
///
/// #[derive(PartialEq)]
/// struct Key(u8);
///
/// let mut map = HashMap::new();
/// map.insert(Key(1), 1u8);   // `Key: Eq` and `Key: Hash` are not satisfied
/// ```
pub fn comparison_traits_make_types_usable_as_keys() {
    // Hash + Eq: deduplication in a set.
    let mut seen: HashSet<Version> = HashSet::new();
    assert!(seen.insert(Version {
        major: 1,
        minor: 2,
        patch: 3
    }));
    assert!(!seen.insert(Version {
        major: 1,
        minor: 2,
        patch: 3
    }));
    assert_eq!(seen.len(), 1);

    // Eq + Hash: a map key.
    let mut counts: HashMap<Version, u32> = HashMap::new();
    for _ in 0..2 {
        *counts
            .entry(Version {
                major: 1,
                minor: 2,
                patch: 3,
            })
            .or_insert(0) += 1;
    }
    assert_eq!(
        counts.get(&Version {
            major: 1,
            minor: 2,
            patch: 3
        }),
        Some(&2)
    );

    // Ord: deterministic order, both in a `BTreeSet` and after `sort`.
    let mut ordered: BTreeSet<Version> = BTreeSet::new();
    ordered.insert(Version {
        major: 2,
        minor: 0,
        patch: 0,
    });
    ordered.insert(Version {
        major: 1,
        minor: 9,
        patch: 9,
    });
    assert_eq!(
        ordered.iter().next(),
        Some(&Version {
            major: 1,
            minor: 9,
            patch: 9
        })
    );

    let mut versions = [
        Version {
            major: 2,
            minor: 0,
            patch: 0,
        },
        Version {
            major: 1,
            minor: 10,
            patch: 0,
        },
        Version {
            major: 1,
            minor: 9,
            patch: 9,
        },
    ];
    versions.sort();
    assert_eq!(versions[0].minor, 9); // 1.9.9 before 1.10.0
    assert_eq!(versions[2].major, 2); // 2.0.0 last
}

// ---------------------------------------------------------------------------
// Copy: the trait the book leaves out
// ---------------------------------------------------------------------------

/// `Copy` for a type whose fields are all `Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Meters(pub u32);

/// The same type after one field stops being `Copy` — the change the book warns
/// about, made visible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labeled {
    pub meters: Meters,
    pub label: String,
}

/// > users do not generally expect types to be `Copy`; quite to the contrary,
/// > they tend to expect that if they want two copies of something, they have to
/// > call `clone` . `Copy` changes the semantics of moving a value of the given
/// > type, which might surprise the user.
///
/// The surprise is observable: after `consume(distance)`, `distance` is still
/// usable — the move copied instead of moving.
pub fn copy_is_a_semantic_commitment_not_an_optimisation() {
    fn consume(meters: Meters) -> u32 {
        meters.0
    }

    let distance = Meters(100);
    assert_eq!(consume(distance), 100);
    assert_eq!(distance, Meters(100));

    // Without `Copy`, the same call needs an explicit `clone`.
    let labeled = Labeled {
        meters: Meters(100),
        label: "hundred".to_string(),
    };
    fn describe(labeled: Labeled) -> String {
        labeled.label
    }
    assert_eq!(describe(labeled.clone()), "hundred");
    assert_eq!(labeled.label, "hundred");
}

/// And the commitment is easy to break: the moment a field is not `Copy`, the
/// derive fails. Removing `Copy` afterwards is a breaking change, while removing
/// `Clone` is not.
///
/// ```compile_fail,E0204
/// #[derive(Clone, Copy)]
/// struct Meters(u32);
///
/// #[derive(Clone, Copy)]
/// struct Labeled {
///     meters: Meters,
///     label: String,   // the field type must implement `Copy`
/// }
/// ```
pub fn copy_requires_every_field_to_be_copy() {
    // The `Copy` type is trivially copyable field by field…
    assert_eq!(Meters(1).0, 1);

    // …and the non-`Copy` one only clones, which is why `Clone` is the safe
    // promise to make eagerly and `Copy` is the one to make deliberately.
    let labeled = Labeled {
        meters: Meters(2),
        label: "two".to_string(),
    };
    assert_eq!(labeled.clone().label, "two");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn debug_is_the_first_trait_to_implement_runs() {
        debug_is_the_first_trait_to_implement();
    }

    #[test]
    fn debug_can_be_written_by_hand_to_avoid_the_derive_bound_runs() {
        debug_can_be_written_by_hand_to_avoid_the_derive_bound();
    }

    #[test]
    fn send_sync_and_unpin_hold_for_ordinary_types_runs() {
        send_sync_and_unpin_hold_for_ordinary_types();
    }

    #[test]
    fn the_auto_traits_can_be_opted_out_but_document_it_runs() {
        the_auto_traits_can_be_opted_out_but_document_it();
    }

    #[test]
    fn clone_and_default_are_worth_providing_runs() {
        clone_and_default_are_worth_providing();
    }

    #[test]
    fn comparison_traits_make_types_usable_as_keys_runs() {
        comparison_traits_make_types_usable_as_keys();
    }

    #[test]
    fn copy_is_a_semantic_commitment_not_an_optimisation_runs() {
        copy_is_a_semantic_commitment_not_an_optimisation();
    }

    #[test]
    fn copy_requires_every_field_to_be_copy_runs() {
        copy_requires_every_field_to_be_copy();
    }
}
