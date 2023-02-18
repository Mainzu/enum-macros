use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    punctuated::Punctuated,
    Attribute, Error, ItemEnum, Lit, LitBool, Meta, MetaList, NestedMeta, Result, Token,
};

use crate::common::{
    generate_conversion_impl, ident, optional_attribute_args_list, APIAttributeArgs, Args,
    AttributeArgs, WrappedVariant,
};

pub fn doit(args: TokenStream, item_enum: ItemEnum) -> Result<TokenStream> {
    let args = Args::parse_terminated.parse2(args)?;
    let params = Params::try_from(args)?; // TODO - this naming scheme is really stupid, should probabably change it some day
    let Config {
        implement_conversion,
    } = Config::new(params);

    let ItemEnum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        brace_token,
        variants,
    } = &item_enum;

    let wrapped_variants: Vec<WrappedVariant> = item_enum
        .variants
        .iter()
        .map(TryFrom::try_from)
        .try_collect()?;

    let variants = if implement_conversion {
        &wrapped_variants[..]
    } else {
        &[]
    };
    let conversion_impls = variants
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

struct Config {
    implement_conversion: bool,
}
impl Config {
    fn new(Params { no_impl }: Params) -> Self {
        Self {
            implement_conversion: !no_impl.unwrap_or(false),
        }
    }
}

#[derive(Default)]
struct Params {
    no_impl: Option<bool>,
}

impl TryFrom<Args> for Params {
    type Error = Error;

    fn try_from(args: Args) -> std::result::Result<Self, Self::Error> {
        let mut params = Params::default();

        for arg in args {
            let ident = ident(&arg)?;

            match ident.to_string().as_str() {
                "no_impl" => {
                    params.no_impl = Some(match arg {
                        Meta::Path(..) => true,
                        Meta::List(list) => Err(Error::new(
                            list.paren_token.span,
                            "`no_impl` does not accept list",
                        ))?,
                        Meta::NameValue(value) => match value.lit {
                            Lit::Bool(LitBool { value, .. }) => value,
                            lit => Err(Error::new_spanned(lit, "expected bool"))?,
                        },
                    })
                }
                _ => Err(Error::new_spanned(
                    ident,
                    "variant_wrapper: unrecognized parameter",
                ))?,
            }
        }
        Ok(params)
    }
}
