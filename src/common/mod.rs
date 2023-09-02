use std::{
    collections::HashMap,
    default,
    fmt::{Debug, Display},
};

use proc_macro2::{Delimiter, Group, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    custom_keyword, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Error, Expr, ExprLit, Fields, FieldsNamed, FieldsUnnamed, Ident, Lit,
    LitBool, LitInt, LitStr, Meta, MetaNameValue, Path, Result, Token, Type, TypePath, Variant,
};

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

pub type Args = Punctuated<Meta, Token![,]>;
pub fn ident(arg: &Meta) -> Result<&Ident> {
    path_id(arg.path())
}
pub fn path_id(path: &Path) -> Result<&Ident> {
    path.get_ident()
        .ok_or_else(|| Error::new_spanned(path, "must be a bare identifier"))
}

pub fn no_impl_value(arg: Meta) -> Result<Option<bool>> {
    Ok(Some(match arg {
        Meta::Path(..) => true,
        Meta::List(list) => Err(Error::new(
            list.delimiter.span().join(),
            "`no_impl` does not accept list",
        ))?,
        Meta::NameValue(MetaNameValue { value, .. }) => match value {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(LitBool { value, .. }),
                ..
            }) => value,
            _ => Err(Error::new_spanned(value, "expected bool"))?,
        },
    }))
}

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
    fn try_from(raw: AttributeArgs) -> std::result::Result<Self, Self::Error> {
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

#[cfg(feature = "tag")]
pub use tag::*;
#[cfg(feature = "tag")]
mod tag {
    use super::*;
    pub fn generate_variant_of_impl(ident: &Ident, id: &Ident, ty: &Type) -> TokenStream {
        quote! {
            impl ::enum_tag::VariantOf<#ident> for #ty {
                const TAG: #ident::Tag = #ident::Tag::#id;
            }
        }
    }
}

pub struct Visitor<T>(Result<T>);
impl<T: Default> Default for Visitor<T> {
    fn default() -> Self {
        Self(Ok(T::default()))
    }
}
pub mod kw {
    use syn::custom_keyword;

    custom_keyword!(no_impl);
    custom_keyword!(prefix);
    custom_keyword!(suffix);
    custom_keyword!(style);
    custom_keyword!(simplify);
    custom_keyword!(tag);
    custom_keyword!(generate);
}

pub struct Eq<T = Lit> {
    pub eq_token: Token![=],
    pub value: T,
}
impl<T: Parse> Parse for Eq<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}
impl<T: ToTokens> ToTokens for Eq<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.eq_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
pub struct Parenthesized<T> {
    pub paren_token: token::Paren,
    pub value: T,
}
impl<T: ToTokens> ToTokens for Parenthesized<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Group::new(
            Delimiter::Parenthesis,
            self.value.to_token_stream(),
        ))
    }
}
impl<T: Parse> Parse for Parenthesized<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            value: content.parse()?,
        })
    }
}
pub struct NoImpl {
    pub no_impl: kw::no_impl,
    pub value: Option<Eq<LitBool>>,
}
impl NoImpl {
    pub fn truthy(&self) -> bool {
        self.value.as_ref().map_or(true, |eq| eq.value.value)
    }
}
impl Parse for NoImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            no_impl: input.parse()?,
            value: if input.peek(Token![=]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}
impl ToTokens for NoImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.no_impl.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
pub struct Prefix {
    pub prefix: kw::prefix,
    pub value: Parenthesized<Ident>,
}
impl Parse for Prefix {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            prefix: input.parse()?,
            value: input.parse()?,
        })
    }
}
impl ToTokens for Prefix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.prefix.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
pub struct Suffix {
    pub suffix: kw::suffix,
    pub value: Parenthesized<Ident>,
}
impl Parse for Suffix {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            suffix: input.parse()?,
            value: input.parse()?,
        })
    }
}
impl ToTokens for Suffix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.suffix.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
pub enum StyleValue {
    Wrap(LitStr),
    Keep(LitStr),
}
impl Parse for StyleValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let str: LitStr = input.parse()?;
        match str.value().as_str() {
            "wrap" => Ok(StyleValue::Wrap(str)),
            "keep" => Ok(StyleValue::Keep(str)),
            _ => Err(Error::new_spanned(
                str,
                r#"expected one of: "wrap", "keep""#,
            )),
        }
    }
}
impl ToTokens for StyleValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            StyleValue::Wrap(wrap) => wrap.to_tokens(tokens),
            StyleValue::Keep(keep) => keep.to_tokens(tokens),
        }
    }
}
pub struct Style {
    pub style: kw::style,
    pub value: Eq<StyleValue>,
}
impl Parse for Style {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            style: input.parse()?,
            value: input.parse()?,
        })
    }
}
impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.style.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
pub struct Simplify {
    pub simplify: kw::simplify,
    pub value: Option<Eq<LitInt>>,
}
impl Parse for Simplify {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            simplify: input.parse()?,
            value: if input.peek(Token![=]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}
impl ToTokens for Simplify {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.simplify.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
