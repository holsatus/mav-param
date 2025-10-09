use proc_macro::TokenStream;
use syn::{parse::ParseBuffer, parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, Ident, Lit, LitInt, MetaNameValue, Variant};
use quote::{quote, ToTokens};
enum VariantType {
    Unit,
    UnnamedSingle
}

pub fn enum_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Verify it's an enum and extract variants
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("Enum derive only works on enums"),
    };

    let mut discriminant = 0;
    
    // Each variants match arm in the implementations
    let mut discriminant_list = Vec::with_capacity(variants.len());
    let mut active_variant_ref = Vec::with_capacity(variants.len());
    let mut active_variant_mut = Vec::with_capacity(variants.len());
    let mut get_discriminant = Vec::with_capacity(variants.len());
    let mut set_discriminant = Vec::with_capacity(variants.len());

    for variant in variants {
        
        let name = &variant.ident;
        let value_variant = get_repr_mav_param_value(&input.attrs);

        let variant_type = match &variant.fields {
            Fields::Unit => VariantType::Unit,
            Fields::Unnamed(f) if f.unnamed.len() == 1 => VariantType::UnnamedSingle,
            _ => panic!("Each variant must either have no fields or exactly one field"),
        };

        if let Some(literal) = get_discriminant_literal(variant) {
            discriminant = literal.base10_parse().expect("Literal is not valid base 10");
        }

        let discriminant_tokens = discriminant.to_string().parse::<proc_macro2::TokenStream>()
            .expect("Failed to turn discriminant into token stream");

        discriminant_list.push(quote! {
            #value_variant(#discriminant_tokens),
        });

        active_variant_ref.push(match &variant_type {
            VariantType::Unit => quote! {
                Self::#name => mav_param::NodeRef::None,
            },
            VariantType::UnnamedSingle => quote! {
                Self::#name(inner) => inner.node_ref(),
            },
        });

        active_variant_mut.push(match &variant_type {
            VariantType::Unit => quote! {
                Self::#name => mav_param::NodeMut::None,
            },
            VariantType::UnnamedSingle => quote! {
                Self::#name(inner) => inner.node_mut(),
            },
        });

        get_discriminant.push(match &variant_type {
            VariantType::Unit => quote! {
                Self::#name => #value_variant(#discriminant_tokens),
            },
            VariantType::UnnamedSingle => quote! {
                Self::#name(_) => #value_variant(#discriminant_tokens),
            },
        });

        set_discriminant.push(match (&variant_type, get_default_attr(&variant.attrs)) {
            (VariantType::Unit, _) => quote! { 
                #value_variant(#discriminant_tokens) => *self = Self::#name, 
            },
            (VariantType::UnnamedSingle, None) => quote! { 
                #value_variant(#discriminant_tokens) => *self = Self::#name(Default::default()), 
            },
            (VariantType::UnnamedSingle, Some(default)) => quote! { 
                #value_variant(#discriminant_tokens) => *self = Self::#name(#default), 
            },
        });

        discriminant += 1;
    }

    // Generate the implementations
    let output = quote! {
        impl mav_param::Node for #name {
            fn node_ref(&self) -> mav_param::NodeRef<'_> {
                mav_param::NodeRef::Enum(self)
            }
            
            fn node_mut(&mut self) -> mav_param::NodeMut<'_> {
                mav_param::NodeMut::Enum(self)
            }
        }

        impl<'a> mav_param::Enum<'a> for #name {
            fn discriminants(&self) -> &'static [mav_param::Value] {
                &[ #(#discriminant_list)* ]
            }
            
            fn active_node_ref(&'a self) -> mav_param::NodeRef<'a> {
                use mav_param::Node;
                match self {
                    #(#active_variant_ref)*
                }
            }
            
            fn active_node_mut(&'a mut self) -> mav_param::NodeMut<'a> {
                use mav_param::Node;
                match self {
                    #(#active_variant_mut)*
                }
            }
        }

        impl mav_param::Leaf for #name {
            fn get(&self) -> mav_param::Value {
                match self {
                   #(#get_discriminant)*
                }
            }

            fn set(&mut self, val: mav_param::Value) -> bool {
                match val {
                   #(#set_discriminant)*
                   _ => return false,
                }
                return true;
            }
        }
    };

    TokenStream::from(output)
}

// Helper function to extract the discriminant value from a variant
fn get_discriminant_literal(variant: &Variant) -> Option<LitInt> {
    if let Expr::Lit(ref expr_lit) = variant.discriminant.as_ref()?.1 {
        if let Lit::Int(ref lit_int) = expr_lit.lit {
            return Some(lit_int.clone())
        };
    };

    None
}

// Helper function to extract the enum representation type
fn get_repr_mav_param_value(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("repr") {
            if let Ok(content) = attr.parse_args_with(|parser: &ParseBuffer<'_>| {
                Ok(parser.parse::<Ident>()?.to_string())
            }) {
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
    
    panic!("Enum must have a #[repr(inttype)] attribute")
}

fn get_default_attr(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("param") {
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