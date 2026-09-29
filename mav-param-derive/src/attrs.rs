use syn::{punctuated::Punctuated, Attribute, MetaNameValue, Token};

/// Collect all `name = value` entries from every `#[param(...)]` attribute.
///
/// A single attribute may contain any number of comma-separated `name = value`
/// pairs, e.g. `#[param(rename = "r", condition = "self.a != 0")]`.
pub fn parse_param_attrs(attrs: &[Attribute]) -> syn::Result<Vec<MetaNameValue>> {
    let mut out = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("param") {
            let entries =
                attr.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
            out.extend(entries);
        }
    }

    Ok(out)
}
