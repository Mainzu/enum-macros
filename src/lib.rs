// #![allow(unused)]
#![feature(iterator_try_collect)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, Result};

mod common;

mod variant_wrapper;

mod extract_variant;
mod variant_implement;
mod variant_of;

#[inline]
fn result_of(doit: Result<impl Into<TokenStream>>) -> TokenStream {
    match doit {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// Extracts each variant in an enum into its own standalone struct, then implements conversion traits
/// between the original enum and the generated struct.
///
/// Take argument in the same format as other proc_macro_attribute, eg. `#[extract_variant(no_impl, prefix(Extracted), style = "wrapped")]`.
///
/// Valid arguments:
/// - `no_impl`: stop [`From`] variant and [`TryFrom`] enum from being implemented.
/// - `prefix`: prepend to the identifier of every generated structs.
/// - `suffix`: append to the identifier of every generated structs.
/// - `style`: affect the enum itself, can be one of two values
///     - "wrapped": the default, each enum variant is a tuple holding the generated type
///     - "keep": each enum variant is exactly the same as the generated type (more inconvenient)
///
/// TODO
/// - `debug(transparent)`
/// - `display(transparent)`: does not implement the display for each variant
#[proc_macro_attribute]
pub fn extract_variant(args: TokenStream, input: TokenStream) -> TokenStream {
    result_of(extract_variant::doit(
        parse_macro_input!(args),
        parse_macro_input!(input),
    ))
}

// #[proc_macro_derive(VariantImplement, attributes(variant_implement))]
// pub fn variant_derive(input: TokenStream) -> TokenStream {
//     todo!()
// }

// #[proc_macro_derive(extract_variant, attributes(variant_of))]

/// Annotate the enum as a wrapper for its variants. Useful for when you already have its
/// variant type(s) defined as struct(s).
///
/// The primary purpose of this macro is to create [`From`] and [`TryFrom`] implementations.
#[proc_macro_attribute]
pub fn variant_wrapper(args: TokenStream, input: TokenStream) -> TokenStream {
    result_of(variant_wrapper::doit(
        parse_macro_input!(args),
        parse_macro_input!(input),
    ))
}

/// Does nothing by itself
#[proc_macro_derive(EnableExtraParameters, attributes(attribute))]
pub fn enable_extra_parameters(_input: TokenStream) -> TokenStream {
    TokenStream::default()
}
