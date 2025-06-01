use proc_macro::TokenStream;
use syn::{parse::ParseBuffer, parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, Ident, Lit, LitInt, MetaNameValue, Variant};
use quote::{quote, ToTokens};

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
    let variant_infos = variants.iter().map(|variant| {
        
        // Get the inner type (must have exactly one field)
        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => (),
            _ => panic!("Each variant must have exactly one field"),
        };
        
        // Extract the discriminant (required)
        let discriminant = get_discriminant(variant);
        let default_opt = find_default_attr(&variant.attrs);
        
        (variant.ident.clone(), default_opt, discriminant)
    }).collect::<Vec<_>>();

    let value_variant = get_repr_value(&input.attrs);

    let discriminant_arms = variant_infos.iter().map(|(name, _, disc)| {
        quote! { Self::#name(_) => #value_variant(#disc), }
    });

    let set_discriminant_arms = variant_infos.iter().map(|(name, default, disc)| {
        match default {
            Some(default) => quote! { #value_variant(#disc) => *self = Self::#name(#default), },
            _ => quote! { #value_variant(#disc) => *self = Self::#name(Default::default()), },
        }
    });

    let discriminant_list = variant_infos.iter().map(|(_, _, disc)| {
        quote! { #value_variant(#disc), }
    });

    let active_variant_ref_arms = variant_infos.iter().map(|(name, _, _)| {
        quote! { Self::#name(inner) => inner.node_ref(), }
    });

    let active_variant_mut_arms = variant_infos.iter().map(|(name, _, _)| {
        quote! { Self::#name(inner) => inner.node_mut(), }
    });

    // Generate the implementations
    let output = quote! {
        impl<'a> mav_param::Enum<'a> for #name {
            fn discriminants(&self) -> &'static [mav_param::Value] {
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

        impl mav_param::Leaf for #name {
            fn get(&self) -> mav_param::Value {
                match self {
                   #(#discriminant_arms)*
                }
            }

            fn set(&mut self, val: mav_param::Value) -> bool {
                match val {
                   #(#set_discriminant_arms)*
                   _ => return false,
                }
                return true;
            }
        }
    };

    TokenStream::from(output)
}

// Helper function to extract the discriminant value from a variant
fn get_discriminant(variant: &Variant) -> LitInt {

    if let Expr::Lit(ref expr_lit) = variant.discriminant.as_ref().unwrap().1 {
        if let Lit::Int(ref lit_int) = expr_lit.lit {
            return lit_int.clone()
        };
    };

    panic!("Each variant must have an explicit discriminant defined")
}

// Helper function to extract the enum representation type
fn get_repr_value(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("repr") {
            // Use parse_args_with to parse the content
            if let Ok(content) = attr.parse_args_with(|parser: &ParseBuffer<'_>| {
                Ok(parser.parse::<Ident>()?.to_string())
            }) {
                // Return the corresponding Value variant
                return match content.as_str() {
                    "u8" => quote! { mav_param::Value::U8 },
                    "i8" => quote! { mav_param::Value::I8 },
                    "u16" => quote! { mav_param::Value::U16 },
                    "i16" => quote! { mav_param::Value::I16 },
                    "u32" => quote! { mav_param::Value::U32 },
                    "i32" => quote! { mav_param::Value::I32 },
                    _ => panic!("Unsupported repr type: {}, must be a mav_param::Value variant", content)
                }
            }
        }
    }
    
    // If we got here, no repr attribute was found
    panic!("Enum must have a #[repr(inttype)] attribute specifying a mav_param::Value variant")
}

// Updated function to extract rename attribute using syn 2.0 API
fn find_default_attr(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("param") {
            // Use the parse_args method for more reliable parsing in syn 2.0
            let meta = match attr.parse_args::<MetaNameValue>() {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            if meta.path.is_ident("default") {
                return Some(meta.value.to_token_stream())
            }
        }
    }
    None
}