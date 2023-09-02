use enum_macros::variant_wrapper;

#[variant_wrapper(no_impl, no_impl)]
enum MyEnum {
    A,
    B(),
    C {},
}

fn main() {}
