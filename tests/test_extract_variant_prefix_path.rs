use enum_macros::extract_variant;

#[extract_variant(prefix(Prefix))]
enum MyEnum {
    A,
    B(),
    C {},
}

impl MyEnum {}

fn f() {
    MyEnum::A(PrefixA);
    MyEnum::from(PrefixA);
    MyEnum::from(PrefixB());
    MyEnum::from(PrefixC {});
    let a = PrefixA::try_from(MyEnum::A(PrefixA));
}
