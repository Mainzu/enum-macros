use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use proc_macro2::{Delimiter, Group, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, Error, Fields, FieldsNamed, FieldsUnnamed, Ident, Lit, Path, Token, Type,
    TypePath, Variant,
};

#[derive(Clone)]
pub struct AttributeArgValue {
    pub eq_token: Token![=],
    pub lit: Lit,
}
impl ToTokens for AttributeArgValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.eq_token.to_tokens(tokens);
        self.lit.to_tokens(tokens);
    }
}
#[derive(Clone)]
pub struct AttributeArg {
    pub ident: Ident,
    pub value: Option<AttributeArgValue>,
}
impl ToTokens for AttributeArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
pub type AttributeArgs = Punctuated<AttributeArg, Token![,]>;

pub struct APIAttributeArgs {
    pub raw: AttributeArgs,
    pub api: HashMap<String, AttributeArg>,
}
pub enum APIAttributeArgsConstructionError {
    DuplicateParameter {
        first: AttributeArg,
        key: String,
        second: AttributeArg,
    },
}
impl Debug for APIAttributeArgsConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APIAttributeArgsConstructionError::DuplicateParameter { first, key, second } => f
                .debug_struct("DuplicateParameter")
                .field("key", key)
                .field(
                    "first",
                    &first
                        .value
                        .as_ref()
                        .map(|value| value.lit.to_token_stream().to_string()),
                )
                .field(
                    "second",
                    &second
                        .value
                        .as_ref()
                        .map(|value| value.lit.to_token_stream().to_string()),
                )
                .finish(),
        }
    }
}
impl Display for APIAttributeArgsConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APIAttributeArgsConstructionError::DuplicateParameter { first, key, second } => {
                write!(
                    f,
                    "Duplicate parameter '{}' (first: {:?}, second: {:?})",
                    key,
                    first.to_token_stream().to_string(),
                    second.to_token_stream().to_string()
                )
            }
        }
    }
}
impl TryFrom<AttributeArgs> for APIAttributeArgs {
    type Error = APIAttributeArgsConstructionError;
    fn try_from(raw: AttributeArgs) -> Result<Self, Self::Error> {
        let mut api = HashMap::new();
        for arg in &raw {
            let key = arg.ident.to_string();
            if api.contains_key(&key) {
                Err(APIAttributeArgsConstructionError::DuplicateParameter {
                    first: api.remove(&key).unwrap(),
                    key,
                    second: arg.clone(),
                })?
            } else {
                api.insert(key, arg.clone());
            }
        }
        Ok(APIAttributeArgs { raw, api })
    }
}

pub struct AttributeArgsList {
    pub paren_token: token::Paren,
    pub args: AttributeArgs,
}
impl ToTokens for AttributeArgsList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Group::new(
            Delimiter::Parenthesis,
            self.args.to_token_stream(),
        ))
    }
}
pub fn optional_attribute_args_list(input: ParseStream) -> syn::Result<Option<AttributeArgsList>> {
    if input.peek(token::Paren) {
        Ok(Some(input.parse()?))
    } else {
        Ok(None)
    }
}
impl Parse for AttributeArgsList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(AttributeArgsList {
            paren_token: parenthesized!(content in input),
            args: Punctuated::parse_terminated(&content)?,
        })
    }
}
impl Parse for AttributeArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let eq_token = input.parse()?;
        if let Some(eq_token) = eq_token {
            Ok(AttributeArg {
                ident,
                value: Some(AttributeArgValue {
                    eq_token,
                    lit: input.parse()?,
                }),
            })
        } else {
            Ok(AttributeArg { ident, value: None })
        }
    }
}
impl Parse for AttributeArgValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AttributeArgValue {
            eq_token: input.parse()?,
            lit: input.parse()?,
        })
    }
}

pub struct WrappedVariant {
    pub id: Ident,
    pub ty: Type,
}

impl ToTokens for WrappedVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.id.to_tokens(tokens);
        tokens.append(Group::new(
            Delimiter::Parenthesis,
            self.ty.to_token_stream(),
        ));
    }
}

pub enum WrappedVariantsError {
    NamedFields(FieldsNamed),
    UnnamedFields(FieldsUnnamed),
}
impl TryFrom<&Variant> for WrappedVariant {
    type Error = WrappedVariantsError;
    fn try_from(variant: &Variant) -> Result<Self, Self::Error> {
        let id = variant.ident.clone();
        let ty = match &variant.fields {
            Fields::Named(named_fields) => {
                return Err(WrappedVariantsError::NamedFields(named_fields.clone()))
            }
            Fields::Unnamed(unnamed_fields) => {
                let FieldsUnnamed { unnamed, .. } = unnamed_fields;
                if unnamed.len() != 1 {
                    return Err(WrappedVariantsError::UnnamedFields(unnamed_fields.clone()));
                }
                unnamed.first().unwrap().ty.clone()
            }
            Fields::Unit => Type::Path(TypePath {
                qself: None,
                path: Path::from(id.clone()),
            }),
        };
        Ok(WrappedVariant { id, ty })
    }
}

pub fn generate_conversion_impl(ident: &Ident, id: &Ident, ty: &Type) -> TokenStream {
    quote! {
        impl ::core::convert::From<#ty> for #ident {
            fn from(value: #ty) -> Self {
                #ident::#id(value)
            }
        }

        impl ::core::convert::TryFrom<#ident> for #ty {
            type Error = #ident;
            fn try_from(value: #ident) -> ::core::result::Result<Self, Self::Error> {
                if let #ident::#id(value) = value {
                    ::core::result::Result::Ok(value)
                } else {
                    ::core::result::Result::Err(value)
                }
            }
        }
    }
}
