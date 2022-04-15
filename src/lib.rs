#![cfg_attr(not(test), no_std)]
//! Create a trusted carrier with a new lifetime that is guaranteed to be
//! unique. When you call [`make_guard!`] to make a unique lifetime, the macro
//! creates a [`Guard`] to hold it. This guard can be converted `into` an
//! [`Id`], which can be stored in structures to uniquely "brand" them. A different
//! invocation of the macro will produce a new lifetime that cannot be unified.
//! These types have no safe way to construct them other than via
//! [`make_guard!`].
//!
//! ```rust
//! use generativity::{Id, make_guard};
//! struct Struct<'id>(Id<'id>);
//! make_guard!(a);
//! Struct(a.into());
//! ```

use core::{fmt, marker::PhantomData};

/// A phantomdata-like type taking a single invariant lifetime.
///
/// Used to manipulate and store the unique invariant lifetime obtained from
/// [`Guard`]. Use `guard.into()` to create a new `Id`.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id<'id> {
    phantom: PhantomData<fn(&'id ()) -> &'id ()>,
}

impl<'id> Id<'id> {
    // Do not use this function; use the `make_guard!` macro instead.
    #[doc(hidden)]
    pub unsafe fn new() -> Self {
        Id {
            phantom: PhantomData,
        }
    }
}

impl<'id> fmt::Debug for Id<'id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("#[invariant] 'id").finish()
    }
}

impl<'id> From<Guard<'id>> for Id<'id> {
    fn from(_guard: Guard<'id>) -> Self {
        Id {
            phantom: PhantomData,
        }
    }
}

/// An invariant lifetime phantomdata that is guaranteed to be unique.
///
/// In effect, this means that `'id` is a "generative brand". Use [`make_guard`]
/// to obtain a new `Guard`.
#[derive(Eq, PartialEq)]
pub struct Guard<'id> {
    #[allow(unused)]
    id: Id<'id>,
}

impl<'id> Guard<'id> {
    // Do not use this function; use the `guard!` macro instead.
    #[doc(hidden)]
    pub unsafe fn new(id: Id<'id>) -> Guard<'id> {
        Guard { id }
    }
}

impl<'id> fmt::Debug for Guard<'id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("#[unique] 'id").finish()
    }
}

/// Create a `Guard` with a unique lifetime.
///
/// Multiple invocations will not unify:
///
/// ```rust,compile_fail
/// # use generativity::make_guard;
/// make_guard!(a);
/// make_guard!(b);
/// dbg!(a == b); // ERROR (here == is a static check)
/// ```
#[macro_export]
macro_rules! make_guard {
    ($name:ident) => {
        let tag = unsafe { $crate::Id::new() };
        let $name = unsafe { $crate::Guard::new(tag) };
        let _guard = {
            // FUTURE(optimization): make `make_guard` a ZST with `PhantomData`
            // Restrict `'id` with `fn new(&'id Id<'id>) -> make_guard<'id>`?
            #[allow(non_camel_case_types)]
            struct make_guard<'id>(&'id $crate::Id<'id>);
            impl<'id> ::core::ops::Drop for make_guard<'id> {
                #[inline(always)]
                fn drop(&mut self) {}
            }
            make_guard(&tag)
        };
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[allow(clippy::eq_op)]
    fn dont_error_in_general() {
        make_guard!(a);
        make_guard!(b);
        dbg!(a == a);
        dbg!(b == b); // OK
    }

    #[test]
    fn is_unwind_safe() {
        make_guard!(a);
        struct Wrapper<'id>(Id<'id>);
        let x = Wrapper(a.into());
        std::panic::catch_unwind(|| {
            let _x = x;
        })
        .unwrap();
    }
}
