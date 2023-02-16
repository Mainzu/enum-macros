struct DebugWrapper<'a, T>(pub &'a T);

impl<'a, T> Debug for DebugWrapper<'a, Option<T>>
where
    DebugWrapper<'a, T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(val) => write!(f, "Some({:#?})", DebugWrapper(val)),
            None => write!(f, "None"),
        }
    }
}

// impl<T: ToTokens> Debug for DebugWrapper<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0.to_token_stream().to_string())
//     }
// }
impl<'a> Debug for DebugWrapper<'a, Colon2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Colon2")
    }
}
impl<'a> Debug for DebugWrapper<'a, Comma> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Comma")
    }
}
impl<'a, T, P> Debug for DebugWrapper<'a, Pair<T, P>>
where
    DebugWrapper<'a, T>: Debug,
    DebugWrapper<'a, P>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Pair::Punctuated(t, p) => f
                .debug_tuple("Punctuated")
                .field(&DebugWrapper(t))
                .field(&DebugWrapper(p))
                .finish(),
            Pair::End(t) => f.debug_tuple("End").field(&DebugWrapper(t)).finish(),
        }
    }
}
impl<'a, 'b, T> Debug for DebugWrapper<'a, &'b T>
where
    DebugWrapper<'b, T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", DebugWrapper(*self.0))
    }
}

impl<'a, T, P> Debug for DebugWrapper<'a, Punctuated<T, P>>
where
    DebugWrapper<'a, T>: Debug,
    DebugWrapper<'a, P>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pairs = self.0.pairs().collect::<Vec<_>>();
        f.debug_list()
            .entries(pairs.iter().map(|p| DebugWrapper(p)))
            .finish()
        // write!(f, "Punctuated[")?;
        // for p in self.0.pairs() {
        //     write!(f, "{:#?}", DebugWrapper(&p))?;
        // }
        // write!(f, "]")
    }
}
impl<'a> Debug for DebugWrapper<'a, PathArguments> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            PathArguments::None => write!(f, "None"),
            PathArguments::AngleBracketed(a) => f
                .debug_tuple("AngleBracketed")
                .field(&DebugWrapper(a))
                .finish(),
            PathArguments::Parenthesized(a) => f
                .debug_tuple("Parenthesized")
                .field(&DebugWrapper(&a))
                .finish(),
        }
        // write!(f, "{}", self.0.to_token_stream().to_string())
    }
}
impl<'a> Debug for DebugWrapper<'a, Ident> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}
impl<'a> Debug for DebugWrapper<'a, Lifetime> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}", self.0.ident.to_string())
    }
}
impl<'a> Debug for DebugWrapper<'a, GenericArgument> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            GenericArgument::Lifetime(a) => {
                f.debug_tuple("Lifetime").field(&DebugWrapper(a)).finish()
            }
            GenericArgument::Type(a) => write!(f, "Type({:?})", DebugWrapper(a)),
            GenericArgument::Const(a) => write!(f, "Const"),
            GenericArgument::Binding(a) => write!(f, "Binding"),
            GenericArgument::Constraint(a) => write!(f, "Constraint"),
        }
    }
}
impl<'a> Debug for DebugWrapper<'a, Type> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_token_stream().to_string())
    }
}
impl<'a> Debug for DebugWrapper<'a, AngleBracketedGenericArguments> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AngleBracketedGenericArguments")
            .field("args", &DebugWrapper(&self.0.args))
            .finish()
    }
}
impl<'a> Debug for DebugWrapper<'a, ReturnType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ReturnType::Default => write!(f, "Default"),
            ReturnType::Type(_, t) => write!(f, "Type({:?})", DebugWrapper(t.as_ref())),
        }
    }
}
impl<'a> Debug for DebugWrapper<'a, ParenthesizedGenericArguments> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self.0.
        f.debug_struct("ParenthesizedGenericArguments")
            .field("inputs", &DebugWrapper(&self.0.inputs))
            .field("output", &DebugWrapper(&self.0.output))
            .finish()
    }
}
impl<'a> Debug for DebugWrapper<'a, PathSegment> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathSegment")
            .field("ident", &self.0.ident.to_string())
            .field("arguments", &DebugWrapper(&self.0.arguments))
            .finish()
    }
}
impl<'a> Debug for DebugWrapper<'a, Path> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Path")
            .field("leading_colon", &self.0.leading_colon.is_some())
            .field("segments", &DebugWrapper(&self.0.segments))
            .finish()

        // if self.0.leading_colon.is_some() {
        //     write!(f, "::")?;
        // }
        // for seg in &self.0.segments {
        //     write!(f, "{}", seg.ident.to_string())?;
        //     match &seg.arguments {
        //         syn::PathArguments::None => {}
        //         syn::PathArguments::AngleBracketed(a) => {
        //             write!(f, "{}", a.to_token_stream().to_string())?;
        //         }
        //         syn::PathArguments::Parenthesized(a) => {
        //             write!(f, "{}", a.to_token_stream().to_string())?;
        //         }
        //     }
        // }
        // Ok(())
    }
}
impl<'a> Debug for DebugWrapper<'a, NestedMeta> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            NestedMeta::Meta(meta) => write!(f, "Meta({:#?})", DebugWrapper(meta)),
            NestedMeta::Lit(lit) => write!(f, "Lit({:#?})", DebugWrapper(lit)),
        }
    }
}

impl<'a> Debug for DebugWrapper<'a, MetaList> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self.0.
        f.debug_struct("MetaList")
            .field("path", &DebugWrapper(&self.0.path))
            .field("nested", &DebugWrapper(&self.0.nested))
            .finish()

        // write!(f, "{:#?}(", DebugWrapper(&self.0.path))?;
        // let mut first = true;
        // for nested in &self.0.nested {
        //     if !first {
        //         write!(f, ", ")?;
        //     } else {
        //         first = false;
        //     }
        //     match nested {
        //         syn::NestedMeta::Meta(meta) => write!(f, "{:#?}", DebugWrapper(meta))?,
        //         syn::NestedMeta::Lit(lit) => write!(f, "{:#?}", DebugWrapper(lit))?,
        //     }
        // }
        // if self.0.nested.trailing_punct() {
        //     write!(f, ", ")?;
        // }
        // write!(f, ")")
    }
}
impl<'a> Debug for DebugWrapper<'a, MetaNameValue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetaNameValue")
            .field("path", &DebugWrapper(&self.0.path))
            .field("lit", &DebugWrapper(&self.0.lit))
            .finish()
    }
}
impl<'a> Debug for DebugWrapper<'a, Meta> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Meta::Path(a) => write!(f, "Path({:#?})", DebugWrapper(a)),
            Meta::List(a) => write!(f, "List({:#?})", DebugWrapper(a)),
            Meta::NameValue(a) => write!(f, "NamedValue({:#?})", DebugWrapper(a)),
        }
    }
}
impl<'a> Debug for DebugWrapper<'a, Lit> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Lit::Str(str) => write!(f, r#""{}""#, str.value()),
            Lit::ByteStr(bstr) => write!(f, r#"b{:#?}"#, bstr.value()),
            Lit::Byte(byte) => write!(f, "b'{}'", byte.value() as char),
            Lit::Char(ch) => write!(f, "'{}'", ch.value()),
            Lit::Int(int) => write!(f, "{}", int.base10_digits()),
            Lit::Float(float) => write!(f, "{}", float.base10_digits()),
            Lit::Bool(bool) => write!(f, "{}", bool.value),
            Lit::Verbatim(ver) => write!(f, "{}", ver.to_string()),
        }
    }
}
