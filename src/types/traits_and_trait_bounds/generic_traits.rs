//! # Generic Traits
//!
//! > Rust traits can be generic in one of two ways: with generic type parameters
//! > like `trait Foo<T>` or with associated types like `trait Foo { type Bar; }`.
//! > The difference between these is not immediately apparent, but luckily the
//! > rule of thumb is quite simple: use an associated type if you expect only one
//! > implementation of the trait for a given type, and use a generic type
//! > parameter otherwise.
//!
//! > The rationale for this is that associated types are often significantly
//! > easier to work with, but will not allow multiple implementations. So, more
//! > simply put, the advice is really just to use associated types whenever you
//! > can.
//!
//! The two designs are compared on the same problem — a type that can produce
//! values of another type — with the same trade-off appearing every time: the
//! generic version can be implemented repeatedly and needs disambiguating at
//! every call; the associated version can be implemented once and never needs it.

use std::fmt::Debug;

// ---------------------------------------------------------------------------
// The generic-parameter design
// ---------------------------------------------------------------------------

/// The generic form. `T` is an *input* to the trait: it is part of what selects
/// an implementation, so one type may have many.
pub trait Convert<T> {
    fn convert(&self) -> T;
}

/// One type, two conversions, both `impl Convert<…> for Doubler`.
pub struct Doubler;

impl Convert<u8> for Doubler {
    fn convert(&self) -> u8 {
        2 // "doubling" is not the point; having two impls is
    }
}

impl Convert<u16> for Doubler {
    fn convert(&self) -> u16 {
        4
    }
}

/// > But the upside is that you can implement the trait multiple times for the
/// > same type — for example, you can implement `PartialEq` against multiple
/// > right-hand side types for your type.
pub fn a_generic_parameter_allows_multiple_implementations() {
    let doubler = Doubler;

    assert_eq!(<Doubler as Convert<u8>>::convert(&doubler), 2u8);
    assert_eq!(<Doubler as Convert<u16>>::convert(&doubler), 4u16);

    // Both are `From`-like: the same type, two different outputs.
    let _as_byte: u8 = Convert::<u8>::convert(&doubler);
    let _as_word: u16 = Convert::<u16>::convert(&doubler);
}

/// > And since multiple implementations of a trait may exist for a given type,
/// > the compiler may have a hard time deciding which instance of the trait you
/// > meant to use, leading to awful disambiguating function calls like
/// > `FromIterator::<u32>::from_iter`.
///
/// ```compile_fail,E0283
/// trait Convert<T> {
///     fn convert(&self) -> T;
/// }
///
/// struct Doubler;
/// impl Convert<u8> for Doubler { fn convert(&self) -> u8 { 2 } }
/// impl Convert<u16> for Doubler { fn convert(&self) -> u16 { 4 } }
///
/// let value = Doubler.convert();   // type annotations needed: two candidates
/// ```
pub fn generic_parameters_force_disambiguation() {
    let doubler = Doubler;

    // An expected type settles it…
    let inferred: u8 = doubler.convert();
    assert_eq!(inferred, 2);

    // …but otherwise the call has to name the implementation.
    let turbofish: u16 = Convert::<u16>::convert(&doubler);
    let qualified: u16 = <Doubler as Convert<u16>>::convert(&doubler);

    assert_eq!(turbofish, qualified);
}

// ---------------------------------------------------------------------------
// The associated-type design
// ---------------------------------------------------------------------------

/// The associated-type form. `Output` is an *output* of the trait: it is
/// dictated by the implementing type, so there is exactly one of each.
pub trait ConvertInto {
    type Output;

    fn convert(&self) -> Self::Output;
}

/// A type with one conversion — the case associated types are for.
pub struct Identity;

impl ConvertInto for Identity {
    type Output = u8;

    fn convert(&self) -> u8 {
        7
    }
}

/// > With associated types, on the other hand, the compiler needs to know only
/// > the type that implements the trait, and all the associated types follow
/// > (since there is only one implementation). … you never have to disambiguate
/// > with the unified function calling syntax.
pub fn an_associated_type_allows_only_one_implementation() {
    let identity = Identity;

    // No turbofish, no `<Type as Trait>::`, no expected type needed.
    assert_eq!(identity.convert(), 7u8);

    // The associated type is nameable, and it is fixed.
    let explicit: <Identity as ConvertInto>::Output = identity.convert();
    assert_eq!(explicit, 7u8);
}

