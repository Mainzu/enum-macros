use std::fmt::{Debug, Write};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parser,
    punctuated::{Pair, Punctuated},
    token::{self, Colon2, Comma},
    AngleBracketedGenericArguments, Attribute, Error, FieldsNamed, GenericArgument, Generics,
    ItemEnum, ItemStruct, Lifetime, Lit, Meta, MetaList, MetaNameValue, NestedMeta,
    ParenthesizedGenericArguments, Path, PathArguments, PathSegment, Result, ReturnType, Type,
    TypePath, Variant,
};

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

    let wrap_variant = |variant: &Variant| {
        let id = variant.ident.clone();
        let ty = Type::Path(TypePath {
            qself: None,
            path: Path::from(map_ident(&id)),
        });
        WrappedVariant { id, ty }
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
                               discriminant,
                           }: &Variant| {
        ItemStruct {
            attrs: attrs.clone(),
            vis: vis.clone(),
            struct_token: token::Struct {
                span: Span::call_site(),
            },
            ident: map_ident(&ident),
            generics: Generics::default(), // TODO
            fields: fields.clone(),
            semi_token: None,
        }
    };

    let generated_structs = variants.iter().map(generate_struct);

    let conversion_impls = wrapped_variants.iter().map(|WrappedVariant { id, ty }| {
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
}
impl Config {
    fn new(
        Params {
            prefix,
            suffix,
            no_impl,
            simplify,
            variant_style,
        }: Params,
        item_enum: &ItemEnum,
    ) -> Self {
        let prefix = prefix.unwrap_or_default();
        let suffix = suffix.unwrap_or_default();
        Self {
            map_ident: Box::new(move |vid| format_ident!("{prefix}{vid}{suffix}")),
            implement_conversions: !no_impl.unwrap_or_default(),
            style: variant_style.unwrap_or_default(),
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
                    params.prefix = Some(match arg {
                        Meta::Path(path) => Err(Error::new_spanned(path, "`prefix` unset"))?,
                        Meta::List(MetaList {
                            path,
                            paren_token,
                            ref nested,
                        }) => {
                            if nested.len() != 1 {
                                Err(Error::new_spanned(nested, "must have exactly 1 field"))?
                            }
                            match nested.first().unwrap() {
                                NestedMeta::Meta(Meta::Path(path)) => path_id(path)?.to_string(),
                                NestedMeta::Lit(Lit::Str(str)) => str.value(),
                                _ => Err(Error::new_spanned(
                                    nested,
                                    r#"valid forms are `prefix(Ident)` or `prefix = "Ident"`"#,
                                ))?,
                            }
                        }
                        Meta::NameValue(MetaNameValue {
                            path,
                            eq_token,
                            lit: Lit::Str(str),
                        }) => str.value(),
                        _ => Err(Error::new_spanned(
                            arg,
                            r#"valid forms are `prefix(Ident)` or `prefix = "Ident"`"#,
                        ))?,
                    })
                }
                "suffix" => {
                    params.suffix = Some(match arg {
                        Meta::Path(path) => Err(Error::new_spanned(path, "`suffix` unset"))?,
                        Meta::List(MetaList {
                            path,
                            paren_token,
                            ref nested,
                        }) => {
                            if nested.len() != 1 {
                                Err(Error::new_spanned(nested, "must have exactly 1 field"))?
                            }
                            match nested.first().unwrap() {
                                NestedMeta::Meta(Meta::Path(path)) => path_id(path)?.to_string(),
                                NestedMeta::Lit(Lit::Str(str)) => str.value(),
                                _ => Err(Error::new_spanned(
                                    nested,
                                    r#"valid forms are `suffix(Ident)` or `suffix = "Ident"`"#,
                                ))?,
                            }
                        }
                        Meta::NameValue(MetaNameValue {
                            path,
                            eq_token,
                            lit: Lit::Str(str),
                        }) => str.value(),
                        _ => Err(Error::new_spanned(
                            arg,
                            r#"valid forms are `suffix(Ident)` or `suffix = "Ident"`"#,
                        ))?,
                    })
                }
                "no_impl" => params.no_impl = no_impl_value(arg)?,
                "style" => {
                    params.variant_style = if let Meta::NameValue(MetaNameValue {
                        path,
                        eq_token,
                        lit,
                    }) = arg
                    {
                        if let Lit::Str(str) = lit {
                            match str.value().as_str() {
                                "wrap" => Some(Style::Wrap),
                                "keep" => Some(Style::Keep),
                                _ => Err(Error::new_spanned(
                                    str,
                                    r#"valid forms are `style = "wrap"`, or `style = "keep"`"#,
                                ))?,
                            }
                        } else {
                            Err(Error::new_spanned(
                                lit,
                                r#"valid forms are `style = "wrap"`, or `style = "keep"`"#,
                            ))?
                        }
                    } else {
                        Err(Error::new_spanned(
                            arg,
                            r#"valid forms are `style = "wrap"`, or `style = "keep"`"#,
                        ))?
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
