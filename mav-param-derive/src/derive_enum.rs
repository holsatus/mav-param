use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, Ident, Lit, Variant};
use quote::{quote, ToTokens as _};


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
