#![cfg_attr(not(test), no_std)]
#![warn(clippy::pedantic)]

pub use mav_param_derive::{Enum, Node, Tree};

pub mod ident;
pub mod impls;
pub mod iter;
pub mod value;

pub use ident::Ident;
pub use value::{Leaf, Value};

#[derive(Debug, PartialEq)]
pub enum Error {
    /// While iterating a tree, the resulting
    /// identifier would exceed 16 bytes
    PathTooLong(Ident, &'static str),

    /// While iterating a tree, the resulting
    /// identifier would exceed the depth limit.
    DepthTooBig(Ident, &'static str),

    /// The sequence is too long to be an identifier
    SequenceTooLong,

    /// The sequence is not valid utf8
    SequenceNotUtf8,
}

/// A reference to either another tree or a value
pub enum NodeRef<'a> {
    None,
    Tree(&'a dyn Tree<'a>),
    Enum(&'a dyn Enum<'a>),
    Leaf(&'a dyn Leaf),
}

/// A mutable reference to either another tree or a value
pub enum NodeMut<'a> {
    None,
    Tree(&'a mut dyn Tree<'a>),
    Enum(&'a mut dyn Enum<'a>),
    Leaf(&'a mut dyn Leaf),
}

/// A parameter combines a 16-byte identifier with a value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Parameter {
    pub ident: ident::Ident,
    pub value: value::Value,
}

pub trait Tree<'a>: Node {
    /// Retrieve a reference to the node at a given path.
    fn get_ref(&'a self, node: &str) -> Option<NodeRef<'a>>;

    /// Retrieve a mutable reference to the node at a given path.
    fn get_mut(&'a mut self, node: &str) -> Option<NodeMut<'a>>;

    /// List all the entries (child names) at this level of the tree.
    ///
    /// Note: This will also list conditionally disabled entries.
    /// To check whether an entry is avialable, ensure the value
    /// of [`Tree::get_ref`] satisfies `.is_some()`.
    fn entries(&self) -> &'static [&'static str];
}

pub trait Enum<'a>: Node + value::Leaf {
    /// List all discriminants as [`Value`] for this enum
    fn discriminants(&self) -> &'static [value::Value];

    /// Get a [`NodeRef`] to the active enum variant
    fn active_node_ref(&'a self) -> NodeRef<'a>;

    /// Get a [`NodeMut`] to the active enum variant
    fn active_node_mut(&'a mut self) -> NodeMut<'a>;
}

/// A [`Node`] represents either a [`Tree`] (struct),
/// an [`Enum`] (enum) or a [`Leaf`] (value).
pub trait Node: Send + Sync {
    /// Turn a dynamic implementor of [`Node`] into a [`NodeRef`]
    /// to allow for working with the specific variants.
    fn node_ref<'a>(&'a self) -> NodeRef<'a>;

    /// Turn a dynamic implementor of [`Node`] into a [`NodeRef`]
    /// to allow for working with the specific variants.
    fn node_mut<'a>(&'a mut self) -> NodeMut<'a>;
}

/// Iterate all values of this tree with a "root" name defined
///
/// Note: This iterator yields `Result`, since some parameter identifiers
/// may turn out to be longer than 16 bytes, or if structs are nested too deeply.
pub fn param_iter_named<'a>(node: &'a dyn Node, name: &str) -> iter::ParamIter<'a> {
    iter::ParamIter::new(node.node_ref(), Some(name))
}

/// Iterate all values of this tree
///
/// Note: This iterator yields `Result`, since some parameter identifiers
/// may turn out to be longer than 16 bytes, or if structs are nested too deeply.
pub fn param_iter(node: &dyn Node) -> iter::ParamIter<'_> {
    iter::ParamIter::new(node.node_ref(), None)
}

/// Returns the value for the given identifier
pub fn get_value(tree_ref: &dyn Node, ident: &str) -> Option<value::Value> {
    let mut segments = ident.trim_start_matches('.').split('.');
    let mut work_node = tree_ref.node_ref();
    let mut next = segments.next();

    loop {
        match work_node {
            NodeRef::None => return None,
            NodeRef::Tree(tree_ref) => {
                let segment = next.take().or_else(|| segments.next())?;
                work_node = tree_ref.get_ref(segment)?;
            }
            NodeRef::Enum(enum_ref) => {
                let segment = next.take().or_else(|| segments.next());
                work_node = if segment == Some("#") {
                    NodeRef::Leaf(enum_ref)
                } else {
                    next = segment;
                    enum_ref.active_node_ref()
                };

                continue;
            }
            NodeRef::Leaf(leaf_ref) => return Some(leaf_ref.get()),
        }
    }
}

/// Returns a mutable reference to the value for the given identifier
pub fn set_value(tree_mut: &mut dyn Node, ident: &str, value: Value) -> Option<()> {
    let mut segments = ident.trim_start_matches('.').split('.');
    let mut work_node = tree_mut.node_mut();
    let mut next = segments.next();

    loop {
        match work_node {
            NodeMut::None => return None,
            NodeMut::Tree(tree_ref) => {
                let segment = next.take().or_else(|| segments.next())?;
                work_node = tree_ref.get_mut(segment)?;
            }
            NodeMut::Enum(enum_ref) => {
                let segment = next.take().or_else(|| segments.next());
                work_node = if segment == Some("#") {
                    NodeMut::Leaf(enum_ref)
                } else {
                    next = segment;
                    enum_ref.active_node_mut()
                };

                continue;
            }
            NodeMut::Leaf(leaf_ref) => return leaf_ref.set(value).then(|| ()),
        }
    }
}
