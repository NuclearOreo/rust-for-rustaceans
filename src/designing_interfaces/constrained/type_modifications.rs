//! # Constrained — Type Modifications
//!
//! > Removing or renaming a public type will almost certainly break some user's
//! > code. To counter this, you'll want to take advantage of Rust's visibility
//! > modifiers, like `pub(crate)` and `pub(in path)` , whenever possible. The
//! > fewer public types you have, the more freedom you have to change things
//! > later without breaking existing code.
//!
//! > User code can depend on your types in more ways than just by name, though.
//!
//! Listing 3-3 says an "innocent-looking public type" is a constructor, and
//! Listing 3-4 says an exhaustive pattern is a promise about every field, even
//! the ones the user cannot see:
//!
//! | Change | What it deletes from user code | Code |
//! | --- | --- | --- |
//! | unit struct gains a private field | `lib::Unit` as a value, and `Unit { .. }` literals | `E0423`, `E0451` |
//! | struct gains a private field | patterns that list every visible field | `E0027` |
//! | tuple struct becomes a named struct | tuple patterns, even with identical fields | `E0532` |
//! | `#[non_exhaustive]` is added | literal construction, patterns without `, ..` | `E0639`, `E0638` |
//!
//! Every pair below is the same two versions of one type: the 1.0 shape that the
//! runnable demo still uses, and the 2.0 shape whose doctest is the user program
//! that no longer compiles.

use std::mem::size_of_val;

// ---------------------------------------------------------------------------
// Listing 3-3: a unit struct is a constructor
// ---------------------------------------------------------------------------

/// Listing 3-3, the 1.0 shape: a public unit struct, so the type's own name is
/// the value.
pub struct Unit;

/// Listing 3-3, the 2.0 shape: the same type after one private field was added.
///
/// > Even though the field you add is private, the change will still break the
/// > user's code, because the constructor they relied on has disappeared.
///
/// The old code was `let u = lib::Unit;`. Now the name is a type that needs a
/// constructor, and the private field means no other crate has one:
///
/// ```compile_fail,E0423
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::UnitWithPrivateField;
///
/// // cannot find value `UnitWithPrivateField` in this scope:
/// // the unit-struct constructor is gone
/// let _ = UnitWithPrivateField;
/// ```
///
/// Trying to supply the field by name is refused for the other reason, and that
/// one does carry a code:
///
/// ```compile_fail,E0451
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::UnitWithPrivateField;
///
/// // field `_private` of struct `UnitWithPrivateField` is private
/// let _ = UnitWithPrivateField { _private: () };
/// ```
pub struct UnitWithPrivateField {
    _private: (),
}

/// The 1.0 half: `Unit` is a value, its size is zero, and nothing about its
/// construction depends on a field.
pub fn listing_3_3_a_unit_struct_is_a_constructor() {
    let unit = Unit;

    assert_eq!(size_of_val(&unit), 0);

    // The 2.0 shape has no such expression. What it does have is a size, because
    // the private field exists even if the user cannot name it.
    assert_eq!(
        size_of_val(&UnitWithPrivateField { _private: () }),
        size_of_val(&())
    );
}

// ---------------------------------------------------------------------------
// Listing 3-4: exhaustive patterns see private fields
// ---------------------------------------------------------------------------

/// Listing 3-4, the 1.0 shape: one public field, so a pattern naming it is
/// exhaustive.
pub struct OpenFlags {
    pub field: bool,
}

/// Listing 3-4, the 2.0 shape: the same public field, plus a private one.
///
/// > Rust's exhaustive pattern match checking logic is able to see parts of the
/// > interface that the user cannot see. It recognizes that there are more
/// > fields, even though the user code cannot access them, and rejects the user's
/// > pattern as incomplete.
///
/// ```compile_fail,E0027
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::Flags;
///
/// // pattern does not mention field `_private`
/// fn is_true(flags: Flags) -> bool {
///     matches!(flags, Flags { field: true })
/// }
/// ```
///
/// The fix on the user's side is `, ..` — the same trailing dots
/// `#[non_exhaustive]` would have demanded up front:
///
/// ```
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::Flags;
///
/// fn is_true(flags: Flags) -> bool {
///     matches!(flags, Flags { field: true, .. })
/// }
/// # let _ = is_true;
/// ```
// The 2.0 shape is Listing 3-4's private field, not the attribute — that is the
// point: the break happens even before `#[non_exhaustive]` is considered.
#[allow(clippy::manual_non_exhaustive)]
pub struct Flags {
    pub field: bool,
    _private: (),
}

impl Flags {
    pub fn new(field: bool) -> Self {
        Self {
            field,
            _private: (),
        }
    }
}

/// The 1.0 half: with every field public, the short pattern *is* exhaustive, so
/// nothing is silently reserved.
pub fn listing_3_4_an_exhaustive_pattern_names_every_public_field() {
    let open = OpenFlags { field: true };

    assert!(matches!(open, OpenFlags { field: true }));

    // And once the private field exists, the same pattern needs `..`.
    let closed = Flags::new(true);
    assert!(matches!(closed, Flags { field: true, .. }));
}

