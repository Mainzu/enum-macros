#![cfg(feature = "tag")]

use enum_macros::Tagged;

enum Tag {
    A,
    B,
    C,
}

#[derive(Tagged)]
// #[tagged(tag(generate(X)))]
// #[tagged(tag(generate()))]
#[tagged(tag(Tag))]
enum MyEnum {
    A,
    B(),
    C {},
}

fn f() {
    // MyEnumTag::A;
    let a = X::A;
}
