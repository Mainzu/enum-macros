use enum_macros::extract_variant;

#[extract_variant()]
enum MyEnum {
    A,
    B(),
    C {},
}

impl MyEnum {}

fn f() {
    MyEnum::A(A);
    MyEnum::from(A);
    MyEnum::from(B());
    MyEnum::from(C {});
    let a = A::try_from(MyEnum::A(A));
}
