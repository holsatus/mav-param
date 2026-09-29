use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Expr, ExprLit, Fields, FieldsNamed,
    Ident, Lit,
};

use crate::attrs::parse_param_attrs;

pub fn tree_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match tree_derive_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn tree_derive_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Tree derive does not support generic types",
        ));
    }

    // Only structs with named fields are supported
    let fields_named = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Tree derive only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Tree derive only supports structs",
            ));
        }
    };

    generate_named_fields_impl(name, fields_named)
}

// Generate implementation for struct with named fields
fn generate_named_fields_impl(
    name: &Ident,
    fields_named: &FieldsNamed,
) -> syn::Result<proc_macro2::TokenStream> {
    // Collect field processing info
    let mut field_info = Vec::with_capacity(fields_named.named.len());

    for field in &fields_named.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let mut param_name = field_name_str.clone();
        let mut condition = None;

        for meta in parse_param_attrs(&field.attrs)? {
            let path = &meta.path;
            if path.is_ident("rename") {
                param_name = get_string_value(&meta.value, "rename")?;
            } else if path.is_ident("condition") {
                condition = Some(get_string_value(&meta.value, "condition")?);
            } else {
                return Err(syn::Error::new_spanned(
                    path,
                    "unknown `param` attribute, expected `rename` or `condition`",
                ));
            }
        }

        validate_param_name(&param_name, field_name.span())?;
        field_info.push((field_name, condition, param_name));
    }

    // Generate match arms for get_ref
    let get_ref_arms = field_info
        .iter()
        .map(|(field_name, condition, param_name)| {
            match condition {
                Some(condition_str) => {
                    // Parse the condition string into a token stream
                    let condition_expr = syn::parse_str::<syn::Expr>(condition_str)
                        .map_err(|err| syn::Error::new(field_name.span(), err))?;

                    Ok(quote! {
                        #param_name if #condition_expr => Some(self.#field_name.node_ref()),
                    })
                }
                None => Ok(quote! {
                    #param_name => Some(self.#field_name.node_ref()),
                }),
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // Generate match arms for get_mut
    let get_mut_arms = field_info
        .iter()
        .map(|(field_name, condition, param_name)| {
            match condition {
                Some(condition_str) => {
                    // Parse the condition string into a token stream
                    let condition_expr = syn::parse_str::<syn::Expr>(condition_str)
                        .map_err(|err| syn::Error::new(field_name.span(), err))?;

                    Ok(quote! {
                        #param_name if #condition_expr => Some(self.#field_name.node_mut()),
                    })
                }
                None => Ok(quote! {
                    #param_name => Some(self.#field_name.node_mut()),
                }),
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // Generate entries list
    let entry_strings = field_info.iter().map(|(_, _, param_name)| {
        quote! { #param_name }
    });

    Ok(quote! {
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
    })
}

/// Extract a string literal from an attribute value expression.
fn get_string_value(expr: &Expr, attr: &str) -> syn::Result<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        Ok(lit_str.value())
    } else {
        Err(syn::Error::new_spanned(
            expr,
            format!("`{attr}` must be a string literal"),
        ))
    }
}

/// Reject parameter names that cannot be represented unambiguously in a path.
fn validate_param_name(name: &str, span: proc_macro2::Span) -> syn::Result<()> {
    if name.is_empty() {
        return Err(syn::Error::new(span, "parameter name must not be empty"));
    }

    if let Some(bad) = name.chars().find(|c| *c == '#' || *c == '.') {
        return Err(syn::Error::new(
            span,
            format!("parameter name must not contain `{bad}`"),
        ));
    }

    Ok(())
}
