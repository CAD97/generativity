use generativity::{Id, make_guard};

fn assert_eq_lt<'id>(_: Id<'id>, _: Id<'id>) {}

fn main() {
    make_guard!(g_a);
    make_guard!(g_b);

    let a: Id = g_a.into();
    let b: Id = g_b.into();

    assert_eq_lt(a, b);

    loop {}
}
