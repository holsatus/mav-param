use crate::{Enum, Leaf, Node, NodeMut, NodeRef, Value};

impl<T: Node + Default> Node for Option<T> {
    fn node_ref(&self) -> NodeRef<'_> {
        NodeRef::Enum(self)
    }

    fn node_mut(&mut self) -> NodeMut<'_> {
        NodeMut::Enum(self)
    }
}

impl<T: Node + Default> Leaf for Option<T> {
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

impl<'a, T: Node + Default> Enum<'a> for Option<T> {
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
