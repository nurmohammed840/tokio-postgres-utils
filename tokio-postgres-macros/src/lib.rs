use quote2::{proc_macro2::TokenStream, quote, Quote};
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
#[proc_macro_derive(FromRow)]
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
                            let raw_str = name.to_string();
                            quote!(tokens, {
                                #name: r.get(#raw_str),
                            });
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
#[proc_macro_derive(TryFromRow)]
pub fn try_from_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = quote(|tokens| match input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let body = quote(|tokens| {
                    for field in &fields.named {
                        if let Some(name) = &field.ident {
                            let raw_str = name.to_string();
                            quote!(tokens, {
                                #name: r.try_get(#raw_str)?,
                            });
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

    let mut tokens = TokenStream::new();
    quote!(tokens, {
        impl #impl_generics ::std::convert::TryFrom<&tokio_postgres::Row> for #name #ty_generics #where_clause {
            type Error = tokio_postgres::Error;
            #[inline]
            fn try_from(r: &tokio_postgres::Row) -> ::std::result::Result<Self, Self::Error> {
                Ok(Self #body)
            }
        }
    });
    tokens.into()
}
