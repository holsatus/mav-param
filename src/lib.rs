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

pub trait Tree<'a>: Node<'a> + 'a {
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

pub trait Enum<'a>: Node<'a> + value::Leaf + 'a {
    /// List all discriminants as [`Value`] for this enum
    fn discriminants(&self) -> &'static [value::Value];

    /// Get a [`NodeRef`] to the active enum variant
    fn active_node_ref(&'a self) -> NodeRef<'a>;

    /// Get a [`NodeMut`] to the active enum variant
    fn active_node_mut(&'a mut self) -> NodeMut<'a>;
}

/// A [`Node`] represents either a [`Tree`] (struct),
/// an [`Enum`] (enum) or a [`Leaf`] (value).
pub trait Node<'a>: Send + Sync + 'a {
    /// Turn a dynamic implementor of [`Node`] into a [`NodeRef`]
    /// to allow for working with the specific variants.
    fn node_ref(&'a self) -> NodeRef<'a>;

    /// Turn a dynamic implementor of [`Node`] into a [`NodeRef`]
    /// to allow for working with the specific variants.
    fn node_mut(&'a mut self) -> NodeMut<'a>;
}

/// Iterate all values of this tree with a "root" name defined
///
/// Note: This iterator yields `Result`, since some parameter identifiers
/// may turn out to be longer than 16 bytes, or if structs are nested too deeply.
pub fn param_iter_named<'a>(node: &'a impl Node<'a>, name: &str) -> iter::ParamIter<'a> {
    iter::ParamIter::new(node.node_ref(), Some(name))
}

/// Iterate all values of this tree
///
/// Note: This iterator yields `Result`, since some parameter identifiers
/// may turn out to be longer than 16 bytes, or if structs are nested too deeply.
pub fn param_iter<'a>(node: &'a impl Node<'a>) -> iter::ParamIter<'a> {
    iter::ParamIter::new(node.node_ref(), None)
}

/// Returns the value for the given identifier
pub fn get_value(mut tree_ref: &dyn Tree, ident: &str) -> Option<value::Value> {
    let mut segments = ident.trim_start_matches('.').split('.');
    'tree_loop: loop {
        let next = segments.next()?;
        let mut node_ref = tree_ref.get_ref(next)?;

        'enum_loop: loop {
            match node_ref {
                NodeRef::Enum(enum_ref) => {
                    node_ref = enum_ref.active_node_ref();
                    continue 'enum_loop;
                }
                NodeRef::Tree(new_tree_ref) => {
                    tree_ref = new_tree_ref;
                    continue 'tree_loop;
                }
                NodeRef::Leaf(value_ref) => {
                    return Some(value_ref.get())
                },
                NodeRef::None => return None,
            }
        }
    }
}

/// Returns a mutable reference to the value for the given identifier
pub fn set_value<'a>(tree_mut: &'a mut dyn Tree, ident: &str, value: Value) -> bool {
    let inner = || {
        let mut tree_mut = tree_mut;
        let mut segments = ident.trim_start_matches('.').split('.');
        'tree_loop: loop {
            let next = segments.next()?;
            let mut node_mut = tree_mut.get_mut(next)?;

            'enum_loop: loop {
                match node_mut {
                    NodeMut::Enum(enum_ref) => {
                        node_mut = enum_ref.active_node_mut();
                        continue 'enum_loop;
                    }
                    NodeMut::Tree(new_tree_mut) => {
                        tree_mut = new_tree_mut;
                        continue 'tree_loop;
                    }
                    NodeMut::Leaf(value_mut) => {
                        return value_mut.set(value).then(||());
                    }
                    NodeMut::None => return None,
                }
            }
        }
    };

    // Only interested in the boolean
    inner().is_some()
}

#[cfg(test)]
mod tests {
    use crate::{self as mav_param};
    use mav_param::{Tree, param_iter_named};

