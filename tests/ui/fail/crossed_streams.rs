use generativity::make_guard;

fn main() {
    make_guard!(a);
    make_guard!(b);
    dbg!(a == b); // ERROR (here == is a static check)
}
