#![no_std]
#![doc = include_str!("../README.md")]

//  NB: trybuild tests (error messages) include line numbers from this macro,
//      so doc changes will require re-blessing the tests, unfortunately.
#[macro_export]
macro_rules! guard {
    // make a local binding guard (statement macro)
    {let $($name:pat),+ $(,)?} => {
        $(
            let id = unsafe { $crate::Id::new() };
            let brand = unsafe { $crate::Brand::tie(&id) };
            let $name = unsafe { $crate::Guard::tie(&brand) };
        )*
    };
    // make a temporary guard (expression macro)
    () => {
        // SAFETY: oh god, what have I done
        unsafe { $crate::Guard::tie(&$crate::Brand::tie(&$crate::Id::new())) }
    };
}

// ! Create a trusted carrier with a new lifetime that is guaranteed to be
// ! unique among other trusted carriers. When you call [`make_guard!`] to make a
// ! unique lifetime, the macro creates a [`Guard`] to hold it. This guard can be
// ! converted `into` an [`Id`], which can be stored in structures to uniquely
// ! "brand" them. A different invocation of the macro will produce a new
// ! lifetime that cannot be unified. The only way to construct these types is
// ! with [`make_guard!`] or `unsafe` code.
// !
// ! ```rust
// ! use generativity::{Id, make_guard};
// ! struct Struct<'id>(Id<'id>);
// ! make_guard!(a);
// ! Struct(a.into());
// ! ```
// !
// ! This is the concept of "generative" lifetime brands. `Guard` and `Id` are
// ! [invariant](https://doc.rust-lang.org/nomicon/subtyping.html#variance) over
// ! their lifetime parameter, meaning that it is never valid to substitute or
// ! otherwise coerce `Id<'a>` into `Id<'b>`, for *any* concrete `'a` or `'b`,
// ! *including* the `'static` lifetime.
// !
// ! Any invariant lifetime can be "trusted" to carry a brand, so long as they
// ! are known to be restricted to carrying a brand, and haven't been derived
// ! from some untrusted lifetime (or are completely unbound). When using this
// ! library, it is recommended to always use `Id<'id>` to carry the brand, as
// ! this reduces the risk of accidentally trusting an untrusted lifetime.
// ! Importantly, non-invariant lifetimes *cannot* be trusted, as the variance
// ! allows lifetimes to be contracted to match and copy the brand lifetime.

use core::fmt;
use core::marker::PhantomData;

#[doc(hidden)]
pub extern crate core;

/// A phantomdata-like type taking a single invariant lifetime.
///
/// Used to manipulate and store the unique invariant lifetime obtained from
/// [`Guard`]. Use `guard.into()` to create a new `Id`.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id<'id> {
    marker: PhantomData<fn(&'id ()) -> &'id ()>,
}

#[doc(hidden)]
impl<'id> Id<'id> {
    /// Construct an `Id` with an unbounded lifetime.
    ///
    /// You should not need to use this function; use [`make_guard!`] instead.
    ///
    /// # Safety
    ///
    /// `Id` holds an invariant lifetime that must be derived from a generative
    /// brand. Using this function directly is the "I know what I'm doing"
    /// button; restrict the lifetime to a known brand immediately to avoid
    /// introducing unsoundness.
    pub unsafe fn new() -> Self {
        Id {
            marker: PhantomData,
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

#[doc(hidden)]
impl<'id> Guard<'id> {
    /// Construct a `Guard` with an unbound lifetime.
    ///
    /// You should not need to use this function; use [`make_guard!`] instead.
    ///
    /// # Safety
    ///
    /// `Guard` holds an invariant lifetime that must be an unused generative
    /// brand. Using this function directly is the "I know what I'm doing"
    /// button; restrict the lifetime to a known brand immediately to avoid
    /// introducing unsoundness.
    pub unsafe fn new(id: Id<'id>) -> Guard<'id> {
        Guard { id }
    }

    pub unsafe fn tie(brand: &Brand<'id>) -> Guard<'id> {
        Guard::new(brand.id)
    }
}

impl<'id> fmt::Debug for Guard<'id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("#[unique] 'id").finish()
    }
}

#[doc(hidden)]
pub struct Brand<'id> {
    id: Id<'id>,
}

impl Drop for Brand<'_> {
    #[inline(always)]
    fn drop(&mut self) {}
}

#[doc(hidden)]
impl<'id> Brand<'id> {
    pub unsafe fn tie(&id: &'id Id<'id>) -> Self {
        Brand { id }
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
        $crate::guard!(let $name);
    };
}

// fn test_let() {
//     let a = guard!();
//     drop(a);
// }

#[cfg(test)]
mod test {
    use super::*;
    use core::panic::{RefUnwindSafe, UnwindSafe};

    #[test]
    fn statement_guard_is_usable() {
        guard!(let a);
        guard!(let b);
        assert_eq!(a, a);
        assert_eq!(b, b);
    }

    #[test]
    fn expression_guard_is_usable() {
        guard!();
    }

    #[test]
    fn test_oibits() {
        fn assert_oibits<T>(_: &T)
        where
            T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
        {
        }

        guard!(let a);
        assert_oibits(&a);
        let id: Id<'_> = a.into();
        assert_oibits(&id);
    }
}