    #[test]
    fn basic_derive() {
        #[derive(mav_param::Tree, Default)]
        struct Params {
            entry1: u8,
            entry2: u8,
            var: Union,
        }

        #[repr(u8)]
        #[derive(mav_param::Enum)]
        enum Union {
            Var1(Inner) = 0,
            Var2(f32) = 1,
        }

        #[derive(mav_param::Tree, Default)]
        struct Inner {
            i1: u8,
            i2: u8,
            i3: u8,
        }

        impl Default for Union {
            fn default() -> Self {
                Union::Var1(Default::default())
            }
        }

        let mut test = Params::default();

        let params_vec = param_iter_named(&test, "param")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec![
            "param.entry1".to_owned(),
            "param.entry2".to_owned(),
            "param.var.#".to_owned(),
            "param.var.i1".to_owned(),
            "param.var.i2".to_owned(),
            "param.var.i3".to_owned(),
        ];

        assert_eq!(names, params_vec);

        test.var = Union::Var2(2.0);

        let params_vec = param_iter_named(&test, "param")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec![
            "param.entry1".to_owned(),
            "param.entry2".to_owned(),
            "param.var.#".to_owned(),
            "param.var".to_owned(),
        ];

        assert_eq!(names, params_vec)
    }

    #[test]
    fn option_enum() {
        #[derive(mav_param::Tree, Default)]
        struct Params {
            entry1: u8,
            entry2: u8,
            var: Option<(f32, f32)>,
            float1: f32,
            float2: f32,
        }

        let mut test = Params::default();

        let params_vec = param_iter_named(&test, "param")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec![
            "param.entry1".to_owned(),
            "param.entry2".to_owned(),
            "param.var.#".to_owned(),
            "param.float1".to_owned(),
            "param.float2".to_owned(),
        ];

        assert_eq!(names, params_vec);

        test.var = Some((2.0, 3.0));

        let params_vec = param_iter_named(&test, "param")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec![
            "param.entry1".to_owned(),
            "param.entry2".to_owned(),
            "param.var.#".to_owned(),
            "param.var.0".to_owned(),
            "param.var.1".to_owned(),
            "param.float1".to_owned(),
            "param.float2".to_owned(),
        ];

        assert_eq!(names, params_vec)
    }

    #[test]
    fn basic_derive_renamed() {
        #[derive(Tree, Default)]
        struct NestedParams {
            #[tree(rename = "e1")]
            entry1: u8,
            #[tree(rename = "e2")]
            entry2: u8,
        }

        #[derive(Tree, Default)]
        struct TestParams {
            entry1: u8,
            entry2: u8,
            nest: NestedParams,
        }

        let test = TestParams::default();

        let params_vec = param_iter_named(&test, "test")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec![
            "test.entry1".to_owned(),
            "test.entry2".to_owned(),
            "test.nest.e1".to_owned(),
            "test.nest.e2".to_owned(),
        ];

        assert_eq!(names, params_vec)
    }

    #[test]
    fn basic_derive_conditional() {
        #[derive(Tree, Default)]
        struct NestedParams {
            entry1: u8,
            entry2: u8,
        }

        #[derive(Tree, Default)]
        struct TestParams {
            entry1: u8,
            entry2: u8,
            #[tree(condition = "self.entry1 != self.entry2")]
            nest: NestedParams,
        }

        let mut test = TestParams::default();

        let params_vec = param_iter_named(&test, "test")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec!["test.entry1".to_owned(), "test.entry2".to_owned()];

        assert_eq!(names, params_vec);

        test.entry1 += 1;

        let params_vec = param_iter_named(&test, "test")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec![
            "test.entry1".to_owned(),
            "test.entry2".to_owned(),
            "test.nest.entry1".to_owned(),
            "test.nest.entry2".to_owned(),
        ];

        assert_eq!(names, params_vec)
    }

    #[test]
    fn derive_conditional_and_rename() {
        #[derive(Tree, Default)]
        struct NestedParams {
            entry1: u8,
            entry2: u8,
        }

        #[derive(Tree, Default)]
        struct TestParams {
            entry1: u8,
            entry2: u8,
            #[tree(rename = "n")]
            #[tree(condition = "self.entry1 != self.entry2")]
            nest: NestedParams,
        }

        let mut test = TestParams::default();

        let params_vec = param_iter_named(&test, "test")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec!["test.entry1".to_owned(), "test.entry2".to_owned()];

        assert_eq!(names, params_vec);

        test.entry1 += 1;

        let params_vec = param_iter_named(&test, "test")
            .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
            .take(20)
            .collect::<Vec<_>>();

        let names = vec![
            "test.entry1".to_owned(),
            "test.entry2".to_owned(),
            "test.n.entry1".to_owned(),
            "test.n.entry2".to_owned(),
        ];

        assert_eq!(names, params_vec)
    }
}
