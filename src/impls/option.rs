use crate::{Enum, Leaf, Node, NodeMut, NodeRef, Value};

impl<'a, T: Node<'a> + Default> Node<'a> for Option<T> {
    fn node_ref(&'a self) -> NodeRef<'a> {
        NodeRef::Enum(self)
    }

    fn node_mut(&'a mut self) -> NodeMut<'a> {
        NodeMut::Enum(self)
    }
}

impl<'a, T: Node<'a> + Default> Leaf for Option<T> {
    fn get(&self) -> Value {
        match self {
            None => Value::U8(0),
            Some(_) => Value::U8(1),
        }
    }

    fn set(&mut self, val: Value) -> bool {
        match val {
            Value::U8(0) => *self = None,
            Value::U8(1) => *self = Some(Default::default()),
            _ => return false,
        }
        return true;
    }
}

impl<'a, T: Node<'a> + Default> Enum<'a> for Option<T> {
    fn discriminants(&self) -> &'static [Value] {
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
