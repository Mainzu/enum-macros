use enum_macros::{extract_variant, variant_wrapper};

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

#[variant_wrapper(no_impl = false)]
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
