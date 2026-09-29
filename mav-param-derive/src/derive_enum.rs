use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Attribute, Data, DeriveInput,
    Expr, ExprLit, Fields, Ident, Lit, LitInt, Token,
};

use crate::attrs::parse_param_attrs;

enum VariantType {
    Unit,
    UnnamedSingle,
}

pub fn enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match enum_derive_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn enum_derive_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Enum derive does not support generic types",
        ));
    }

    // Verify it's an enum and extract variants
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Enum derive only works on enums",
            ));
        }
    };

    // All variants must share a single primitive representation
    let value_variant = get_repr_mav_param_value(&input.attrs, input.span())?;

    // Start at zero, or at the explicitly given discriminant
    let mut discriminant: i64 = 0;

    // Each variant's match arm in the implementations
    let mut discriminant_list = Vec::with_capacity(variants.len());
    let mut active_variant_ref = Vec::with_capacity(variants.len());
    let mut active_variant_mut = Vec::with_capacity(variants.len());
    let mut get_discriminant = Vec::with_capacity(variants.len());
    let mut set_discriminant = Vec::with_capacity(variants.len());

    for variant in variants {
        let variant_name = &variant.ident;

        let variant_type = match &variant.fields {
            Fields::Unit => VariantType::Unit,
            Fields::Unnamed(f) if f.unnamed.len() == 1 => VariantType::UnnamedSingle,
            _ => {
                return Err(syn::Error::new_spanned(
                    &variant.fields,
                    "Each variant must either have no fields or exactly one field",
                ));
            }
        };

        if let Some((_, expr)) = &variant.discriminant {
            discriminant = parse_discriminant(expr)?;
        }

        let discriminant_tokens = discriminant
            .to_string()
            .parse::<proc_macro2::TokenStream>()
            .expect("integer literal is always a valid token stream");

        discriminant_list.push(quote! {
            #value_variant(#discriminant_tokens),
        });

        active_variant_ref.push(match &variant_type {
            VariantType::Unit => quote! {
                Self::#variant_name => mav_param::NodeRef::None,
            },
            VariantType::UnnamedSingle => quote! {
                Self::#variant_name(inner) => inner.node_ref(),
            },
        });

        active_variant_mut.push(match &variant_type {
            VariantType::Unit => quote! {
                Self::#variant_name => mav_param::NodeMut::None,
            },
            VariantType::UnnamedSingle => quote! {
                Self::#variant_name(inner) => inner.node_mut(),
            },
        });

        get_discriminant.push(match &variant_type {
            VariantType::Unit => quote! {
                Self::#variant_name => #value_variant(#discriminant_tokens),
            },
            VariantType::UnnamedSingle => quote! {
                Self::#variant_name(_) => #value_variant(#discriminant_tokens),
            },
        });

        let default = get_default_attr(&variant.attrs)?;
        set_discriminant.push(match (&variant_type, default) {
            (VariantType::Unit, _) => quote! {
                #value_variant(#discriminant_tokens) => *self = Self::#variant_name,
            },
            (VariantType::UnnamedSingle, None) => quote! {
                #value_variant(#discriminant_tokens) => *self = Self::#variant_name(Default::default()),
            },
            (VariantType::UnnamedSingle, Some(default)) => quote! {
                #value_variant(#discriminant_tokens) => *self = Self::#variant_name(#default),
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

    Ok(output)
}

/// Parse a non-negative integer discriminant, accepting decimal, hex, octal and
/// binary literals. Expressions and negative values are rejected.
fn parse_discriminant(expr: &Expr) -> syn::Result<i64> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(lit_int),
        ..
    }) = expr
    {
        parse_int_literal(lit_int)
    } else {
        Err(syn::Error::new_spanned(
            expr,
            "discriminants must be non-negative integer literals; \
             constants, expressions and negative values are not supported",
        ))
    }
}

/// Parse an integer literal in any radix into an `i64`.
fn parse_int_literal(lit: &LitInt) -> syn::Result<i64> {
    let text = lit.to_string();

    // `strip_suffix("")` returns the input, so this also handles suffix-less literals
    let body = text.strip_suffix(lit.suffix()).unwrap_or(&text);

    let (radix, digits) =
        if let Some(digits) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
            (16, digits)
        } else if let Some(digits) = body.strip_prefix("0o").or_else(|| body.strip_prefix("0O")) {
            (8, digits)
        } else if let Some(digits) = body.strip_prefix("0b").or_else(|| body.strip_prefix("0B")) {
            (2, digits)
        } else {
            (10, body)
        };

    let digits: String = digits.chars().filter(|c| *c != '_').collect();

    i64::from_str_radix(&digits, radix).map_err(|_| {
        syn::Error::new_spanned(
            lit,
            "discriminant is not a valid 64-bit non-negative integer literal",
        )
    })
}

/// Extract the enum representation type from its `#[repr(...)]` attribute.
fn get_repr_mav_param_value(
    attrs: &[Attribute],
    span: proc_macro2::Span,
) -> syn::Result<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("repr") {
            let idents = attr.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)?;

            // The repr list may contain other specifiers (e.g. `C`); pick the
            // first supported integer type.
            for ident in idents {
                let matched = match ident.to_string().as_str() {
                    "u8" => quote! { mav_param::Value::U8 },
                    "i8" => quote! { mav_param::Value::I8 },
                    "u16" => quote! { mav_param::Value::U16 },
                    "i16" => quote! { mav_param::Value::I16 },
                    "u32" => quote! { mav_param::Value::U32 },
                    "i32" => quote! { mav_param::Value::I32 },
                    _ => continue,
                };

                return Ok(matched);
            }
        }
    }

    Err(syn::Error::new(
        span,
        "Enum must have a `#[repr(inttype)]` attribute (u8, i8, u16, i16, u32 or i32)",
    ))
}

/// Extract the `#[param(default = ...)]` expression from a variant, if present.
fn get_default_attr(attrs: &[Attribute]) -> syn::Result<Option<proc_macro2::TokenStream>> {
    for meta in parse_param_attrs(attrs)? {
        if meta.path.is_ident("default") {
            return Ok(Some(meta.value.to_token_stream()));
        }
    }

    Ok(None)
}
