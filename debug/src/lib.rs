use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);

    let name = parsed_input.ident;
    let quoted_ident = format!(r#"{}"#, name);
    let generics = add_trait_bounds(parsed_input.generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields_impl = fields_implementation(&parsed_input.data);

    let quote = quote! {
        impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.debug_struct(#quoted_ident)
                    #fields_impl
            }
        }
    };

    proc_macro::TokenStream::from(quote)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(fmt::Debug));
        }
    }
    generics
}

fn fields_implementation(data: &Data) -> TokenStream {
    match data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let quoted_ident = format!(r#"{}"#, name.as_ref().unwrap());
                    quote! {
                        .field(#quoted_ident, &self.#name)
                    }
                });
                quote! {
                    #(#recurse)*
                    .finish()
                }
            }
            Fields::Unnamed(ref fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    let quoted_ident = format!(r#"{}"#, i);
                    quote! {
                        .field(#quoted_ident, &self.#i)
                    }
                });
                quote! {
                    #(#recurse)*
                    .finish()
                }
            }
            Fields::Unit => quote! {.finish()},
        },
        _ => unimplemented!(),
    }
}
