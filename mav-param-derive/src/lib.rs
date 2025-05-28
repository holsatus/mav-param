use proc_macro::TokenStream;

mod derive_enum;
mod derive_node;
mod derive_tree;

/// Derives the `Enum` trait for an enum type.
///
/// This macro automatically implements the `Enum` and `Node` traits for enums, enabling
/// them to be used within the parameter tree as tagged unions.
///
/// Each variant must have a single field and a discriminant value.
///
/// Example:
/// ```rust
/// #[repr(u8)]
/// #[derive(mav_param::Enum)]
/// enum Vehicle {
///     Car(CarConfig) = 0,
///     Bike(BikeConfig) = 1,
///     Boat(BoatConfig) = 2,
/// }
/// ```
#[proc_macro_derive(Enum, attributes(enum_variant))]
pub fn enum_derive(input: TokenStream) -> TokenStream {
    derive_enum::enum_derive(input)
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
    derive_node::node_derive(input)
}

/// Derives the `Tree` trait for a struct.
///
/// This macro automatically implements the `Tree` trait for a struct, allowing it
/// to be part of a parameter hierarchy. Each field of the struct becomes an entry
/// in the parameter tree.
///
/// Use the `#[tree(rename = "name")]` attribute to customize field names in the tree.
#[proc_macro_derive(Tree, attributes(tree))]
pub fn tree_derive(input: TokenStream) -> TokenStream {
    derive_tree::tree_derive(input)
}
