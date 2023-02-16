use enum_macros::{variant_wrapper, ExtractVariant};

// #[derive(ExtractVariant)]
// #[extract_variant(prefix = "Hello", suffix = "World")]
// enum A {
//     World,
//     John,
//     There,
// }

struct A;
struct B();
struct C {}

#[variant_wrapper(x = 1, x = 1)]
enum MyEnum {
    A,
    B,
    C,
}

impl MyEnum {}

fn f() {
    MyEnum::A(A);
    MyEnum::from(A);
    MyEnum::from(B());
    MyEnum::from(C {});
    let a = A::try_from(MyEnum::A(A));
}
