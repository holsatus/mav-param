use crate::{Node, NodeMut, NodeRef, Tree};

macro_rules! impl_tree_tuple {
    ( $(
        $type:ident; $number:tt
    ),* $(,)?) => {
        
        impl<$($type: Node),+> Node for ( $($type,)+ ) {
            fn node_ref(&self) -> NodeRef<'_> {
                NodeRef::Tree(self)
            }

            fn node_mut(&mut self) -> NodeMut<'_> {
                NodeMut::Tree(self)
            }
        }

        impl<
            'a,
            $(
                $type: Node
            ),*
        > Tree<'a> for ($(
                $type
            ),*, )
        {
            fn get_ref(&'a self, path: &str) -> Option<NodeRef<'a>> {
                match path {
                    $(
                        stringify!($number) => Some(self.$number.node_ref()),
                    )*
                    _ => None,
                }
            }

            fn get_mut(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
                match path {
                    $(
                        stringify!($number) => Some(self.$number.node_mut()),
                    )*
                    _ => None,
                }
            }

            fn entries(&self) -> &'static [&'static str] {
                &[$(stringify!($number)),*]
            }
        }

    };
}

impl_tree_tuple!{T0; 0}
impl_tree_tuple!{T0; 0, T1; 1}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2, T3; 3}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2, T3; 3, T4; 4}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2, T3; 3, T4; 4, T5; 5}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2, T3; 3, T4; 4, T5; 5, T6; 6}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2, T3; 3, T4; 4, T5; 5, T6; 6, T7; 7}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2, T3; 3, T4; 4, T5; 5, T6; 6, T7; 7, T8; 8}
impl_tree_tuple!{T0; 0, T1; 1, T2; 2, T3; 3, T4; 4, T5; 5, T6; 6, T7; 7, T8; 8, T9; 9}
