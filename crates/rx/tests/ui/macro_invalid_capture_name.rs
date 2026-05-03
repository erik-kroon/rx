fn main() {
    let _ = rx::pattern! {
        named_capture("123-id", one_or_more(set(ascii::digit)))
    };
}
