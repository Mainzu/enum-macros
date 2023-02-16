// use proc_macro::TokenStream;
// use quote::{quote, ToTokens};
// use syn::{parse_macro_input, DeriveInput, ItemEnum, Meta, NestedMeta};

// #[proc_macro_derive(VariantImplement, attributes(variant_implement))]
// pub fn variant_implement(input: TokenStream) -> TokenStream {
//     let ast = parse_macro_input!(input as ItemEnum);
//     let name = &ast.ident;
//     let mut variants = vec![];
//     let mut method_name = None;

//     ast.attrs.iter().find(|attr|attr.path.is_ident(""))
//     for attr in &ast.attrs {
//         match attr.parse_meta().unwrap() {
//             Meta::List(list) => {
//                 if list.path.is_ident("variant_implement") {
//                     for meta in &list.nested {
//                         match meta {
//                             NestedMeta::Meta(Meta::Word(name)) => {
//                                 method_name = Some(name.to_string());
//                             }
//                             _ => {}
//                         }
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }

//     match &ast.data {
//         syn::Data::Enum(data) => {
//             for variant in &data.variants {
//                 variants.push(&variant.ident);
//             }
//         }
//         _ => panic!("#[derive(VariantImplement)] can only be used on enums"),
//     }

//     let method_name = method_name.unwrap();
//     let tokens = quote! {
//         impl #name {
//             fn #method_name(&self) -> &str {
//                 match self {
//                     #(#name::#variants(inner) => inner.#method_name(),)*
//                 }
//             }
//         }
//     };

//     TokenStream::from(tokens)
// }
// // Note that the above code is just a skeleton and you might need to add some error handling and additional functionality depending on the requirements of your macro.
