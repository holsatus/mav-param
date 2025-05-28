#![cfg_attr(not(test), no_std)]
#![warn(clippy::pedantic)]

pub use ident::Ident;
pub use value::{Value, ValueMut};

pub use mav_param_derive::{Enum, Node, Tree};

pub mod ident;
pub mod iter;
pub mod impls;
pub mod value;

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
    fn entries_full_list(&self) -> &'static [&'static str];

    fn entries(&'a self) -> ValidIter<'a>
    where
        Self: Sized,
    {
        ValidIter {
            tree: self,
            index: 0,
        }
    }
}

pub struct ValidIter<'a> {
    tree: &'a dyn Tree<'a>,
    index: usize,
}

impl<'a> Iterator for ValidIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.tree.entries_full_list().len() {
            match self.tree.entries_full_list().get(self.index) {
                Some(entry) => return Some(entry),
                _ => self.index += 1,
            }
        }
        None
    }
}

/// Iterate all values of this tree with a "root" name defined
///
/// Note: This iterator yields `Result`, since some parameter identifiers
/// may turn out to be longer than 16 bytes, or if structs are nested too deeply.
pub fn param_iter_named<'a>(tree: &'a dyn Tree<'a>, name: &str) -> iter::ParamIter<'a> {
    iter::ParamIter::new(NodeRef::Tree(tree), Some(name))
}

/// Iterate all values of this tree
///
/// Note: This iterator yields `Result`, since some parameter identifiers
/// may turn out to be longer than 16 bytes, or if structs are nested too deeply.
pub fn param_iter<'a>(tree: &'a dyn Tree<'a>) -> iter::ParamIter<'a> {
    iter::ParamIter::new(NodeRef::Tree(tree), None)
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
                NodeRef::Value(value_ref) => return Some(value_ref),
                NodeRef::None => return None,
            }
        }
    }
}

/// Returns a mutable reference to the value for the given identifier
pub fn get_value_mut<'a>(
    mut tree_mut: &'a mut dyn Tree,
    ident: &str,
) -> Option<value::ValueMut<'a>> {
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
                NodeMut::Value(value_mut) => return Some(value_mut),
                NodeMut::None => return None,
            }
        }
    }
}

/// A reference to either another tree or a value
pub enum NodeRef<'a> {
    None,
    Tree(&'a dyn Tree<'a>),
    Enum(&'a dyn Enum<'a>),
    Value(value::Value),
}

/// A mutable reference to either another tree or a value
pub enum NodeMut<'a> {
    None,
    Tree(&'a mut dyn Tree<'a>),
    Enum(&'a mut dyn Enum<'a>),
    Value(value::ValueMut<'a>),
}

/// A [`Node`] represents some named entry in a [`Param`], which may
/// be another [`Param`], or a [`value::Value`] (or [`value::ValueMut`]).
pub trait Node<'a>: Send + Sync + 'a {
    fn node_ref(&'a self) -> NodeRef<'a>;
    fn node_mut(&'a mut self) -> NodeMut<'a>;
}

macro_rules! impl_node {
    ($($named:ident($type:ty)),+ $(,)?) => {
        $(
            impl Node<'_> for $type {
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

pub trait Enum<'a>: Node<'a> + 'a {
    fn discriminant(&self) -> Value;
    fn set_discriminant(&mut self, disc: Value);
    fn discriminants_list(&self) -> &'static [Value];
    fn active_node_ref(&'a self) -> NodeRef<'a>;
    fn active_node_mut(&'a mut self) -> NodeMut<'a>;
}

impl<'a, T: Node<'a> + Default> Node<'a> for Option<T> {
    fn node_ref(&'a self) -> NodeRef<'a> {
        NodeRef::Enum(self)
    }

    fn node_mut(&'a mut self) -> NodeMut<'a> {
        NodeMut::Enum(self)
    }
}

impl<'a, T: Node<'a> + Default> Enum<'a> for Option<T> {
    fn discriminant(&self) -> Value {
        match self {
            None => Value::U8(0),
            Some(_) => Value::U8(1),
        }
    }

    fn set_discriminant(&mut self, disc: Value) {
        match disc {
            Value::U8(0) => *self = None,
            Value::U8(1) => *self = Some(Default::default()),
            _ => {}
        }
    }

    fn discriminants_list(&self) -> &'static [Value] {
        &[Value::U8(0), Value::U8(1)]
    }

    fn active_node_ref(&'a self) -> NodeRef<'a> {
        match self {
            Some(inner) => inner.node_ref(),
            None => NodeRef::None,
        }
    }

    fn active_node_mut(&'a mut self) -> NodeMut<'a> {
        match self {
            Some(inner) => inner.node_mut(),
            None => NodeMut::None,
        }
    }
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