// ---------------------------------------------------------------------------
// A tuple struct and a named struct take different patterns
// ---------------------------------------------------------------------------

/// The 1.0 shape: positional fields.
pub struct TuplePoint(pub u8, pub u8);

/// The 2.0 shape: identical fields, different syntax to reach them.
///
/// > A similar issue arises if we turn a tuple struct into a regular struct with
/// > named fields: even if the fields themselves are exactly the same, any old
/// > patterns will no longer be valid for the new type definition.
///
/// ```compile_fail,E0532
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::NamedPoint;
///
/// // cannot find tuple struct or tuple variant `NamedPoint`
/// fn split(point: NamedPoint) -> (u8, u8) {
///     let NamedPoint(x, y) = point;
///     (x, y)
/// }
/// ```
pub struct NamedPoint {
    pub x: u8,
    pub y: u8,
}

/// The two pattern syntaxes, each against the shape that accepts it — which is
/// the point: the field values are interchangeable, the patterns are not.
pub fn a_tuple_struct_and_a_named_struct_take_different_patterns() {
    let TuplePoint(x, y) = TuplePoint(3, 4);
    assert_eq!((x, y), (3, 4));

    let NamedPoint { x, y } = NamedPoint { x: 3, y: 4 };
    assert_eq!((x, y), (3, 4));
}

// ---------------------------------------------------------------------------
// #[non_exhaustive]: pay up front for the right to add fields
// ---------------------------------------------------------------------------

/// > Rust provides the `#[non_exhaustive]` attribute to help mitigate these
/// > issues. You can add it to any type definition, and the compiler will
/// > disallow the use of implicit constructors (like `lib::Unit { field1: true }`)
/// > and nonexhaustive pattern matches (that is, patterns without a trailing `,
/// > ..`) on that type.
///
/// From outside the crate, the literal form is refused:
///
/// ```compile_fail,E0639
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::Settings;
///
/// // cannot create non-exhaustive struct using struct expression
/// let _ = Settings { verbose: true };
/// ```
///
/// And so is a pattern that names every visible field:
///
/// ```compile_fail,E0638
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::Settings;
///
/// // `..` required with struct marked as non-exhaustive
/// fn is_verbose(settings: Settings) -> bool {
///     matches!(settings, Settings { verbose: true })
/// }
/// ```
///
/// > It does constrain user code though, such as by taking away users' ability to
/// > rely on exhaustive pattern matches, so avoid adding it if you think a given
/// > type is likely to remain stable.
#[non_exhaustive]
pub struct Settings {
    pub verbose: bool,
}

/// Only the *defining* crate is exempt from `#[non_exhaustive]`: it can still
/// write the literal and the short pattern that every user is refused. That
/// asymmetry is the whole mechanism — the reserved right belongs to the author.
pub fn non_exhaustive_reserves_the_right_to_add_a_field() {
    let settings = Settings { verbose: true };

    assert!(matches!(settings, Settings { verbose: true }));
}

// ---------------------------------------------------------------------------
// pub(crate): the fewer public types, the more freedom
// ---------------------------------------------------------------------------

/// > The fewer public types you have, the more freedom you have to change things
/// > later without breaking existing code.
///
/// A `pub(crate)` item is invisible to users, so renaming or deleting it can
/// never break them. From outside the crate there is nothing to import:
///
/// ```compile_fail,E0603
/// // function `internal_helper` is private
/// use rust_for_rustaceans::designing_interfaces::constrained::type_modifications::internal_helper;
///
/// let _ = internal_helper();
/// ```
///
/// The demo below is the in-crate half: the same item, fully usable, and therefore
/// still needing care inside the crate — just not across a version boundary.
pub fn pub_crate_items_are_not_part_of_the_contract() {
    assert_eq!(internal_helper(), 7);
}

pub(crate) fn internal_helper() -> u8 {
    7
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listing_3_3_a_unit_struct_is_a_constructor_runs() {
        listing_3_3_a_unit_struct_is_a_constructor();
    }

    #[test]
    fn listing_3_4_an_exhaustive_pattern_names_every_public_field_runs() {
        listing_3_4_an_exhaustive_pattern_names_every_public_field();
    }

    #[test]
    fn a_tuple_struct_and_a_named_struct_take_different_patterns_runs() {
        a_tuple_struct_and_a_named_struct_take_different_patterns();
    }

    #[test]
    fn non_exhaustive_reserves_the_right_to_add_a_field_runs() {
        non_exhaustive_reserves_the_right_to_add_a_field();
    }

    #[test]
    fn pub_crate_items_are_not_part_of_the_contract_runs() {
        pub_crate_items_are_not_part_of_the_contract();
    }
}
