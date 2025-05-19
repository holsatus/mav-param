#![no_std]

pub mod iter;
pub mod param_impl;

pub use param_rs_derive::Tree;

pub trait Tree {
    /// Retrieve a reference to the field at a given path.
    fn get_ref<'a>(&'a self, field: &str) -> Option<EitherRef<'a>>;

    /// Retrieve a mutable reference to the field at a given path.
    fn get_mut<'a>(&'a mut self, field: &str) -> Option<EitherMut<'a>>;

    /// List all the entries (child names) at this level of the tree.
    fn entries(&self) -> &'static [&'static str];
}

/// Iterate all leaves of this node tree
pub fn leaf_iter<'a>(root: &'a dyn Tree, root_name: &str) -> iter::LeafIter<'a> {
    iter::LeafIter::new(root, root_name)
}

pub fn get_leaf_ref<'a>(mut node: &'a dyn Tree, path: &str) -> Option<LeafRef<'a>> {
    let mut segments = path.trim_start_matches('.').split('.');
    loop {
        let next = segments.next()?;
        match node.get_ref(next)? {
            EitherRef::Tree(node_ref) => node = node_ref.0,
            EitherRef::Leaf(leaf_ref) => return Some(leaf_ref),
        }
    }
}

pub fn get_leaf_mut<'a>(mut node: &'a mut dyn Tree, path: &str) -> Option<LeafMut<'a>> {
    let mut segments = path.trim_start_matches('.').split('.');
    loop {
        let next = segments.next()?;
        match node.get_mut(next)? {
            EitherMut::Tree(node_mut) => node = node_mut.0,
            EitherMut::Leaf(leaf_mut) => return Some(leaf_mut),
        }
    }
}

/// A reference to either another tree or a leaf (value)
pub enum EitherRef<'a> {
    Tree(TreeRef<'a>),
    Leaf(LeafRef<'a>),
}

/// A Mutable reference to either another tree or a leaf (value)
pub enum EitherMut<'a> {
    Tree(TreeMut<'a>),
    Leaf(LeafMut<'a>),
}

pub struct TreeRef<'a>(pub &'a dyn Tree);
pub struct TreeMut<'a>(pub &'a mut dyn Tree);

#[derive(Debug)]
pub enum LeafRef<'a> {
    U8(&'a u8),
    I8(&'a i8),
    U16(&'a u16),
    I16(&'a i16),
    U32(&'a u32),
    I32(&'a i32),
    F32(&'a f32),
}

impl LeafRef<'_> {
    pub fn as_owned(&self) -> LeafVal {
        match &self {
            LeafRef::U8(x) => LeafVal::U8(**x),
            LeafRef::I8(x) => LeafVal::I8(**x),
            LeafRef::U16(x) => LeafVal::U16(**x),
            LeafRef::I16(x) => LeafVal::I16(**x),
            LeafRef::U32(x) => LeafVal::U32(**x),
            LeafRef::I32(x) => LeafVal::I32(**x),
            LeafRef::F32(x) => LeafVal::F32(**x),
        }
    }
}

#[derive(Debug)]
pub enum LeafMut<'a> {
    U8(&'a mut u8),
    I8(&'a mut i8),
    U16(&'a mut u16),
    I16(&'a mut i16),
    U32(&'a mut u32),
    I32(&'a mut i32),
    F32(&'a mut f32),
}

impl LeafMut<'_> {
    pub fn as_owned(&self) -> LeafVal {
        match &self {
            LeafMut::U8(x) => LeafVal::U8(**x),
            LeafMut::I8(x) => LeafVal::I8(**x),
            LeafMut::U16(x) => LeafVal::U16(**x),
            LeafMut::I16(x) => LeafVal::I16(**x),
            LeafMut::U32(x) => LeafVal::U32(**x),
            LeafMut::I32(x) => LeafVal::I32(**x),
            LeafMut::F32(x) => LeafVal::F32(**x),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LeafVal {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
}

pub trait IntoEither {
    fn as_either_ref(&self) -> EitherRef<'_>;
    fn as_either_mut(&mut self) -> EitherMut<'_>;
}

// This ensure that any struct
impl<T: Tree> IntoEither for T {
    fn as_either_ref(&self) -> EitherRef<'_> {
        EitherRef::Tree(TreeRef(self))
    }

    fn as_either_mut(&mut self) -> EitherMut<'_> {
        EitherMut::Tree(TreeMut(self))
    }
}

macro_rules! leaf_into_either {
    ($($named:ident($type:ty)),+ $(,)?) => {
        $(
            impl IntoEither for $type {
                fn as_either_ref(&self) -> EitherRef<'_> {
                    EitherRef::Leaf(LeafRef::$named(self))
                }

                fn as_either_mut(&mut self) -> EitherMut<'_> {
                    EitherMut::Leaf(LeafMut::$named(self))
                }
            }
        )+
    };
}

leaf_into_either!(
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
);
