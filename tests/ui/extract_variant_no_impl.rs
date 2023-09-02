use enum_macros::extract_variant;

#[extract_variant(no_impl)]
enum MyEnum {
    A,
    B(),
    C {},
}

impl MyEnum {}

fn main() {
    MyEnum::A(A);

    MyEnum::from(A);

    let a = A::try_from(MyEnum::A(A));
}
