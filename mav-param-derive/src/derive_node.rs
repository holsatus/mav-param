use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, Fields};
use quote::quote;

pub fn node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Verify it's a newtype struct
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Valid newtype struct
            }
            _ => panic!("Node derive only works on newtype structs with a single field"),
        },
        _ => panic!("Node derive only works on structs"),
    };

    // Delegate to the Node trait implementation of the inner type
    quote! {
        impl <'a> mav_param::Node<'a> for #name {
            fn node_ref(&self) -> mav_param::NodeRef<'a> {
                self.0.node_ref()
            }

            fn node_mut(&mut self) -> mav_param::NodeMut<'a> {
                self.0.node_mut()
            }
        }
    }
    .into()
}
