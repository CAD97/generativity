# Generativity

[![Chat on Discord](https://img.shields.io/badge/-chat-26262b.svg?style=popout&logo=discord)][Discord]
[![Travis status](https://img.shields.io/travis/com/CAD97/generativity.svg?style=popout&logo=travis)][Travis]
[![Coverage on CodeCov](https://img.shields.io/badge/-coverage-fo1f7a.svg?style=popout&logo=Codecov)][Codecov]

Generativity refers to the creation of a unique lifetime: one that the Rust
borrow checker will not unify with any other lifetime. This can be used to
brand types such that you know that you, and not another copy of you, created
them. This is required for sound unchecked indexing and similar tricks.

The first step of achieving generativity is an [invariant][variance] lifetime.
Then the consumer must not be able to unify that lifetime with any other lifetime.

Traditionally, this is achieved with a closure. If you have the user write their code in a
`for<'a> fn(Invariant<'a>) -> _` callback, the local typechecking within this callback
is forced to assume that it can be handed _any_ lifetime (the `for<'a>` bound), and that
it cannot possibly use another lifetime, even `'static`, in its place (the invariance).

This crate implements a different approach using macros and a `Drop`-based scope guard.
When you call `generativity::make_guard!` to make a unique lifetime guard, the macro
first creates an `Id` holding an invariant lifetime. It then puts within a `Drop` type
a `&'id Id<'id>` reference. A different invocation of the macro has a different end timing
to its tag lifetime, so the lifetimes cannot be unified. These types have no safe way to
construct them other than via `make_guard!`, thus the lifetime is guaranteed unique.

This effectively does the same thing as wrapping the rest of the function in an
immediately invoked closure. We do some pre-work (create the tag and give the caller the
guard), run the user code (the code after here, the closure previously), and then run
some cleanup code after (in the drop implementation). This same technique of a macro
hygiene hidden `impl Drop` can be used by any API that would normally use a closure
argument, such as crossbeam's scoped threads, to make the containing scope the safe,
wrapped scope if they so desire the trade-off of macro versus closure indentation.

## Informal proof of correctness

```rust
let tag = unsafe { $crate::Id::new() };
let $name = unsafe { $crate::Guard::new(tag) };
let _guard = {
    #[allow(non_camel_case_types)]
    struct make_guard<'id>(&'id $crate::Id<'id>);
    impl<'id> ::core::ops::Drop for make_guard<'id> {
        #[inline(always)]
        fn drop(&mut self) {}
    }
    make_guard(&tag)
};
```

See also #1 for further discussion.

### Unimportant, nicety details

- New unique type per macro invocation: this is merely to avoid having a type in the public API,
  and such that the compiler emits slightly more useful error messages for lifetime errors.
- `#[inline(always)]`: This is a micro-optimization not required for safety. This makes it easier
  for the optimizer to optimize out the `_guard`'s drop implementation.

### Three places, all required

- `$name` is the user-named `#[unique] 'id` type that we give to the calling context.
- `tag` is a location that we use to define the `'id` lifetime without restricting `$name`.
- `_guard` is an `impl Drop` that we use to restrict `'id`.

### `'id` is unique

- `'id` is _invariant_ due to the invariance of `tag: generativity::Tag<'id>`.
- `$name: generativity::Guard<'id>` has the same `'id` because it is created from `tag`.
- `'id` is restricted by creating `_guard: make_guard(&'id generativity::Tag<'id>)`.
- The end point of the `'id` lifetime is restricted to be between the drop timing of `tag`
  (which it borrows) and the drop timing of `_guard: make_guard(&'id tag)` (which holds it).
- Therefore, no lifetime can unify with `'id` unless it ends in the same region.
- All lifetimes created with `make_guard!` are protected in this manner.
- All lifetimes created with `make_guard!` are thus mutually ununifyable.
- It is unsafe to create a `generativity::Guard<'_>` without using `make_guard!`.
- Therefore, it is impossible to safely create a `generativity::Guard<'_>` that will unify lifetimes with `'id`.
- Thus, the lifetime created by `make_guard!` is guaranteed unique _in respect to other `generativity` lifetimes_.

## Huge generativity disclaimer

That last point above is ***VERY*** important. We _cannot_ guarantee that the
lifetime is fully unique. We can only guarantee that it's unique among trusted
carriers. This applies equally to the traditional method:

```rust
fn scope<F>(f: F)
where F: for<'id> FnOnce(Guard<'id>)
{
    make_guard!(guard);
    f(guard);
}

fn unify<'a>(_: &'a (), _: &Guard<'a>) {
    // here, you have two `'a` which are equivalent to `guard`'s `'id`
}

fn main() {
    let place = ();
    make_guard!(guard);
    unify(&place, &guard);
    scope(|guard| {
        unify(&place, &guard);
    })
}
```

Larger lifetimes can still shrink to unify with `'id`. What all this means is that you
can't just trust any old `'id` lifetime floating around. It has to be carried in a
trusted carrier; one that can't be created from an untrusted lifetime carrier.

## License

Licensed under either of

- Apache License, Version 2.0, (<LICENSE-APACHE> or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<LICENSE-MIT> or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

  [Discord]: <https://discord.gg/FuPE9JE>
  [Travis]: <https://travis-ci.com/CAD97/generativity>
  [Codecov]: <https://codecov.io/gh/CAD97/generativity>
  
  [variance]: <https://doc.rust-lang.org/nomicon/subtyping.html#variance>
