fn main() {
    let _ = rx::pattern! {
        one_or_more(set(ascii::alnum) literal("x"))
    };
}
