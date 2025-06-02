use crate::{Node, NodeMut, NodeRef, Tree};

macro_rules! impl_tree_array {
    (
        $arraylen:literal =>
        $(
            $num:literal
        ),* $(,)?
    ) => {

        impl<T: Node> Node for [T; $arraylen] {
            fn node_ref(&self) -> NodeRef<'_> {
                NodeRef::Tree(self)
            }

            fn node_mut(&mut self) -> NodeMut<'_> {
                NodeMut::Tree(self)
            }
        }

        impl<'a, T: Node> Tree<'a> for [T; $arraylen] {
            fn get_ref(&'a self, path: &str) -> Option<NodeRef<'a>> {
                match path {
                    $(
                        stringify!($num) => Some(self[$num].node_ref()),
                    )*
                    _ => None,
                }
            }

            fn get_mut(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
                match path {
                    $(
                        stringify!($num) => Some(self[$num].node_mut()),
                    )*
                    _ => None,
                }
            }

            fn entries(&self) -> &'static [&'static str] {
                &[
                    $(
                        stringify!($num),
                    )*
                ]
            }
        }
    };
}

impl_tree_array! {1 => 0}
impl_tree_array! {2 => 0, 1}
impl_tree_array! {3 => 0, 1, 2}
impl_tree_array! {4 => 0, 1, 2, 3}
impl_tree_array! {5 => 0, 1, 2, 3, 4}
impl_tree_array! {6 => 0, 1, 2, 3, 4, 5}
impl_tree_array! {7 => 0, 1, 2, 3, 4, 5, 6}
impl_tree_array! {8 => 0, 1, 2, 3, 4, 5, 6, 7}
impl_tree_array! {9 => 0, 1, 2, 3, 4, 5, 6, 7, 8}
impl_tree_array! {10 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
impl_tree_array! {11 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
impl_tree_array! {12 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11}
impl_tree_array! {13 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12}
impl_tree_array! {14 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13}
impl_tree_array! {15 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14}
impl_tree_array! {16 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15}
impl_tree_array! {17 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16}
impl_tree_array! {18 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17}
impl_tree_array! {19 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18}
impl_tree_array! {20 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19}
