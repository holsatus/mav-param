use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, FieldsNamed, Ident, Lit, MetaNameValue,
};

/// Derives the `Tree` trait for a struct.
///
/// This macro automatically implements the `Tree` trait for a struct, allowing it
/// to be part of a parameter hierarchy. Each field of the struct becomes an entry
/// in the parameter tree.
///
/// Use the `#[tree(rename = "name")]` attribute to customize field names in the tree.
#[proc_macro_derive(Tree, attributes(tree))]
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
            let renamed = find_rename_attr(&field.attrs);
            let param_name = renamed.unwrap_or_else(|| field_name_str.clone());

            (field_name, param_name)
        })
        .collect::<Vec<_>>();

    // Generate match arms for get_ref
    let get_ref_arms = field_info.iter().map(|(field_name, param_name)| {
        quote! {
            #param_name => Some(self.#field_name.node_ref()),
        }
    });

    // Generate match arms for get_mut
    let get_mut_arms = field_info.iter().map(|(field_name, param_name)| {
        quote! {
            #param_name => Some(self.#field_name.node_mut()),
        }
    });

    // Generate entries list
    let entry_strings = field_info.iter().map(|(_, param_name)| {
        quote! { #param_name }
    });

    quote! {
        impl mav_param::Tree for #name {
            fn get_ref<'a>(&'a self, node: &str) -> Option<mav_param::NodeRef<'a>> {
                use mav_param::Node;
                match node {
                    #(#get_ref_arms)*
                    _ => None,
                }
            }

            fn get_mut<'a>(&'a mut self, node: &str) -> Option<mav_param::NodeMut<'a>> {
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
    }
}

// Updated function to extract rename attribute using syn 2.0 API
fn find_rename_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("tree") {
            // Use the parse_args method for more reliable parsing in syn 2.0
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

/// Derives the `Node` trait for a newtype struct.
///
/// This macro generates an implementation that delegates to the inner type's `Node`
/// implementation. It only works on newtype structs (structs with a single unnamed field).
///
/// Example:
/// ```
/// #[derive(Node)]
/// struct MyWrapper(u8);
/// ```
#[proc_macro_derive(Node)]
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
        impl mav_param::Node for #name {
            fn node_ref(&self) -> mav_param::NodeRef<'_> {
                self.0.node_ref()
            }

            fn node_mut(&mut self) -> mav_param::NodeMut<'_> {
                self.0.node_mut()
            }
        }
    }
    .into()
}
