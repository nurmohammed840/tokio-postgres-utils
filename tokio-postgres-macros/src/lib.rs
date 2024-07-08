use std::cell::Cell;
use quote2::{proc_macro2::{TokenStream, TokenTree, Literal}, quote, Quote};
use syn::*;

/// Implements `From<&Row>` trait for a struct, allowing direct conversion from a database row to the struct.
///
/// ## Example
///
/// ```rust
/// use tokio_postgres_utils::FromRow;
///
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     name: String,
/// }
/// ```
///
/// Expand into:
///
/// ```
/// impl From<&Row> for User {
///     fn from(row: &Row) -> Self {
///         Self {
///             id: row.get("id"),
///             name: row.get("name"),
///         }
///     }
/// }
/// ```
#[proc_macro_derive(FromRow, attributes(column))]
pub fn from_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = quote(|tokens| match input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let body = quote(|tokens| {
                    for field in &fields.named {
                        if let Some(name) = &field.ident {
                            quote!(tokens, { #name: });
                            match column_attr(&field.attrs) {
                                ColumnAttr::Flatten => {
                                    quote!(tokens, {
                                        ::std::convert::TryFrom::try_from(r).unwrap(),
                                    });
                                }
                                ColumnAttr::Rename(rename) => {
                                    quote!(tokens, {
                                        r.get(#rename),
                                    });
                                },
                                ColumnAttr::None => {
                                    let raw_str = name.to_string();
                                    quote!(tokens, {
                                        r.get(#raw_str),
                                    });
                                }
                                ColumnAttr::Skip => {
                                    quote!(tokens, {
                                        ::std::default::Default::default(),
                                    });
                                }
                            }
                        }
                    }
                });
                quote!(tokens, {
                    { #body }
                });
            }
            Fields::Unnamed(fields) => {
                let body = quote(|tokens| {
                    for (i, _) in fields.unnamed.iter().enumerate() {
                        let idx = Index::from(i);
                        quote!(tokens, {
                            r.get(#idx),
                        });
                    }
                });
                quote!(tokens, {
                    (#body)
                });
            }
            Fields::Unit => {}
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    });

    let mut tokens = TokenStream::new();
    quote!(tokens, {
        impl #impl_generics ::std::convert::From<&tokio_postgres::Row> for #name #ty_generics #where_clause {
            #[inline]
            fn from(r: &tokio_postgres::Row) -> Self {
                Self #body
            }
        }
    });
    tokens.into()
}

/// Implements the `TryFrom<&Row>` trait for a struct
#[proc_macro_derive(TryFromRow, attributes(column))]
pub fn try_from_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let has_attr = Cell::new(false);

    let body = quote(|tokens| match input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let body = quote(|tokens| {
                    for field in &fields.named {
                        if let Some(name) = &field.ident {
                            quote!(tokens, { #name: });
                            match column_attr(&field.attrs) {
                                ColumnAttr::Flatten => {
                                    has_attr.set(true);
                                    quote!(tokens, {
                                        ::std::convert::TryFrom::try_from(r)?,
                                    });
                                }
                                ColumnAttr::Rename(rename) => {
                                    quote!(tokens, {
                                        r.try_get(#rename)?,
                                    });
                                },
                                ColumnAttr::None => {
                                    let raw_str = name.to_string();
                                    quote!(tokens, {
                                        r.try_get(#raw_str)?,
                                    });
                                }
                                ColumnAttr::Skip => {
                                    quote!(tokens, {
                                        ::std::default::Default::default(),
                                    });
                                }
                            }
                        }
                    }
                });
                quote!(tokens, {
                    { #body }
                });
            }
            Fields::Unnamed(fields) => {
                let body = quote(|tokens| {
                    for (i, _) in fields.unnamed.iter().enumerate() {
                        let idx = Index::from(i);
                        quote!(tokens, {
                            r.try_get(#idx)?,
                        });
                    }
                });
                quote!(tokens, {
                    (#body)
                });
            }
            Fields::Unit => {}
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    });

    let err_ty = quote(|t| {
        if has_attr.get() {
            quote!(t, { ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync> });
        } else {
            quote!(t, { tokio_postgres::Error });
        }
    });

    let mut tokens = TokenStream::new();
    quote!(tokens, {
        impl #impl_generics ::std::convert::TryFrom<&tokio_postgres::Row> for #name #ty_generics #where_clause {
            #[inline]
            fn try_from(r: &tokio_postgres::Row) -> ::std::result::Result<Self, Self::Error> {
                Ok(Self #body)
            }
            type Error = #err_ty;
        }
    });
    tokens.into()
}



enum ColumnAttr {
    Skip,
    Flatten,
    None,
    Rename(Literal),
}

fn column_attr(attrs: &[Attribute]) -> ColumnAttr {
    attrs
        .iter()
        .find_map(|attr| {
            if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta {
                if path.segments.first()?.ident == "column" {
                    let mut tokens = tokens.clone().into_iter();
                    match tokens.next()?.to_string().as_str() {
                        "skip" => return Some(ColumnAttr::Skip),
                        "flatten" => return Some(ColumnAttr::Flatten),
                        "rename" => {
                            if matches!(tokens.next()?, TokenTree::Punct(p) if p.as_char() == '=') {
                                if let TokenTree::Literal(lit) = tokens.next()? {
                                    return Some(ColumnAttr::Rename(lit));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            None
        })
        .unwrap_or(ColumnAttr::None)
}