/// > However, you cannot implement `Deref` against multiple `Target` types, nor
/// > can you implement `Iterator` with multiple different `Item` types.
///
/// ```compile_fail,E0119
/// trait ConvertInto {
///     type Output;
///     fn convert(&self) -> Self::Output;
/// }
///
/// struct Identity;
/// impl ConvertInto for Identity { type Output = u8; fn convert(&self) -> u8 { 7 } }
/// impl ConvertInto for Identity { type Output = u16; fn convert(&self) -> u16 { 7 } }
/// // conflicting implementations of trait `ConvertInto` for type `Identity`
/// ```
pub fn an_associated_type_cannot_be_implemented_twice() {
    // The compile-time guarantee, restated: `Identity` has exactly one `Output`,
    // so a bound of the form `T: ConvertInto<Output = u8>` is decidable.
    fn assert_output_is_u8<T: ConvertInto<Output = u8>>(_value: &T) {}

    assert_output_is_u8(&Identity);
}

// ---------------------------------------------------------------------------
// Naming an associated type that needs disambiguation
// ---------------------------------------------------------------------------

/// A generic trait that *also* has an associated type — the case where the short
/// `Type::AssocType` path stops working.
pub trait Mapper<T> {
    type Output;

    fn map(&self, input: T) -> Self::Output;
}

impl Mapper<u8> for Doubler {
    type Output = u16;

    fn map(&self, input: u8) -> u16 {
        u16::from(input) * 2
    }
}

impl Mapper<u16> for Doubler {
    type Output = u32;

    fn map(&self, input: u16) -> u32 {
        u32::from(input) * 2
    }
}

/// > If a type has multiple associated types by the same name, such as if the
/// > trait that provides the associated type is itself generic (and therefore
/// > there are many implementations), you can disambiguate with the syntax
/// > `<Type as Trait>::AssocType`.
///
/// ```compile_fail,E0223
/// trait Mapper<T> {
///     type Output;
/// }
///
/// struct Doubler;
/// impl Mapper<u8> for Doubler { type Output = u16; }
/// impl Mapper<u16> for Doubler { type Output = u32; }
///
/// let value: Doubler::Output;   // ambiguous associated type
/// ```
pub fn a_generic_trait_with_an_associated_type_needs_fully_qualified_syntax() {
    let doubler = Doubler;

    let byte_result: <Doubler as Mapper<u8>>::Output = doubler.map(3u8);
    let word_result: <Doubler as Mapper<u16>>::Output = doubler.map(3u16);

    assert_eq!(byte_result, 6u16);
    assert_eq!(word_result, 6u32);
}

// ---------------------------------------------------------------------------
// Bounds that live in the trait
// ---------------------------------------------------------------------------

/// > This means the bounds can all live in the trait itself and do not need to
/// > be repeated on use. In turn, this allows the trait to add further associated
/// > types without affecting its users.
pub trait Summary {
    type Item: Debug;

    fn summary(&self) -> Self::Item;
}

impl Summary for Doubler {
    type Item = &'static str;

    fn summary(&self) -> &'static str {
        "doubler"
    }
}

/// `render` names one bound — `T: Summary` — and gets `T::Item: Debug` for free
/// from the trait definition. With a generic parameter for the item type, this
/// function would have to say `T: Summary<Item>, Item: Debug` and its callers
/// would have to name `Item` too.
pub fn bounds_can_live_in_the_trait() {
    fn render<T: Summary>(value: &T) -> String {
        format!("{:?}", value.summary())
    }

    assert_eq!(render(&Doubler), "\"doubler\"");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_generic_parameter_allows_multiple_implementations_runs() {
        a_generic_parameter_allows_multiple_implementations();
    }

    #[test]
    fn generic_parameters_force_disambiguation_runs() {
        generic_parameters_force_disambiguation();
    }

    #[test]
    fn an_associated_type_allows_only_one_implementation_runs() {
        an_associated_type_allows_only_one_implementation();
    }

    #[test]
    fn an_associated_type_cannot_be_implemented_twice_runs() {
        an_associated_type_cannot_be_implemented_twice();
    }

    #[test]
    fn a_generic_trait_with_an_associated_type_needs_fully_qualified_syntax_runs() {
        a_generic_trait_with_an_associated_type_needs_fully_qualified_syntax();
    }

    #[test]
    fn bounds_can_live_in_the_trait_runs() {
        bounds_can_live_in_the_trait();
    }
}
