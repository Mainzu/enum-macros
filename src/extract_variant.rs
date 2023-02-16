use std::fmt::{Debug, Write};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    punctuated::{Pair, Punctuated},
    token::{Colon2, Comma},
    AngleBracketedGenericArguments, Attribute, Error, GenericArgument, ItemEnum, Lifetime, Lit,
    Meta, MetaList, MetaNameValue, NestedMeta, ParenthesizedGenericArguments, Path, PathArguments,
    PathSegment, Result, ReturnType, Type, Variant,
};

#[derive(Debug, Default)]
struct Config {
    prefix: String,
    suffix: String,
    no_impl: bool,
}

pub fn doit(item_enum: ItemEnum) -> Result<TokenStream> {
    if let Some(lt_token) = item_enum.generics.lt_token {
        return Err(Error::new_spanned(
            lt_token,
            "`extract_variant` does not support generic parameters",
        ));
    }

    let mut config = Config::default();
    if let Some(a) = item_enum
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("extract_variant"))
        .and_then(|attr| attr.parse_meta().ok())
    {
        // dbg!(DebugWrapper(&a));
    }
    todo!()
}
