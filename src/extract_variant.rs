use std::fmt::{Debug, Write};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    punctuated::{Pair, Punctuated},
    token::{self, Comma},
    AngleBracketedGenericArguments, Attribute, Error, Expr, ExprLit, FieldsNamed, GenericArgument,
    Generics, ItemEnum, ItemStruct, Lifetime, Lit, LitStr, Meta, MetaList, MetaNameValue,
    ParenthesizedGenericArguments, Path, PathArguments, PathSegment, Result, ReturnType, Token,
    Type, TypePath, Variant,
};

use tap::prelude::*;

use crate::common::{
    generate_conversion_impl, ident, no_impl_value, path_id, Args, WrappedVariant,
};

pub fn doit(args: TokenStream, item_enum: ItemEnum) -> Result<TokenStream> {
    let value = Args::parse_terminated.parse2(args)?;
    let params = Params::try_from(value)?;
    let Config {
        map_ident,
        implement_conversions,
        style,
        derive_exclude,
    } = Config::new(params, &item_enum);

    let ItemEnum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        brace_token,
        variants,
    } = &item_enum;

    let global_derive = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .map(|attr| -> Result<Attribute> {
            Ok(Attribute {
                meta: if let Meta::List(list) = &attr.meta {
                    let a = Punctuated::<Path, Token![,]>::parse_terminated
                        .parse2(list.tokens.clone())?
                        .into_iter()
                        .filter(|path| !derive_exclude.contains(path));
                    Meta::List(MetaList {
                        tokens: quote! { #(#a),* },
                        ..list.clone()
                    })
                } else {
                    attr.meta.clone()
                },
                ..attr.clone()
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let wrap_variant = |variant: &Variant| {
        let attrs = variant.attrs.clone();
        let id = variant.ident.clone();
        let ty = Type::Path(TypePath {
            qself: None,
            path: Path::from(map_ident(&id)),
        });
        WrappedVariant { attrs, id, ty }
    };

    let wrapped_variants: Vec<WrappedVariant> = variants.iter().map(wrap_variant).collect();

    let variants_def = variants.iter().map(|variant| match style {
        Style::Wrap => wrap_variant(variant).into_token_stream(),
        Style::Keep => variant.to_token_stream(),
    });

    let generate_struct = |Variant {
                               attrs,
                               ident,
                               fields,
                               discriminant: _,
                           }: &Variant|
     -> Result<ItemStruct> {
        Ok(ItemStruct {
            attrs: attrs
                .iter()
                .filter_map(|attr| {
                    let path = attr.path();
                    if path.is_ident("attribute") {
                        Some(if let Meta::List(MetaList { tokens, .. }) = &attr.meta {
                            syn::parse2::<Meta>(tokens.clone()).map(|meta| Attribute {
                                meta,
                                ..attr.clone()
                            })
                        } else {
                            Err(Error::new_spanned(
                                attr.meta.clone(),
                                "must be in the form of `#[attribute(...)]`",
                            ))
                        })
                    } else if path.is_ident("doc") {
                        Some(Ok(attr.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Result<Vec<_>>>()?
                .tap_mut(|attrs| attrs.extend_from_slice(&global_derive)),
            vis: vis.clone(),
            struct_token: token::Struct {
                span: Span::call_site(),
            },
            ident: map_ident(&ident),
            generics: Generics::default(), // TODO
            fields: fields.clone(),
            semi_token: None,
        })
    };

    // let generated_structs = variants.iter().map(generate_struct);
    let generated_structs = variants
        .iter()
        .map(generate_struct)
        .collect::<Result<Vec<_>>>()?;
    // .try_fold(quote! {}, |acc, s| {
    //     s.map(
    //         |ItemStruct {
    //              attrs,
    //              vis,
    //              struct_token,
    //              ident,
    //              generics: _,
    //              fields,
    //              semi_token: _,
    //          }| {
    //             quote! {
    //                 #acc

    //                 #(#attrs)*
    //                 #(#global_derive)*
    //                 #vis #struct_token #ident {
    //                     #fields
    //                 }
    //             }
    //         },
    //     )
    // })?;

    let conversion_impls = wrapped_variants
        .iter()
        .map(|WrappedVariant { attrs: _, id, ty }| {
            if implement_conversions {
                match style {
                    Style::Wrap => generate_conversion_impl(ident, id, ty),
                    Style::Keep => todo!(),
                }
            } else {
                quote!()
            }
        });

    // if let Some(lt_token) = item_enum.generics.lt_token {
    //     return Err(Error::new_spanned(
    //         lt_token,
    //         "`extract_variant` does not support generic parameters",
    //     ));
    // }

    // if let Some(a) = item_enum
    //     .attrs
    //     .iter()
    //     .find(|attr| attr.path.is_ident("extract_variant"))
    //     .and_then(|attr| attr.parse_meta().ok())
    // {
    //     // dbg!(DebugWrapper(&a));
    // }

    Ok(quote! {
        #(#attrs)*
        #vis #enum_token #ident {
            #(#variants_def),*
        }
        #(#generated_structs)*
        #(#conversion_impls)*
    })
}

#[derive(Default)]
enum Style {
    /// Extract the fields definition out to a generated struct
    /// and use variant to wrap around that struct.
    #[default]
    Wrap,
    /// Keep the variant as defined, just generate struct that is
    /// identical the variant.
    Keep,
}
struct Config {
    map_ident: Box<dyn Fn(&Ident) -> Ident>,
    implement_conversions: bool,
    style: Style,
    derive_exclude: Vec<Path>,
}
impl Config {
    fn new(
        Params {
            prefix,
            suffix,
            no_impl,
            simplify,
            variant_style,
            derive_exclude,
        }: Params,
        item_enum: &ItemEnum,
    ) -> Self {
        let prefix = prefix.unwrap_or_default();
        let suffix = suffix.unwrap_or_default();
        Self {
            map_ident: Box::new(move |vid| format_ident!("{prefix}{vid}{suffix}")),
            implement_conversions: !no_impl.unwrap_or_default(),
            style: variant_style.unwrap_or_default(),
            derive_exclude,
        }
    }
}

#[derive(Default)]
struct Params {
    prefix: Option<String>,
    suffix: Option<String>,
    no_impl: Option<bool>,
    /// Simplifies the variants in 3 cases depending on the level.
    /// Simplification here means reducing the variant into a unit variant.
    ///
    /// Cases:
    /// 1. `Variant` unit variant (only for "wrap" style)
    /// 2. `Variant()` empty tuple-like variant
    /// 3. `Variant {}` empty named-fields variant
    ///
    /// On level:
    /// - 0: (default) do nothing
    /// - 1: simplify case 1 only
    /// - 2: simplify all cases
    simplify: Option<u32>,
    variant_style: Option<Style>,
    derive_exclude: Vec<Path>,
    // generic: TODO
}

impl TryFrom<Args> for Params {
    type Error = Error;
    fn try_from(args: Args) -> std::result::Result<Self, Self::Error> {
        let mut params = Params::default();
        for arg in args {
            let ident = ident(&arg)?;
            match ident.to_string().as_str() {
                "prefix" => {
                    macro_rules! error {
                        ($tokens:expr) => {
                            Error::new_spanned(
                                $tokens,
                                r#"valid forms are `prefix(Ident)` or `prefix = "Ident"`"#,
                            )
                        };
                    }
                    params.prefix = Some(match arg {
                        Meta::Path(path) => Err(error!(path))?,
                        Meta::List(MetaList { tokens, .. }) => {
                            let id: Ident = syn::parse2(tokens.clone()).map_err(|mut err| {
                                err.combine(error!(tokens));
                                err
                            })?;
                            id.to_string()
                        }
                        Meta::NameValue(MetaNameValue {
                            value:
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(s), ..
                                }),
                            ..
                        }) => s.value(),
                        _ => Err(error!(arg))?,
                    })
                }
                "suffix" => {
                    macro_rules! error {
                        ($tokens:expr) => {
                            Error::new_spanned(
                                $tokens,
                                r#"valid forms are `suffix(Ident)` or `suffix = "Ident"`"#,
                            )
                        };
                    }
                    params.suffix = Some(match arg {
                        Meta::Path(path) => Err(error!(path))?,
                        Meta::List(MetaList { tokens, .. }) => {
                            let id: Ident = syn::parse2(tokens.clone()).map_err(|mut err| {
                                err.combine(error!(tokens));
                                err
                            })?;
                            id.to_string()
                        }
                        Meta::NameValue(MetaNameValue {
                            value:
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(s), ..
                                }),
                            ..
                        }) => s.value(),
                        _ => Err(error!(arg))?,
                    })
                }
                "no_impl" => params.no_impl = no_impl_value(arg)?,
                "style" => {
                    macro_rules! error {
                        ($tokens:expr) => {
                            Error::new_spanned(
                                $tokens,
                                r#"valid forms are `style = "wrap"`, or `style = "keep"`"#,
                            )
                        };
                    }
                    params.variant_style = if let Meta::NameValue(MetaNameValue { value, .. }) = arg
                    {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(str), ..
                        }) = value
                        {
                            match str.value().as_str() {
                                "wrap" => Some(Style::Wrap),
                                "keep" => Some(Style::Keep),
                                _ => Err(error!(str))?,
                            }
                        } else {
                            Err(error!(value))?
                        }
                    } else {
                        Err(error!(arg))?
                    }
                }
                "derive_exclude" => {
                    macro_rules! error {
                        ($tokens:expr) => {
                            Error::new_spanned(
                                $tokens,
                                r#"valid form is `derive_exclude(Path0, Path1, ...)`"#,
                            )
                        };
                    }

                    if let Meta::List(MetaList {
                        path,
                        delimiter,
                        tokens,
                    }) = arg
                    {
                        let a = Punctuated::<Path, Token![,]>::parse_terminated.parse2(tokens)?;
                        params.derive_exclude.extend(a.into_iter())
                    } else {
                        Err(error!(arg))?
                    }
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

#[cfg(test)]
mod tests {
    use super::doit;
    use quote::quote;

    #[test]
    fn curly() {
        let s = doit(
            quote!(),
            syn::parse2(quote! {
                #[derive(Debug)]
                enum MyEnum {
                    #[something]
                    C {},
                }
            })
            .unwrap(),
        );
        assert!(s.is_ok());
        println!("{}", s.unwrap().to_string());
        assert!(false);
    }
}
