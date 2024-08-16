use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Attribute, Error, Fields, FieldsUnnamed, ItemEnum, Lit, LitBool, Meta, MetaList, Path, Result,
    Token, Type, TypePath, Variant,
};

use crate::common::{
    generate_conversion_impl, ident, kw, no_impl_value, optional_attribute_args_list,
    APIAttributeArgs, AttributeArgs, Eq, NoImpl, WrappedVariant,
};

type Params = Punctuated<Param, Token![,]>;
#[inline]
fn parse_params(args: TokenStream) -> Result<Params> {
    Params::parse_terminated.parse2(args)
}

pub fn doit(args: TokenStream, item_enum: ItemEnum) -> Result<TokenStream> {
    let params = parse_params(args)?;
    let options = Options::try_from(params)?; // TODO - this naming scheme is really stupid, should probabably change it some day
    let Config {
        implement_conversion,
    } = Config::new(options);

    let ItemEnum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        brace_token,
        variants,
    } = &item_enum;

    let wrapped_variants: Vec<WrappedVariant> =
        item_enum.variants.iter().map(wrap_variant).try_collect()?;

    let conversion_impls = wrapped_variants
        .iter()
        .map(|WrappedVariant { attrs: _, id, ty }| {
            if implement_conversion {
                generate_conversion_impl(ident, id, ty)
            } else {
                quote!()
            }
        });

    Ok(quote! {
        #(#attrs)*
        #vis #enum_token #ident {
            #(#wrapped_variants),*
        }
        #(#conversion_impls)*
    })
}

fn wrap_variant(variant: &Variant) -> Result<WrappedVariant> {
    let attrs = variant.attrs.clone();
    let id = variant.ident.clone();
    let ty = match &variant.fields {
        Fields::Named(named_fields) => Err(Error::new(
            named_fields.brace_token.span.join(),
            "named fields unsupported",
        ))?,
        Fields::Unnamed(FieldsUnnamed {
            unnamed,
            paren_token,
        }) => {
            if unnamed.len() != 1 {
                Err(Error::new(
                    paren_token.span.join(),
                    "tuple-like variant must have exactly 1 field",
                ))?
            }
            unnamed.first().unwrap().ty.clone()
        }
        Fields::Unit => Type::Path(TypePath {
            qself: None,
            path: Path::from(id.clone()),
        }),
    };
    Ok(WrappedVariant { attrs, id, ty })
}

enum Param {
    NoImpl(NoImpl),
}
impl Parse for Param {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::no_impl) {
            Ok(Param::NoImpl(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}
fn fill_empty_or_else<T>(
    opt: &mut Option<T>,
    new: T,
    err: impl FnOnce(&T, T) -> Error,
) -> Result<()> {
    match opt {
        Some(old) => Err(err(&old, new)),
        None => Ok({
            opt.replace(new);
        }),
    }
}

// #[derive(FromMeta)]
struct Config {
    implement_conversion: bool,
}
impl Config {
    fn new(Options { no_impl }: Options) -> Self {
        Self {
            implement_conversion: !no_impl.map_or(false, |a| a.truthy()),
        }
    }
}

#[derive(Default)]
struct Options {
    no_impl: Option<NoImpl>,
}
impl TryFrom<Params> for Options {
    type Error = Error;
    fn try_from(params: Params) -> std::result::Result<Self, Self::Error> {
        let mut options = Options::default();
        for arg in params {
            match arg {
                Param::NoImpl(no_impl) => {
                    fill_empty_or_else(&mut options.no_impl, no_impl, |old, new| {
                        Error::new_spanned(new, "duplicate parameter")
                    })?
                }
            }
        }
        Ok(options)
    }
}

fn pipeline(input: TokenStream) -> Result<Options> {
    let params = parse_params(input)?;
    Ok(Options::try_from(params)?)
}
#[test]
fn test() {
    for input in [
        quote!(),
        quote!(no_impl),
        quote!(no_impl = true),
        quote!(no_impl = false),
    ] {
        let _ = pipeline(input).unwrap();
    }
}
// impl TryFrom<Args> for Options {
//     type Error = Error;

//     fn try_from(args: Args) -> std::result::Result<Self, Self::Error> {
//         let mut params = Options::default();

//         for arg in args {
//             params
//             // let ident = ident(&arg)?;

//             // match ident.to_string().as_str() {
//             //     "no_impl" => params.no_impl = no_impl_value(arg)?,
//             //     _ => Err(Error::new_spanned(
//             //         ident,
//             //         "variant_wrapper: unrecognized parameter",
//             //     ))?,
//             // }
//         }
//         Ok(params)
//     }
// }
mod m {
    use std::borrow::Cow;
    trait Trait: Clone {
        fn f() {}
    }
    fn f(cow: impl AsRef<i32>) {
        ()
    }
}
