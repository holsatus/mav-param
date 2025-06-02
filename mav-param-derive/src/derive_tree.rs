use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, FieldsNamed, Ident, Lit, MetaNameValue};
use quote::quote;

pub fn tree_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate implementation based on struct type
    let implementation = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            // Handle normal structs with named fields
            Fields::Named(fields_named) => generate_named_fields_impl(name, fields_named),

            _ => panic!("Tree derive only supports structs with named fields"),
        },
        _ => panic!("Tree derive only supports structs"),
    };

    // Return the generated code
    TokenStream::from(implementation)
}

// Generate implementation for struct with named fields
fn generate_named_fields_impl(
    name: &Ident,
    fields_named: &FieldsNamed,
) -> proc_macro2::TokenStream {
    // Collect field processing info
    let field_info = fields_named
        .named
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();

            // Check for rename attribute
            let renamed = get_rename_attr(&field.attrs);
            let condition = get_condition_attr(&field.attrs);
            let param_name = renamed.unwrap_or_else(|| field_name_str.clone());

            (field_name, condition, param_name)
        })
        .collect::<Vec<_>>();

    // Generate match arms for get_ref
    let get_ref_arms = field_info.iter().map(|(field_name, condition, param_name)| {
        match condition {
            Some(condition_str) => {
                // Parse the condition string into a token stream
                let condition_expr = syn::parse_str::<syn::Expr>(&condition_str)
                    .expect("Failed to parse condition expression");
                
                quote! {
                    #param_name if #condition_expr => Some(self.#field_name.node_ref()),
                }
            },
            None => quote! {
                #param_name => Some(self.#field_name.node_ref()),
            },
        }
    });

    // Generate match arms for get_mut
    let get_mut_arms = field_info.iter().map(|(field_name, condition, param_name)| {
        match condition {
            Some(condition_str) => {
                // Parse the condition string into a token stream
                let condition_expr = syn::parse_str::<syn::Expr>(&condition_str)
                    .expect("Failed to parse condition expression");

                quote! {
                    #param_name if #condition_expr => Some(self.#field_name.node_mut()),
                }
            },
            None => quote! {
                #param_name => Some(self.#field_name.node_mut()),
            },
        }
    });

    // Generate entries list
    let entry_strings = field_info.iter().map(|(_, _, param_name)| {
        quote! { #param_name }
    });

    quote! {
        impl <'a> mav_param::Tree<'a> for #name {
            fn get_ref(&'a self, node: &str) -> Option<mav_param::NodeRef<'a>> {
                use mav_param::Node;
                match node {
                    #(#get_ref_arms)*
                    _ => None,
                }
            }

            fn get_mut(&'a mut self, node: &str) -> Option<mav_param::NodeMut<'a>> {
                use mav_param::Node;
                match node {
                    #(#get_mut_arms)*
                    _ => None,
                }
            }

            fn entries(&self) -> &'static [&'static str] {
                &[#(#entry_strings),*]
            }
        }

        impl mav_param::Node for #name {
            fn node_ref(&self) -> mav_param::NodeRef<'_> {
                mav_param::NodeRef::Tree(self)
            }

            fn node_mut(&mut self) -> mav_param::NodeMut<'_> {
                mav_param::NodeMut::Tree(self)
            }
        }
    }
}

fn get_rename_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("param") {
            let meta = match attr.parse_args::<MetaNameValue>() {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            if meta.path.is_ident("rename") {
                let syn::Expr::Lit(lit) = meta.value else {
                    return None;
                };

                if let Lit::Str(lit_str) = lit.lit {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

fn get_condition_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("param") {
            let meta = match attr.parse_args::<MetaNameValue>() {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            if meta.path.is_ident("condition") {
                let syn::Expr::Lit(lit) = meta.value else {
                    return None;
                };

                if let Lit::Str(lit_str) = lit.lit {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}