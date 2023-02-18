#![cfg(feature = "tag")]

use enum_macros::Tagged;

enum Tag {
    A,
    B,
    C,
}

#[derive(Tagged)]
// #[tagged(tag(generate(X)))]
#[tagged(tag(generate()))]
enum MyEnum {
    A,
    B(),
    C {},
}

fn f() {

    // let a = X::A;
}
