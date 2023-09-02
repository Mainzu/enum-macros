use enum_macros::extract_variant;

#[extract_variant(suffix = "Suffix")]
enum MyEnum {
    A,
    B(),
    C {},
}

impl MyEnum {}

fn f() {
    MyEnum::A(ASuffix);
    MyEnum::from(ASuffix);
    MyEnum::from(BSuffix());
    MyEnum::from(CSuffix {});
    let a = ASuffix::try_from(MyEnum::A(ASuffix));
}
