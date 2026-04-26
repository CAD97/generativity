use generativity::guard;

fn main() {
    let a = guard!(a);
    let b = guard!(b);
    dbg!(a == b); // ERROR (here == is a static check)
}
