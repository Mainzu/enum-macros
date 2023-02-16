// #![allow(unused)]
#![feature(iterator_try_collect)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, Result};

mod common;

mod extract_variant;
mod variant_implement;
mod variant_of;
mod variant_wrapper;

#[inline]
fn result_of(doit: Result<impl Into<TokenStream>>) -> TokenStream {
    match doit {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_derive(
    ExtractVariant,
    attributes(extract_variant, variant_attrs, exclude, include)
)]
pub fn extract_variant(input: TokenStream) -> TokenStream {
    result_of(extract_variant::doit(parse_macro_input!(input)))
}

// #[proc_macro_derive(VariantImplement, attributes(variant_implement))]
// pub fn variant_derive(input: TokenStream) -> TokenStream {
//     todo!()
// }

// #[proc_macro_derive(extract_variant, attributes(variant_of))]

#[proc_macro_attribute]
pub fn variant_wrapper(args: TokenStream, input: TokenStream) -> TokenStream {
    result_of(variant_wrapper::doit(
        parse_macro_input!(args),
        parse_macro_input!(input),
    ))
}
