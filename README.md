# Generativity [![Chat on Discord](https://img.shields.io/discord/496481045684944897?style=flat&logo=discord)][Discord]

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

It's important to note that lifetimes are only trusted to carry lifetime brands when they
are in fact invariant. Variant lifetimes, such as `&'a T`, can be shrunk to fit the expected
lifetime; `&'static T` can be used where `&'a T` is expected, for any `'a`.

## Informal proof of correctness

```rust
let tag = unsafe { Id::new() };
let $name = unsafe { Guard::new(tag) };
let _guard = {
    #[allow(non_camel_case_types)]
    struct make_guard<'id> {
        _id: Id<'id>,
    }
    impl<'id> Drop for make_guard<'id> {
        #[inline(always)]
        fn drop(&mut self) {}
    }
    fn make_guard<'id>(_: &'id Id<'id>) -> make_guard<'id> {
        make_guard { _id: *id }
    }
    make_guard(&tag)
};
```

### Disclaimer

This relies on dead code (the empty drop) to impact borrow checking.
Theoretically, a smarter CFG based borrow checker (i.e. NLL/polonius) *could*
utilize the fact that this is dead code to remove this restriction, but this is
*very* unlikely; the current (as of Rust 2021) NLL borrow checker requires dead
code to be lifetime-correct, and this code isn't *dead* dead, as in, it *runs*,
it just doesn't actually do anything other than impact lifetime solving. If you
want to discuss the proof of correctness, the place to do so is [issue #1].

[issue #1]: https://github.com/CAD97/generativity/issues/1

### Unimportant, nicety details

- New unique type per macro invocation: this is merely to avoid having a type in
  the public API, and such that the compiler emits slightly more useful error
  messages for lifetime errors.
- `#[inline(always)]`: This is a micro-optimization not required for safety.
  This makes it easier for the optimizer to optimize out the `_guard`'s drop
  implementation.
- `make_guard` is created from `&'id Id<'id>` but only holds `Id<'id>`. While
  the reference is required to uniquify the lifetime (see below), only `Id<'id>`
  is required to carry the invariant lifetime.

### Three places, all required

- `$name` is the user-named `#[unique] 'id` type that we give to the calling context.
- `tag` is a location that we use to define the `'id` lifetime without restricting `$name`.
- `_guard` is an `impl Drop` that we use to restrict `'id`.

### `'id` is unique

- `'id` is _invariant_ due to the invariance of `tag: generativity::Id<'id>`.
- `$name: generativity::Guard<'id>` has the same `'id` because it is created from `tag`.
- `'id` is restricted by creating `_guard: make_guard(&'id generativity::Tag<'id>)`.
- The end point of the `'id` lifetime is restricted to be between the drop timing of `tag`
  (which it borrows) and the drop timing of `_guard: make_guard(&'id tag)` (which holds it).
- Therefore, no lifetime can unify with `'id` unless it ends in the same region.
- All lifetimes created with `make_guard!` are protected in this manner.
- All lifetimes created with `make_guard!` are thus mutually ununifyable.
- It is unsafe to create a `generativity::Guard<'_>` without using `make_guard!`.
- Therefore, it is impossible to safely create a `generativity::Guard<'_>` that
  will unify lifetimes with `'id`.
- Thus, the lifetime created by `make_guard!` is guaranteed unique
  _with respect to other `generativity` lifetimes_.

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

Other variant lifetimes can still shrink to unify with `'id`. What all this
means is that you can't just trust any old `'id` lifetime floating around. It
has to be carried in a trusted carrier, and one that wasn't created from an
untrusted lifetime.

## Minimum supported Rust version

In theory, this crate should work on even ancient pre-edition Rust versions.
However, the crate is only tested to work as desired on versions that trybuild
targets. As of publishing this version of the crate, that is Rust 1.36+.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

  [Discord]: <https://discord.gg/FuPE9JE>
  [variance]: <https://doc.rust-lang.org/nomicon/subtyping.html#variance>
