use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, FieldsNamed, Ident, Lit, MetaNameValue,
};

#[proc_macro_derive(Tree, attributes(node))]
pub fn param_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate implementation based on struct type
    let implementation = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            // Handle normal structs with named fields
            Fields::Named(fields_named) => generate_named_fields_impl(name, fields_named),

            // Handle unit/newtype structs (single unnamed field)
            Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => {
                generate_unit_struct_impl(name)
            }

            _ => panic!(
                "Tree derive only supports structs with named fields or unit structs with a single field"
            ),
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

    // Generate match arms for get_ref using the renamed fields
    let get_ref_arms = field_info.iter().map(|(field_name, param_name)| {
        quote! {
            #param_name => Some(self.#field_name.as_either_ref()),
        }
    });

    // Generate match arms for get_mut using the renamed fields
    let get_mut_arms = field_info.iter().map(|(field_name, param_name)| {
        quote! {
            #param_name => Some(self.#field_name.as_either_mut()),
        }
    });

    // Generate entries list with renamed fields
    let entry_strings = field_info.iter().map(|(_, param_name)| {
        quote! { #param_name }
    });

    quote! {
        impl ::param_rs::Tree for #name {
            fn get_ref<'a>(&'a self, path: &str) -> Option<::param_rs::EitherRef<'a>> {
                use ::param_rs::IntoEither;
                match path {
                    #(#get_ref_arms)*
                    _ => None,
                }
            }

            fn get_mut<'a>(&'a mut self, path: &str) -> Option<::param_rs::EitherMut<'a>> {
                use ::param_rs::IntoEither;
                match path {
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

fn generate_unit_struct_impl(name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl ::param_rs::Tree for #name {
            fn get_ref<'a>(&'a self, path: &str) -> Option<::param_rs::EitherRef<'a>> {
                use ::param_rs::IntoEither;
                if path.is_empty() {
                    // For empty path, return the field as EitherRef
                    Some(self.0.as_either_ref())
                } else {
                    // We can't forward calls to Tree methods on the inner field
                    // since it only implements IntoEither
                    None
                }
            }

            fn get_mut<'a>(&'a mut self, path: &str) -> Option<::param_rs::EitherMut<'a>> {
                use ::param_rs::IntoEither;
                if path.is_empty() {
                    // For empty path, return the field as EitherMut
                    Some(self.0.as_either_mut())
                } else {
                    None
                }
            }

            fn entries(&self) -> &'static [&'static str] {
                // Unit structs with IntoEither fields have no entries
                &[]
            }
        }
    }
}

// Updated function to extract rename attribute using syn 2.0 API
fn find_rename_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("param") {
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
