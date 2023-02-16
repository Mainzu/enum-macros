use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::Parser, Error, ItemEnum, Result};

use crate::common::{
    generate_conversion_impl, optional_attribute_args_list, APIAttributeArgs, AttributeArgs,
    WrappedVariant, WrappedVariantsError,
};

enum ConfigError {
    UnrecognizedParameter(Ident),
}
struct Config {}
impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
impl TryFrom<APIAttributeArgs> for Config {
    type Error = ConfigError;
    fn try_from(mut args: APIAttributeArgs) -> std::result::Result<Self, Self::Error> {
        let mut config = Config::default();
        if let Some(_) = args.api.remove("_") {}
        Ok(config)
    }
}

pub fn doit(args: TokenStream, ast: ItemEnum) -> Result<TokenStream> {
    let config = match Config::try_from(
        APIAttributeArgs::try_from(AttributeArgs::parse_terminated.parse2(args)?).unwrap(),
    ) {
        Ok(ok) => ok,
        Err(err) => match err {
            ConfigError::UnrecognizedParameter(_) => todo!(),
        },
    };

    let enum_token = ast.enum_token;
    let ident = &ast.ident;
    let vis = &ast.vis;
    let attrs = ast
        .attrs
        .iter()
        .filter(|attr| !attr.path.is_ident("variant_wrapper"));

    let wrapped_variants: Vec<WrappedVariant> =
        match ast.variants.iter().map(TryFrom::try_from).try_collect() {
            Ok(ok) => ok,
            Err(err) => match err {
                WrappedVariantsError::NamedFields(nf) => Err(Error::new(
                    nf.brace_token.span,
                    "variant_wrapper does not support named fields",
                ))?,
                WrappedVariantsError::UnnamedFields(uf) => Err(Error::new(
                    uf.paren_token.span,
                    "variant_wrapper: tuple-like variant must have exactly 1 field",
                ))?,
            },
        };

    let conversion_impls = wrapped_variants
        .iter()
        .map(|WrappedVariant { id, ty }| generate_conversion_impl(ident, id, ty));

    Ok(quote! {
        #(#attrs)*
        #vis #enum_token #ident {
            #(#wrapped_variants),*
        }
        #(#conversion_impls)*
    })
}
