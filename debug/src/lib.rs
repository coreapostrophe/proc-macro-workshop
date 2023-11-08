use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Meta};

#[proc_macro_derive(CustomDebug, attributes(debug))]
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
    
    // eprintln!("{}", proc_macro::TokenStream::from(quote!(#generics)));

    proc_macro::TokenStream::from(quote)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(std::fmt::Debug));
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
                    let attr = {
                        let mut result: Option<TokenStream> = None;
                        for attr in &f.attrs {
                            let meta = &attr.meta;
                            if let Meta::NameValue(name_value) = meta {
                                let value = &name_value.value;
                                result = Some(quote! {
                                    std::format_args!(#value, &self.#name)
                                });
                            }
                        }
                        result
                    };
                    let quoted_ident = format!(r#"{}"#, name.as_ref().unwrap());
                    
                    match attr {
                        Some(attr_stream) => quote! {
                            .field(#quoted_ident, &#attr_stream)
                        },
                        None => quote! {
                            .field(#quoted_ident, &self.#name)
                        }
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
