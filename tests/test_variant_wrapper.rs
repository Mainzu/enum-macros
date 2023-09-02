use enum_macros::variant_wrapper;

struct A;
struct B;

#[variant_wrapper()]
enum MyEnum {
    A,
    B(B),
}

fn f() {
    let a = MyEnum::from(A);
}
