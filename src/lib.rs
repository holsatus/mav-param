#![cfg_attr(not(test), no_std)]

pub mod ident;
pub mod iter;
pub mod tree_impls;
pub mod value;

pub use ident::Ident;
pub use value::Value;

pub use param_rs_derive::{Node, Tree};

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

/// A parameter combines a 16-byte identifier with a value.
pub struct Parameter {
    pub ident: ident::Ident,
    pub value: value::Value,
}

pub trait Tree {
    /// Retrieve a reference to the node at a given path.
    fn get_ref<'a>(&'a self, node: &str) -> Option<NodeRef<'a>>;

    /// Retrieve a mutable reference to the node at a given path.
    fn get_mut<'a>(&'a mut self, node: &str) -> Option<NodeMut<'a>>;

    /// List all the entries (child names) at this level of the tree.
    fn entries(&self) -> &'static [&'static str];
}

/// Iterate all values of this tree with a "root" name defined
pub fn param_iter_named<'a>(tree: &'a dyn Tree, name: &str) -> iter::ValueIter<'a> {
    iter::ValueIter::new(tree, Some(name))
}

/// Iterate all values of this tree
pub fn param_iter<'a>(tree: &'a dyn Tree) -> iter::ValueIter<'a> {
    iter::ValueIter::new(tree, None)
}

/// Returns the value for the given identifier
pub fn get_value<'a>(mut tree: &'a dyn Tree, ident: &str) -> Option<value::Value> {
    let mut segments = ident.trim_start_matches('.').split('.');
    loop {
        let next = segments.next()?;
        match tree.get_ref(next)? {
            NodeRef::Tree(node_ref) => tree = node_ref,
            NodeRef::Value(leaf_ref) => return Some(leaf_ref),
        }
    }
}

/// Returns a mutable reference to the value for the given identifier
pub fn get_value_mut<'a>(mut node: &'a mut dyn Tree, ident: &str) -> Option<value::ValueMut<'a>> {
    let mut segments = ident.trim_start_matches('.').split('.');
    loop {
        let next = segments.next()?;
        match node.get_mut(next)? {
            NodeMut::Tree(node_mut) => node = node_mut,
            NodeMut::Value(leaf_mut) => return Some(leaf_mut),
        }
    }
}

/// A reference to either another tree or a value
pub enum NodeRef<'a> {
    Tree(&'a dyn Tree),
    Value(value::Value),
}

/// A mutable reference to either another tree or a value
pub enum NodeMut<'a> {
    Tree(&'a mut dyn Tree),
    Value(value::ValueMut<'a>),
}

/// A [`Node`] represents some named entry in a [`Param`], which may
/// be another [`Param`], or a [`value::Value`] (or [`value::ValueMut`]).
pub trait Node {
    fn node_ref(&self) -> NodeRef<'_>;
    fn node_mut(&mut self) -> NodeMut<'_>;
}

impl<T: Tree> Node for T {
    fn node_ref(&self) -> NodeRef<'_> {
        NodeRef::Tree(self)
    }

    fn node_mut(&mut self) -> NodeMut<'_> {
        NodeMut::Tree(self)
    }
}

macro_rules! impl_node {
    ($($named:ident($type:ty)),+ $(,)?) => {
        $(
            impl Node for $type {
                fn node_ref(&self) -> NodeRef<'_> {
                    NodeRef::Value(value::Value::$named(*self))
                }

                fn node_mut(&mut self) -> NodeMut<'_> {
                    NodeMut::Value(value::ValueMut::$named(self))
                }
            }
        )+
    };
}

impl_node!(
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
);
