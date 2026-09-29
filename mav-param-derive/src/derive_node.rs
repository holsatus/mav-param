use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match node_derive_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn node_derive_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Node derive does not support generic types",
        ));
    }

    // Verify it's a newtype struct
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Node derive only works on newtype structs with a single field",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Node derive only works on structs",
            ));
        }
    };

    // Delegate to the Node trait implementation of the inner type
    Ok(quote! {
        impl mav_param::Node for #name {
            fn node_ref(&self) -> mav_param::NodeRef<'_> {
                self.0.node_ref()
            }

            fn node_mut(&mut self) -> mav_param::NodeMut<'_> {
                self.0.node_mut()
            }
        }
    })
}
