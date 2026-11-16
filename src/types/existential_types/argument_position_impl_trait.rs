//! # `impl Trait` in Argument Position
//!
//! > Not all instances of `impl Trait` use existential types. If you use
//! > `impl Trait` in argument position for a function, it's really just shorthand
//! > for an unnamed generic parameter to that function. For example,
//! > `fn foo(s: impl ToString)` is mostly just syntax sugar for
//! > `fn foo<S: ToString>(s: S)`.
//!
//! "Unnamed" is the operative word: the parameter is a real generic parameter,
//! but there is no identifier to write in a turbofish. Everything else follows
//! from that — each `impl Trait` argument is its own parameter, and the function
//! cannot be called with an explicit type argument.

/// Argument-position `impl Trait`, two independent parameters.
pub fn show_anything(first: impl ToString, second: impl ToString) -> (String, String) {
    (first.to_string(), second.to_string())
}

/// The same function with named parameters. This is what the first desugars to,
/// except that the names are not callable.
pub fn show_same_type<T: ToString>(first: T, second: T) -> (String, String) {
    (first.to_string(), second.to_string())
}

/// > it's really just shorthand for an unnamed generic parameter
///
/// When the two arguments have the same type, the two spellings are
/// indistinguishable from the call site.
pub fn the_two_spellings_agree_when_the_types_are_the_same() {
    assert_eq!(show_anything(1u8, 2u8), show_same_type(1u8, 2u8));
    assert_eq!(show_anything("a", "b"), show_same_type("a", "b"));
}

/// > shorthand for an unnamed generic parameter to that function
///
/// One parameter per `impl Trait`, so two different types are fine in the first
/// form — and rejected in the second, where both arguments must be the same `T`:
///
/// ```compile_fail,E0308
/// fn show_same_type<T: ToString>(first: T, second: T) -> (String, String) {
///     (first.to_string(), second.to_string())
/// }
///
/// let pair = show_same_type(1u8, "x");   // mismatched types: expected `u8`,
///                                        // found `&str`
/// ```
pub fn each_impl_trait_argument_is_its_own_type_parameter() {
    let pair = show_anything(1u8, "x");

    assert_eq!(pair, ("1".to_string(), "x".to_string()));

    // Two parameters, two monomorphizations: integer and `&str`.
    assert_eq!(show_anything(1u8, 2u8), ("1".to_string(), "2".to_string()));
}

/// > it's really just shorthand for an *unnamed* generic parameter
///
/// The unnamed part is observable: the function takes no type arguments, so a
/// turbofish — legal for `show_same_type` — is not.
///
/// ```compile_fail,E0107
/// fn show_anything(first: impl ToString, second: impl ToString) -> (String, String) {
///     (first.to_string(), second.to_string())
/// }
///
/// let pair = show_anything::<u8>(1u8, 2u8);   // function takes 0 generic
///                                             // arguments but 1 was supplied
/// ```
pub fn an_argument_position_impl_trait_cannot_be_turbofished() {
    // The named version can be given a type argument…
    let named = show_same_type::<u8>(1, 2);
    assert_eq!(named, ("1".to_string(), "2".to_string()));

    // …the `impl Trait` version cannot; the only thing that selects its
    // parameter is the argument itself.
    let sugar = show_anything(1u8, 2u8);
    assert_eq!(sugar, named);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_two_spellings_agree_when_the_types_are_the_same_runs() {
        the_two_spellings_agree_when_the_types_are_the_same();
    }

    #[test]
    fn each_impl_trait_argument_is_its_own_type_parameter_runs() {
        each_impl_trait_argument_is_its_own_type_parameter();
    }

    #[test]
    fn an_argument_position_impl_trait_cannot_be_turbofished_runs() {
        an_argument_position_impl_trait_cannot_be_turbofished();
    }
}
