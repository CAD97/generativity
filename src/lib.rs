#![no_std]

//! Create a trusted carrier with a new lifetime that is guaranteed to be
//! unique among other trusted carriers. When you call [`make_guard!`] to make a
//! unique lifetime, the macro creates a [`Guard`] to hold it. This guard can be
//! converted `into` an [`Id`], which can be stored in structures to uniquely
//! "brand" them. A different invocation of the macro will produce a new
//! lifetime that cannot be unified. These types have no safe way to construct
//! them other than via [`make_guard!`] or `unsafe` code.
//!
//! ```rust
//! use generativity::{Id, make_guard};
//! struct Struct<'id>(Id<'id>);
//! make_guard!(a);
//! Struct(a.into());
//! ```
//!
//! This is the concept of "generative" lifetime brands. `Guard` and `Id` are
//! [invariant](https://doc.rust-lang.org/nomicon/subtyping.html#variance) over
//! their lifetime parameter, meaning that it is never valid to substitute or
//! otherwise coerce `Id<'a>` into `Id<'b>`, for *any* `'a` or `'b`, *including*
//! the `'static` lifetime.
//!
//! Any invariant lifetime can be "trusted" to cary a brand, but when using this
//! library, it is recommended to always use `Id<'id>` to carry the brand, as
//! this reduces the risk of accidentally trusting an untrusted lifetime.
//! Importantly, non-invariant lifetimes *cannot* be trusted, as the variance
//! allows lifetimes to be contracted to match and copy the brand lifetime.

use core_::{fmt, marker::PhantomData};

#[doc(hidden)]
pub extern crate core as core_;

/// A phantomdata-like type taking a single invariant lifetime.
///
/// Used to manipulate and store the unique invariant lifetime obtained from
/// [`Guard`]. Use `guard.into()` to create a new `Id`.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id<'id> {
    phantom: PhantomData<fn(&'id ()) -> &'id ()>,
}

impl<'id> Id<'id> {
    /// Construct an `Id` with an unbound lifetime.
    ///
    /// You should not need to use this function; use [`make_guard!`] instead.
    ///
    /// # Safety
    ///
    /// This creates an unbound invariant lifetime that people are allowed to
    /// assume means it was derived from their generative brand. This is the
    /// "I know what I'm doing" button; restrict the lifetime to a known brand
    /// immediately to avoid accidentally introducing unsoundness potential.
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
    fn from(guard: Guard<'id>) -> Self {
        guard.id
    }
}

/// An invariant lifetime phantomdata that is guaranteed to be unique with
/// respect to other invariant lifetimes.
///
/// In effect, this means that `'id` is a "generative brand". Use [`make_guard`]
/// to obtain a new `Guard`.
#[derive(Eq, PartialEq)]
pub struct Guard<'id> {
    #[allow(unused)]
    id: Id<'id>,
}

impl<'id> Guard<'id> {
    /// Construct a `Guard` with an unbound lifetime.
    ///
    /// You should not need to use this function; use [`make_guard!`] instead.
    ///
    /// # Safety
    ///
    /// This creates an unbound invariant lifetime that people are allowed to
    /// assume means it was derived from their generative brand. This is the
    /// "I know what I'm doing" button; restrict the lifetime to a known brand
    /// immediately to avoid accidentally introducing unsoundness potential.
    pub unsafe fn new(id: Id<'id>) -> Guard<'id> {
        Guard { id }
    }
}

impl<'id> fmt::Debug for Guard<'id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("#[unique] 'id").finish()
    }
}

/// Create a `Guard` with a unique invariant lifetime (with respect to other
/// trusted/invariant lifetime brands).
///
/// Multiple `make_guard` lifetimes will always fail to unify:
///
/// ```rust,compile_fail,E0597
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
            #[allow(non_camel_case_types)]
            struct make_guard<'id> {
                _id: $crate::Id<'id>,
            }
            impl<'id> $crate::core_::ops::Drop for make_guard<'id> {
                #[inline(always)]
                fn drop(&mut self) {}
            }
            fn make_guard<'id>(id: &'id $crate::Id<'id>) -> make_guard<'id> {
                make_guard { _id: *id }
            }
            make_guard(&tag)
        };
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use core_::panic::{RefUnwindSafe, UnwindSafe};

    #[test]
    fn dont_error_in_general() {
        make_guard!(a);
        make_guard!(b);
        assert_eq!(a, a);
        assert_eq!(b, b);
    }

    #[test]
    fn test_oibits() {
        fn assert_oibits<T>(_: &T)
        where
            T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
        {
        }

        make_guard!(a);
        assert_oibits(&a);
        let id: Id<'_> = a.into();
        assert_oibits(&id);
    }
}
