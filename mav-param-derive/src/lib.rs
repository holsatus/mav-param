use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, FieldsNamed, Ident, Lit, MetaNameValue, Variant
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
            let condition = find_condition_attr(&field.attrs);
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

                if !matches!(condition_expr, syn::Expr::Binary(_) ) {
                    panic!("The conditional ({}) must be a binary/boolean expression", condition_str)
                }
                
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

            fn entries_full_list(&self) -> &'static [&'static str] {
                &[#(#entry_strings),*]
            }
        }

        impl <'a> mav_param::Node<'a> for #name {
            fn node_ref(&'a self) -> mav_param::NodeRef<'a> {
                mav_param::NodeRef::Tree(self)
            }

            fn node_mut(&'a mut self) -> mav_param::NodeMut<'a> {
                mav_param::NodeMut::Tree(self)
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

// Updated function to extract rename attribute using syn 2.0 API
fn find_condition_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("tree") {
            // Use the parse_args method for more reliable parsing in syn 2.0
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



/// Derives the `Enum` trait for an enum type.
///
/// This macro automatically implements the `Enum` and `Node` traits for enums, enabling
/// them to be used within the parameter tree as tagged unions.
///
/// Each variant must have a single field and a discriminant value.
///
/// Example:
/// ```
/// #[derive(mav_param::Enum)]
/// enum ConfigType {
///     Car(CarConfig) = 0,
///     Plane(PlaneConfig) = 1,
/// }
/// ```
#[proc_macro_derive(Enum, attributes(enum_variant))]
pub fn enum_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Verify it's an enum and extract variants
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("Enum derive only works on enums"),
    };

    // Extract variant information (name, inner type, and discriminant)
    let mut variant_infos = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;
        
        // Get the inner type (must have exactly one field)
        let inner_type = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.iter().next().unwrap().ty
            },
            _ => panic!("Each variant must have exactly one field"),
        };
        
        // Extract the discriminant (required)
        let discriminant = extract_discriminant(variant);
        
        variant_infos.push((variant_name, inner_type, discriminant));
    }

    // In your enum_derive function:
    let value_variant = get_repr_value(&input.attrs);

    // Generate the match arms for each trait method
    let discriminant_arms = variant_infos.iter().map(|(name, _, disc)| {
        quote! {
            Self::#name(_) => #value_variant(#disc),
        }
    });

    let set_discriminant_arms = variant_infos.iter().map(|(name, _, disc)| {
        quote! {
            #value_variant(#disc) => {
                *self = Self::#name(Default::default());
            }
        }
    });

    // Generate the match arms for each trait method
    let discriminant_list = variant_infos.iter().map(|(_, _, disc)| {
        quote! { 
            #value_variant(#disc),
        }
    });

    let active_variant_ref_arms = variant_infos.iter().map(|(name, _, _)| {
        quote! {
            Self::#name(inner) => inner.node_ref(),
        }
    });

    let active_variant_mut_arms = variant_infos.iter().map(|(name, _, _)| {
        quote! {
            Self::#name(inner) => inner.node_mut(),
        }
    });

    // Generate the implementations
    let output = quote! {
        impl<'a> mav_param::Enum<'a> for #name {
            fn discriminant(&self) -> mav_param::Value {
                match self {
                    #(#discriminant_arms)*
                }
            }
            
            fn set_discriminant(&mut self, disc: mav_param::Value) {
                match disc {
                    #(#set_discriminant_arms)*
                    _ => (), // No change if discriminant is invalid
                }
            }

            fn discriminants_list(&self) -> &'static [mav_param::Value] {
                &[ #(#discriminant_list)* ]
            }
            
            fn active_node_ref(&'a self) -> mav_param::NodeRef<'a> {
                use crate::Node;
                match self {
                    #(#active_variant_ref_arms)*
                }
            }
            
            fn active_node_mut(&'a mut self) -> mav_param::NodeMut<'a> {
                use crate::Node;
                match self {
                    #(#active_variant_mut_arms)*
                }
            }
        }

        impl<'a> mav_param::Node<'a> for #name {
            fn node_ref(&'a self) -> mav_param::NodeRef<'a> {
                mav_param::NodeRef::Enum(self)
            }
            
            fn node_mut(&'a mut self) -> mav_param::NodeMut<'a> {
                mav_param::NodeMut::Enum(self)
            }
        }
    };

    TokenStream::from(output)
}

// Helper function to extract the discriminant value from a variant
fn extract_discriminant(variant: &Variant) -> proc_macro2::TokenStream {
    variant.discriminant.as_ref().map(|(_, expr)| {
        match expr {
            Expr::Lit(expr) => {
                if let Lit::Int(ref lit_int) = expr.lit {
                    return lit_int.to_token_stream();
                } else {
                    panic!("Discriminant must be a literal integer")
                }
            },
            _ => panic!("Discriminant must be a literal integer"),
        }
    }).unwrap()
}

// Helper function to extract the enum representation type
fn get_repr_value(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("repr") {
            // Use parse_args_with to parse the content
            if let Ok(content) = attr.parse_args_with(|parser: &syn::parse::ParseBuffer<'_>| {
                // Parse a single identifier
                let ident: Ident = parser.parse()?;
                Ok(ident.to_string())
            }) {
                // Return the corresponding Value variant
                match content.as_str() {
                    "u8" => return quote! { mav_param::Value::U8 },
                    "i8" => return quote! { mav_param::Value::I8 },
                    "u16" => return quote! { mav_param::Value::U16 },
                    "i16" => return quote! { mav_param::Value::I16 },
                    "u32" => return quote! { mav_param::Value::U32 },
                    "i32" => return quote! { mav_param::Value::I32 },
                    _ => panic!("Unsupported repr type: {}", content)
                }
            }
        }
    }
    
    // If we got here, no repr attribute was found
    panic!("Enum must have a #[repr(inttype)] attribute specifying a supported primitive type")
}
