#![cfg(feature = "tag")]
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    Error, ItemEnum, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path, Result,
};

use crate::common::{ident, Args, AttributeArgs};

pub fn doit(item_enum: ItemEnum) -> Result<TokenStream> {
    let ItemEnum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        brace_token,
        variants,
    } = &item_enum;

    let args = item_enum
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("tagged"))
        .ok_or_else(|| Error::new(Span::call_site(), "missing `#[tagged(...)]` attribute"))?
        .parse_args_with(Args::parse_terminated)?;

    let params = Params::try_from(args)?;
    // let tag = params.tag.un;
    let Config { tag } = Config::new(params, &item_enum);

    let generated_tag = match &tag {
        Tag::Generate(ident) => {
            let stripped_variants = variants.iter().map(|variants| &variants.ident);
            quote! {
                #vis enum #ident {
                    #(#stripped_variants),*
                }
            }
        }
        Tag::Existing(_) => quote!(),
    };
    Ok(quote! {
        impl ::enum_tag::Tagged for #ident {
            type Tag = #tag;
        }
        #generated_tag
    })
}
#[derive(Default)]
struct Params {
    tag: Option<Tag>,
}
impl TryFrom<Args> for Params {
    type Error = Error;
    fn try_from(args: Args) -> std::result::Result<Self, Self::Error> {
        let mut params = Params::default();
        const TAG_VALID_FORMS: &str =
            "valid forms are `tag(PathTo::ExistingEnum)`, `tag(generate())`, or `tag(generate(NameOfEnumToBeGenerated))`";

        for arg in args {
            let ident = ident(&arg)?;

            match ident.to_string().as_str() {
                "tag" => {
                    params.tag = if let Meta::List(MetaList {
                        path,
                        paren_token,
                        nested,
                    }) = arg
                    {
                        if nested.len() != 1 {
                            Err(Error::new(paren_token.span, TAG_VALID_FORMS))?
                        }
                        let item = nested.into_iter().next().unwrap();
                        match item {
                            NestedMeta::Meta(Meta::Path(path)) => Some(Tag::Existing(path)),
                            NestedMeta::Meta(Meta::List(MetaList {
                                path,
                                paren_token,
                                nested,
                            })) => {
                                if !path.is_ident("generate") {
                                    Err(Error::new_spanned(path, TAG_VALID_FORMS))?
                                }
                                match nested.len() {
                                    0 => None,
                                    1 => {
                                        let item = nested.into_iter().next().unwrap();
                                        if let NestedMeta::Meta(Meta::Path(path)) = item {
                                            if let Some(ident) = path.get_ident().cloned() {
                                                Some(Tag::Generate(ident))
                                            } else {
                                                Err(Error::new_spanned(
                                                    path,
                                                    "must be a bare identifier",
                                                ))?
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                    _ => Err(Error::new_spanned(
                                        nested,
                                        "must be a bare identifier",
                                    ))?,
                                }
                            }
                            a => Err(Error::new_spanned(a, TAG_VALID_FORMS))?,
                        }
                    } else {
                        Err(Error::new_spanned(arg, TAG_VALID_FORMS))?
                    }
                }
                _ => Err(Error::new_spanned(ident, "Tagged: unrecognized parameter"))?,
            }
        }
        Ok(params)
    }
}

enum Tag {
    Generate(Ident),
    Existing(Path),
}
impl ToTokens for Tag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Tag::Generate(ident) => ident.to_tokens(tokens),
            Tag::Existing(path) => path.to_tokens(tokens),
        }
    }
}
struct Config {
    tag: Tag,
}
impl Config {
    fn new(Params { tag }: Params, item_enum: &ItemEnum) -> Self {
        Self {
            tag: tag.unwrap_or(Tag::Generate(Ident::new(
                &format!("{}Tag", item_enum.ident.to_string()),
                Span::call_site(),
            ))),
        }
    }
}
