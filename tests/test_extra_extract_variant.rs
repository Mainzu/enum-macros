use enum_macros::{extract_variant, EnableExtraParameters};

#[derive(EnableExtraParameters)]
#[extract_variant]
enum MyEnum {
    #[attribute(derive(Debug))]
    A,
    #[attribute(derive(Debug, PartialEq, Eq))]
    B(),
    #[attribute(derive(Debug, Clone, Copy))]
    C {},
}

impl MyEnum {}

fn f() {
    format!("{:?}", A);
    B() == B();
    C {}.clone();
}

#[test]
fn a() {
    assert_eq!(format!("{:?}", A), "A");
}

#[test]
fn b() {
    assert_eq!(B(), B())
}
